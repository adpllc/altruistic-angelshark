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

async fn remove_entries(entries: Entries, runner: AcmRunner) -> Result<impl Reply, Infallible> {
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

impl From<Entry> for Vec<Message> {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::StationUser { acm, ext } => {
                vec![Message::new(&format!("clear amw all {}", ext))]
            }
            Entry::AgentLoginId { acm, ext } => {
                todo!()
            }
        }
    }
}
