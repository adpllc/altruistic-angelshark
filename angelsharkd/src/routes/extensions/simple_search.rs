use crate::{config::Config, routes::with_runner};
use cached::proc_macro::cached;
use libangelshark::AcmRunner;
use std::convert::Infallible;
use warp::{
    body::{content_length_limit, json},
    get, path, post, Filter, Rejection, Reply,
};

const CMD_LIST_EXT: &str = "list extension-type";
const CMD_LIST_STAT: &str = "list station";

type Terms = Vec<String>;

pub fn search(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let runner = config.runner.clone();
    // TODO: anti-caching headers on resp?

    path("search")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json::<Terms>())
        .and(with_runner(runner))
        .and_then(handle_simple_search)
}

pub fn refresh() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("search" / "refresh")
        .and(get())
        .and_then(handle_refresh)
}

async fn handle_refresh() -> Result<impl Reply, Infallible> {
    fetch(String::from("test"), true);
    Ok("")
}

async fn handle_simple_search(terms: Terms, runner: AcmRunner) -> Result<impl Reply, Infallible> {
    Ok(fetch(terms[0].clone(), false))
}

#[cached]
fn fetch(param: String, refresh: bool) -> String {
    if refresh {
        let param = param.clone();
        tokio::spawn(async move {
            std::thread::sleep_ms(10000);
            let mut handle = FETCH.lock().unwrap();
            handle.cache_clear();
            handle.cache_set((param.clone(), false), format!("blargh {}", param.clone()));
            eprintln!("evicted");
        });
    }

    param
}
