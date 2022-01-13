use crate::routes::dtos::{Error, Response};
use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::error;
use serde::Deserialize;
use warp::{
    body::{content_length_limit, json},
    hyper::StatusCode,
    path, post, reply, Filter, Rejection, Reply,
};

const SIXTEEN_K: u64 = 1024 * 16;

pub fn busy_filter(
    runner: AcmRunner,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    post()
        .and(path!("service" / "busyout" / ..))
        .and(content_length_limit(SIXTEEN_K))
        .and(json())
        .map(move |entries| queue_and_run(entries, "busyout", runner.to_owned()))
}

pub fn release_filter(
    runner: AcmRunner,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    post()
        .and(path!("service" / "release" / ..))
        .and(content_length_limit(SIXTEEN_K))
        .and(json())
        .map(move |entries| queue_and_run(entries, "release", runner.to_owned()))
}

pub fn toggle_filter(
    runner: AcmRunner,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    post()
        .and(path!("service" / "toggle" / ..))
        .and(content_length_limit(SIXTEEN_K))
        .and(json())
        .map(move |entries| queue_and_run(entries, "toggle", runner.to_owned()))
}

fn queue_and_run(entries: Entries, command: &str, mut runner: AcmRunner) -> impl Reply {
    for entry in entries.into_iter() {
        if command == "toggle" {
            runner.queue_input(
                &entry.acm,
                &Message::new(&format!("busyout station {}", entry.ext)),
            );
            runner.queue_input(
                &entry.acm,
                &Message::new(&format!("release station {}", entry.ext)),
            );
        } else {
            runner.queue_input(
                &entry.acm,
                &Message::new(&format!("{} station {}", command, entry.ext)),
            );
        }
    }

    // generate output on runner
    let output: Result<Vec<Vec<_>>, _> = runner
        .run()
        .map(|(name, output)| -> Result<Vec<Response>, anyhow::Error> {
            Ok(output?
                .into_iter()
                .filter_map(move |msg| {
                    (msg.command != "logoff").then(|| Response::from((name.to_owned(), msg)))
                })
                .collect())
        })
        .collect();

    // handle errors and package output as json
    match output {
        Err(e) => {
            error!("busyout-release extension: {}", e);
            reply::with_status(
                reply::json(&Error {
                    reason: e.to_string(),
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
        Ok(r) => reply::with_status(
            reply::json(&r.into_iter().flatten().collect::<Vec<_>>()),
            StatusCode::OK,
        ),
    }
}

type Entries = Vec<Entry>;
#[derive(Debug, Deserialize)]
struct Entry {
    acm: String,
    ext: String,
}
