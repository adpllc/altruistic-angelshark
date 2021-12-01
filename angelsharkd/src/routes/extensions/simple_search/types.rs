use anyhow::{anyhow, Context, Error};
use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::error;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

const ANGELSHARKD_EXT_SEARCH_ACMS: &str = "ANGELSHARKD_EXT_SEARCH_ACMS";
const OSSI_STAT_NUMBER_FIELD: &str = "8005ff00";
const OSSI_STAT_ROOM_FIELD: &str = "0031ff00";
const OSSI_LIST_STAT_CMD: &str = "list station";
const OSSI_LIST_EXT_CMD: &str = "list extension-type";

/// Collection of search terms
pub type Needles = Vec<String>;

/// Collection of ACM extension types with ROOMs (if applicable) and ACM names
type HaystackEntries = Vec<Vec<String>>;

/// Represents a searchable, refreshable collection of ACM extension data.
#[derive(Clone)]
pub struct Haystack {
    entries: Arc<Mutex<HaystackEntries>>,
    runner: AcmRunner,
}

impl Haystack {
    pub fn new(runner: AcmRunner) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            runner,
        }
    }

    /// Searches for haystack entries that contain all given needles and returns them.
    pub fn search(&self, needles: Needles) -> Result<HaystackEntries, Error> {
        let needles: Vec<_> = needles.iter().map(|n| n.to_lowercase()).collect();

        let entries = self
            .entries
            .lock()
            .map_err(|e| anyhow!(e.to_string()))
            .with_context(|| "Failed to get haystack inner data lock.")?;

        let matches = entries
            .iter()
            .filter(|entry| {
                let entry_str = entry.join("").to_lowercase();
                needles
                    .iter()
                    .all(|needle| entry_str.contains(needle.as_str()))
            })
            .cloned()
            .collect();
        Ok(matches)
    }

    /// Refreshes the haystack data by running relevant commands on a runner,
    /// parsing the results, and updating the entries field with the fresh data.
    /// TODO: Do we want simultaneous refreshes to be possible?
    /// TODO: The entry generation could probably be simplified and the number of clones reduced.
    pub fn refresh(&self) -> Result<(), Error> {
        let mut runner = self.runner.to_owned();

        // Queue jobs in ACM runner
        let configured_acms = env::var(ANGELSHARKD_EXT_SEARCH_ACMS).with_context(|| {
            format!(
                "{} var missing. Cannot refresh haystack.",
                ANGELSHARKD_EXT_SEARCH_ACMS
            )
        })?;

        // Generate jobs and queue on runner.
        for acm in configured_acms.split_whitespace() {
            runner
                .queue_input(acm, &Message::new(OSSI_LIST_EXT_CMD))
                .queue_input(
                    acm,
                    &Message {
                        command: String::from(OSSI_LIST_STAT_CMD),
                        fields: Some(vec![
                            String::from(OSSI_STAT_NUMBER_FIELD),
                            String::from(OSSI_STAT_ROOM_FIELD),
                        ]),
                        datas: None,
                        error: None,
                    },
                );
        }

        // Run jobs and collect output. Filter out uneeded commands, combine errors.
        let output: Result<Vec<(String, Vec<Message>)>, Error> = runner
            .run()
            .map(|(name, output)| {
                let output: Vec<Message> = output?
                    .into_iter()
                    .filter(|m| m.command != "logoff")
                    .collect();
                Ok((name, output))
            })
            .collect();
        let output = output.with_context(|| "Failed to run refresh commands on ACM(s).")?;

        // Log any ACM errors encountered
        for error in output
            .iter()
            .map(|(_, messages)| messages)
            .flatten()
            .filter_map(|m| m.error.as_ref())
        {
            error!("ACM error: {}", error);
        }

        // Build a map of station number-to-rooms
        let rooms: HashMap<String, String> = output
            .iter()
            .map(|(_, messages)| {
                messages
                    .iter()
                    .filter(|message| message.command == OSSI_LIST_STAT_CMD)
                    .filter_map(|message| message.datas.to_owned())
                    .flatten()
                    .filter_map(|stat| Some((stat.get(0)?.to_owned(), stat.get(1)?.to_owned())))
            })
            .flatten()
            .collect();

        // Build the haystack from the room map and extension-type output
        let haystack: HaystackEntries = output
            .into_iter()
            .map(|(acm_name, messages)| {
                let rooms = &rooms;
                let acm_name = format!("CM{}", acm_name);

                messages
                    .into_iter()
                    .filter(|message| message.command == OSSI_LIST_EXT_CMD)
                    .filter_map(|message| message.datas)
                    .flatten()
                    .map(move |mut extension| {
                        let room = extension
                            .get(0)
                            .map(|num| rooms.get(num))
                            .flatten()
                            .map(|room| room.to_owned())
                            .unwrap_or_else(String::new);

                        extension.push(acm_name.to_owned());
                        extension.push(room);
                        extension
                    })
            })
            .flatten()
            .collect();

        // Overwrite shared haystack entries with new data.
        let mut lock = self
            .entries
            .lock()
            .map_err(|e| anyhow!(e.to_string()))
            .with_context(|| "Failed to get haystack inner data lock.")?;
        *lock = haystack;
        Ok(())
    }
}
