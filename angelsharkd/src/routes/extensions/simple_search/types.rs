use anyhow::{anyhow, Context, Error};
use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::{error, info};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    pin::Pin,
    sync::{Arc, Mutex},
};

const ANGELSHARKD_EXT_SEARCH_ACMS: &str = "ANGELSHARKD_EXT_SEARCH_ACMS";
const OSSI_STAT_NUMBER_FIELD: &str = "8005ff00";
const OSSI_STAT_ROOM_FIELD: &str = "0031ff00";
const OSSI_LIST_STAT_CMD: &str = "list station";
const OSSI_LIST_EXT_CMD: &str = "list extension-type";

#[derive(Deserialize)]
pub struct Query {
    pub limit: Option<usize>,
}

/// Collection of search terms
pub type Needles = Vec<String>;

/// Collection of ACM extension types with ROOMs (if applicable) and ACM names
type HaystackEntries = Vec<Vec<String>>;

/// Represents a searchable, refreshable collection of ACM extension data.
///
/// Note: why use a Arc->Mutex->Pin->Box? The `Arc<Mutex<>>` is required because
/// this haystack is borrowed across threads. I want all haystack refreshing to
/// take place on a separate thread so that searches can still be executed while
/// a background refresh takes place.
///
/// The `Pin<Box<>>` is used to ensure that the big block of new entries
/// generated during a refresh is pinned to one allocated portion of the heap.
/// This should prevent expensive moves or stack popping for that large chunk of
/// data which goes unchanged until it is dropped at the next refresh.
#[derive(Clone)]
pub struct Haystack {
    entries: Arc<Mutex<Pin<Box<HaystackEntries>>>>,
    runner: AcmRunner,
}

impl Haystack {
    pub fn new(runner: AcmRunner) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Pin::new(Box::new(Vec::new())))),
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
    /// Note: multiple simultaenous refresh calls could starve the Rayon thread
    /// pool used by `libangelshark`. The entry generation could probably be
    /// simplified and the number of clones reduced.
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
            .flat_map(|(_, messages)| messages)
            .filter_map(|m| m.error.as_ref())
        {
            error!("ACM error: {}", error);
        }

        // Build a map of station number-to-rooms
        let rooms: HashMap<String, String> = output
            .iter()
            .flat_map(|(_, messages)| {
                messages
                    .iter()
                    .filter(|message| message.command == OSSI_LIST_STAT_CMD)
                    .filter_map(|message| message.datas.to_owned())
                    .flatten()
                    .filter_map(|stat| Some((stat.get(0)?.to_owned(), stat.get(1)?.to_owned())))
            })
            .collect();

        // Build the haystack from the room map and extension-type output
        let haystack: HaystackEntries = output
            .into_iter()
            .flat_map(|(acm_name, messages)| {
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
                            .and_then(|num| rooms.get(num))
                            .map(|room| room.to_owned())
                            .unwrap_or_else(String::new);

                        extension.push(acm_name.to_owned());
                        extension.push(room);
                        extension
                    })
            })
            .collect();

        // Calculate some helpful statistics about downloaded extension data.
        let mut counts = HashMap::new();
        for entry in haystack.iter().filter_map(|entry| {
            Some((
                entry.get(8)?,
                entry.get(1)? == "station-user",
                !entry.get(9)?.is_empty(),
            ))
        }) {
            let (acm, is_station, has_room) = entry;

            match (is_station, has_room) {
                (true, true) => {
                    let stat_room = counts.entry(format!("{acm}_stat_room")).or_insert(0);
                    *stat_room += 1;
                }
                (true, false) => {
                    let stat_no_room = counts.entry(format!("{acm}_stat_noroom")).or_insert(0);
                    *stat_no_room += 1;
                }
                _ => {
                    let other = counts.entry(format!("{acm}_other")).or_insert(0);
                    *other += 1;
                }
            }
        }

        // Log found statistics.
        let total = haystack.len();
        info!("Downloaded {total} fresh extension-types. Writing stats to STDERR.");
        eprintln!(
            "{{ {} }}",
            counts
                .iter()
                .map(|(stat, count)| format!("\"{stat}\": {count}"))
                .collect::<Vec<_>>()
                .join(",")
        );

        let haystack = Pin::new(Box::new(haystack));

        // Overwrite shared haystack entries with new data.
        let mut lock = self
            .entries
            .lock()
            .map_err(|e| anyhow!(e.to_string()))
            .with_context(|| "Failed to get haystack inner data lock.")?;

        // Note to developers: this is the reason for the haystack entries to be
        // a pinned box. A box heap allocates and a pin guarantees the entries
        // won't be moved from their existing memory location. This is done with
        // the intent of speeding up this swap by preventing moves or
        // pushing/popping of the stack with large data sets. There may be a
        // better way to implement this.
        *lock = haystack;
        Ok(())
    }
}
