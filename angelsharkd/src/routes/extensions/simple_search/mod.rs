use log::{error, info};
use std::{convert::Infallible, thread};
pub use types::Haystack;
use types::*;
use warp::{
    body::{content_length_limit, json},
    get,
    hyper::{header, StatusCode},
    path, post,
    reply::{self, with},
    Filter, Rejection, Reply,
};

mod types;

/// Returns a warp filter to handle HTTP POSTs for searching the haystack.
pub fn search_filter(
    haystack: Haystack,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("search")
        .and(warp::query::<Query>())
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .and_then(move |query: Query, needles: Needles| search(haystack.to_owned(), needles, query))
        .with(with::header(header::PRAGMA, "no-cache"))
        .with(with::header(header::CACHE_CONTROL, "no-store, max-age=0"))
        .with(with::header(header::X_FRAME_OPTIONS, "DENY"))
}

/// Returns a warp filter to handle HTTP GETs for refreshing the haystack.
pub fn refresh_filter(
    haystack: Haystack,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("search" / "refresh")
        .and(get())
        .and_then(move || refresh(haystack.to_owned()))
}

/// Runs the search request to find all needles in the haystack and converts the
/// results into a reply.
async fn search(
    haystack: Haystack,
    needles: Needles,
    query: Query,
) -> Result<impl Reply, Infallible> {
    match haystack.search(needles) {
        Ok(mut matches) => {
            if let Some(limit) = query.limit {
                matches = matches.into_iter().take(limit).collect();
            }
            Ok(reply::with_status(reply::json(&matches), StatusCode::OK))
        }
        Err(e) => Ok(reply::with_status(
            reply::json(&e.to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Immediately returns. Spawns an asynchronous task to complete the haystack
/// refresh in the background.
async fn refresh(haystack: Haystack) -> Result<impl Reply, Infallible> {
    // Note: why use `thread::spawn()`? Previously I had used `tokio::spawn()`
    // here. The difference is that `tokio::spawn()` spawns the task in a thread
    // from Tokio's dedicated threadpool. This means that this thread is no
    // longer available to answer requests from the HTTP server. This
    // effectively meant that during a refresh, large blocking portions of code
    // were preventing people from running searches, and they would receive 504
    // Gateway Timeouts from the reverse proxy. Using a plain `thread::spawn()`,
    // we ensure that a thread outside of Tokio's or Rayon's threadpools.
    // Therefore, searches can still happen during even the most intensive of
    // refreshes.

    // Run refresh as a background task and immediately return.
    thread::spawn(move || {
        if let Err(e) = haystack.refresh() {
            error!("{}", e.to_string());
        } else {
            info!("Search haystack refreshed.");
        }
    });

    Ok("Refresh scheduled")
}
