use crate::{config::Config, routes::with_runner};
use libangelshark::AcmRunner;
use once_cell::sync::Lazy;
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};
use warp::{
    body::{content_length_limit, json},
    get, path, post, Filter, Rejection, Reply,
};

static HAYSTACK_CACHE: Lazy<Haystack> = Lazy::new(Haystack::new);

/// Collection of search terms
type Needle = Vec<String>;

pub fn search(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let runner = config.runner.clone();
    // TODO: anti-caching headers on resp?

    path("search")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json::<Needle>())
        .and(with_runner(runner))
        .and_then(handle_search)
}

pub fn refresh(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let runner = config.runner.clone();
    path!("search" / "refresh")
        .and(get())
        .and(with_runner(runner))
        .and_then(handle_refresh)
}

async fn handle_search(terms: Needle, runner: AcmRunner) -> Result<impl Reply, Infallible> {
    HAYSTACK_CACHE.search(Vec::new());
    Ok("")
}

async fn handle_refresh(runner: AcmRunner) -> Result<impl Reply, Infallible> {
    HAYSTACK_CACHE.refresh();
    Ok("Refresh scheduled")
}

/// A lazy-loaded, asynchronously-refreshed exension-type haystack cache.
struct Haystack {
    inner: Arc<Mutex<String>>,
}

impl Haystack {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(String::with_capacity(0))),
        }
    }

    pub fn search(&self, needle: Needle) -> String {
        (*self.inner.lock().unwrap()).clone()
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
