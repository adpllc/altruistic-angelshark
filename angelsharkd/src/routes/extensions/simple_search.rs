use libangelshark::AcmRunner;
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};
use warp::{
    body::{content_length_limit, json},
    get, path, post, Filter, Rejection, Reply,
};

/// Collection of search terms
type Needle = Vec<String>;

pub fn search(haystack: Haystack) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // TODO: discourage caching response thru headers

    path("search")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .and_then(move |terms: Needle| handle_search(haystack.clone(), terms))
}

pub fn refresh(
    runner: AcmRunner,
    haystack: Haystack,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("search" / "refresh")
        .and(get())
        .and_then(move || handle_refresh(haystack.clone(), runner.clone()))
}

async fn handle_search(haystack: Haystack, needle: Needle) -> Result<impl Reply, Infallible> {
    Ok(haystack.search(Vec::new()))
}

async fn handle_refresh(haystack: Haystack, runner: AcmRunner) -> Result<impl Reply, Infallible> {
    haystack.refresh();
    Ok("Refresh scheduled")
}

/// A lazy-loaded, asynchronously-refreshed exension-type haystack cache.
#[derive(Clone)]
pub struct Haystack {
    inner: Arc<Mutex<String>>,
}

impl Haystack {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(String::with_capacity(0))),
        }
    }

    pub fn search(&self, needle: Needle) -> String {
        if let Ok(matches) = self.inner.lock() {
            matches.clone()
        } else {
            String::from("stale")
        }
    }

    pub fn refresh(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            std::thread::sleep_ms(10000); // slow generation here

            if let Ok(mut handle) = inner.lock() {
                *handle = String::from("fresh");
                eprintln!("evicted");
            }
        });
    }
}
