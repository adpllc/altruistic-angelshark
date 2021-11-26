use crate::config::Config;
use anyhow::Error as AnyhowError;
use dtos::*;
use libangelshark::{AcmRunner, Message, ParallelIterator};
use log::debug;
use std::convert::Infallible;
use warp::{
    body,
    hyper::{header, StatusCode},
    path, post,
    reply::{self, with},
    Filter, Rejection, Reply,
};

mod dtos;
#[cfg(feature = "extensions")]
pub mod extensions;

/// GET / -> Name and version # of app.
pub fn index() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path::end().and_then(handle_version)
}

/// POST /ossi with JSON inputs -> JSON outputs
pub fn ossi(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let runner = config.runner.clone();
    path("ossi")
        .and(post())
        .and(warp::query::<Query>())
        .and(json_body())
        .and(with_runner(runner))
        .and_then(handle_ossi)
        .with(with::header(header::PRAGMA, "no-cache"))
        .with(with::header(header::CACHE_CONTROL, "no-store, max-age=0"))
        .with(with::header(header::X_FRAME_OPTIONS, "DENY"))
}

/// For passing runner to handlers.
fn with_runner(
    runner: AcmRunner,
) -> impl Filter<Extract = (AcmRunner,), Error = Infallible> + Clone {
    warp::any().map(move || runner.clone())
}

/// JSON request body filter with content length limit.
fn json_body() -> impl Filter<Extract = (Vec<Request>,), Error = Rejection> + Clone {
    body::content_length_limit(1024 * 16).and(body::json())
}

/// Handle version requests.
async fn handle_version() -> Result<impl Reply, Infallible> {
    Ok(reply::json(&Version {
        daemon_version: env!("CARGO_PKG_VERSION"),
    }))
}

/// Handle OSSI requests.
async fn handle_ossi(
    query: Query,
    requests: Vec<Request>,
    mut runner: AcmRunner,
) -> Result<impl Reply, Infallible> {
    debug!("{:?}", query);
    debug!("{:?}", requests);

    // Queue request inputs on runner.
    for (job_name, input) in requests
        .into_iter()
        .map(|r| -> Vec<(String, Message)> { r.into() })
        .flatten()
    {
        runner.queue_input(&job_name, &input);
    }

    // Collect runner results and convert to responses.
    let responses: Vec<Result<Vec<Response>, AnyhowError>> = if query.no_cache.unwrap_or_default() {
        // Run without cache.
        runner
            .run()
            .map(|(name, output)| {
                let output: Vec<Response> = output?
                    .into_iter()
                    .filter_map(move |o| {
                        if o.command == "logoff" {
                            None
                        } else {
                            Some(Response::from((name.clone(), o)))
                        }
                    })
                    .collect();
                Ok(output)
            })
            .collect()
    } else {
        // Run with cache.
        runner
            .run_cached()
            .map(|(name, output)| {
                let output: Vec<Response> = output?
                    .into_iter()
                    .filter_map(move |o| {
                        if o.command == "logoff" {
                            None
                        } else {
                            Some(Response::from((name.clone(), o)))
                        }
                    })
                    .collect();
                Ok(output)
            })
            .collect()
    };

    // Handle errors from runner.
    if query.panicky.unwrap_or_default() {
        // Return an internal error if anything went wrong.
        let responses: Result<Vec<Vec<Response>>, _> = responses.into_iter().collect();
        match responses {
            Err(e) => Ok(reply::with_status(
                reply::json(&Error {
                    reason: e.to_string(),
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
            Ok(r) => Ok(reply::with_status(
                reply::json(&r.into_iter().flatten().collect::<Vec<Response>>()),
                StatusCode::OK,
            )),
        }
    } else {
        // Discard errors and return just good data.
        let responses: Vec<Response> = responses
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();
        Ok(reply::with_status(reply::json(&responses), StatusCode::OK))
    }
}
