use log::{error, info};
use std::convert::Infallible;
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
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .and_then(move |terms: Needles| search(haystack.to_owned(), terms))
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
async fn search(haystack: Haystack, needles: Needles) -> Result<impl Reply, Infallible> {
    // Ok(haystack.search(Vec::new())?)
    // if let Ok(matches = haystack.search(needle);
    match haystack.search(needles) {
        Ok(matches) => Ok(reply::with_status(reply::json(&matches), StatusCode::OK)),
        Err(e) => Ok(reply::with_status(
            reply::json(&e.to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Immediately returns. Spawns an asynchronous task to complete the haystack
/// refresh in the background.
async fn refresh(haystack: Haystack) -> Result<impl Reply, Infallible> {
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
