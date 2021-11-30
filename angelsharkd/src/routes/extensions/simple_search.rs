use anyhow::{anyhow, Context, Error};
use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::{error, info};
use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
};
use warp::{
    body::{content_length_limit, json},
    get,
    hyper::{header, StatusCode},
    path, post,
    reply::{self, with},
    Filter, Rejection, Reply,
};

/// Collection of search terms
type Needles = Vec<String>;

/// Collection of ACM extension types with ROOMs (if applicable) and ACM names
type HaystackEntries = Vec<Vec<String>>;

pub fn search(haystack: Haystack) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("search")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .and_then(move |terms: Needles| handle_search(haystack.to_owned(), terms))
        .with(with::header(header::PRAGMA, "no-cache"))
        .with(with::header(header::CACHE_CONTROL, "no-store, max-age=0"))
        .with(with::header(header::X_FRAME_OPTIONS, "DENY"))
}

pub fn refresh(haystack: Haystack) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("search" / "refresh")
        .and(get())
        .and_then(move || handle_refresh(haystack.to_owned()))
}

async fn handle_search(haystack: Haystack, needle: Needles) -> Result<impl Reply, Infallible> {
    // Ok(haystack.search(Vec::new())?)
    // if let Ok(matches = haystack.search(needle);
    match haystack.search(needle) {
        Ok(matches) => Ok(reply::with_status(reply::json(&matches), StatusCode::OK)),
        Err(e) => Ok(reply::with_status(
            reply::json(&e.to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

async fn handle_refresh(haystack: Haystack) -> Result<impl Reply, Infallible> {
    // Run refresh as a background task and immediately return.
    tokio::spawn(async move {
        if let Err(e) = haystack.refresh() {
            error!("{}", e.to_string()); // TODO: use logger
        } else {
            info!("Search haystack refreshed.");
        }
    });

    Ok("Refresh scheduled")
}

/// A lazy-loaded, asynchronously-refreshed exension-type haystack cache.
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

    pub fn refresh(&self) -> Result<(), Error> {
        let mut runner = self.runner.to_owned(); // TODO: This alone allows multiple refreshers to be run at once. Do we want that?

        // Queue jobs in ACM runner
        for acm in &[
            "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11",
        ] {
            runner
                .queue_input(acm, &Message::new("list extension-type"))
                .queue_input(
                    acm,
                    &Message {
                        command: String::from("list station"),
                        fields: Some(vec![String::from("8005ff00"), String::from("0031ff00")]),
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
                    .filter(|message| message.command == "list station")
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
                    .filter(|message| message.command == "list extension-type")
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

        // Propogate the new data to the shared haystack entries
        let mut lock = self
            .entries
            .lock()
            .map_err(|e| anyhow!(e.to_string()))
            .with_context(|| "Failed to get haystack inner data lock.")?;
        *lock = haystack;
        Ok(())
    }
}
