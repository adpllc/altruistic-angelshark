use std::{convert::Infallible, fmt::Display};

use libangelshark::{AcmRunner, Message};
use serde::Deserialize;
use warp::{
    body::{content_length_limit, json},
    post, Filter, Rejection, Reply,
};

pub fn filter(runner: AcmRunner) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("deprov")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .and_then(move |entries: Entries| remove_entries(entries, runner.to_owned()))
}

async fn remove_entries(entries: Entries, mut runner: AcmRunner) -> Result<impl Reply, Infallible> {
    for entry in entries {
        match entry {
            Entry::StationUser { acm, ext } => {
                runner.queue_input(&acm, &Message::new(&format!("clear amw all {}", ext)));
                runner.queue_input(&acm, &Message::new(&format!("remove station {}", ext)));
            }
            Entry::AgentLoginId { acm, ext } => {
                runner.queue_input(
                    &acm,
                    &Message::new(&format!("remove agent-loginID {}", ext)),
                );
            }
        }
    }
    dbg!(&runner);
    Ok("")
}

type Entries = Vec<Entry>;

#[derive(Debug, Deserialize)]
enum Entry {
    #[serde(rename(deserialize = "station-user"))]
    StationUser { acm: String, ext: String },
    #[serde(rename(deserialize = "agent-loginid"))]
    AgentLoginId { acm: String, ext: String },
}
