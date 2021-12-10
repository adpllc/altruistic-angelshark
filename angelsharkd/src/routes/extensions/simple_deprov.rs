use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::error;
use serde::Deserialize;
use std::convert::Infallible;
use warp::{
    body::{content_length_limit, json},
    post, reply, Filter, Rejection, Reply,
};

const SIXTEEN_K: u64 = 1024 * 16;

/// Returns a warp filter to handle HTTP POSTs for deprovisioning stations, agents, etc.
pub fn filter(runner: AcmRunner) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("deprov")
        .and(post())
        .and(content_length_limit(SIXTEEN_K))
        .and(json())
        .and_then(move |entries| remove_entries(entries, runner.to_owned()))
}

/// Queues removal commands for [Entries] on an [AcmRunner]. Gathers any errors encountered and returns those.
async fn remove_entries(entries: Entries, mut runner: AcmRunner) -> Result<impl Reply, Infallible> {
    // Construct OSSI messages to carry out removals.
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

    // Gather any errors encountered and format them for the client response.
    let errors: Vec<String> = runner
        .run_cached()
        .map(|(acm, output)| match output {
            Ok(messages) => messages
                .into_iter()
                .filter_map(|message| Some(format!("ACM {}: {}", acm.clone(), message.error?)))
                .collect(),
            Err(error) => vec![format!("ACM {}: {}", acm, error)],
        })
        .flatten()
        .collect();

    // Log errors for tracking.
    for error in &errors {
        error!("deprov error: {}", error);
    }

    Ok(reply::json(&errors))
}

/// Collection of [Entry].
type Entries = Vec<Entry>;

/// Very basic [Deserialize] target for deprov inputs. Going from stringly typed to strongly typed.
#[derive(Debug, Deserialize)]
enum Entry {
    #[serde(rename(deserialize = "station-user"))]
    StationUser { acm: String, ext: String },
    #[serde(rename(deserialize = "agent-loginid"))]
    AgentLoginId { acm: String, ext: String },
}
