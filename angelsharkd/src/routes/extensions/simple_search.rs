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

pub fn refresh(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let runner = config.runner.clone();
    path!("search" / "refresh")
        .and(get())
        .and(with_runner(runner))
        .and_then(handle_refresh)
}

async fn handle_refresh(runner: AcmRunner) -> Result<impl Reply, Infallible> {
    get_extensions_cached(true);
    Ok("Refresh scheduled")
}

async fn handle_simple_search(terms: Terms, runner: AcmRunner) -> Result<impl Reply, Infallible> {
    Ok(get_extensions_cached(false))
}

#[cached]
fn get_extensions_cached(refresh: bool) -> String {
    if refresh {
        tokio::spawn(async move {
            std::thread::sleep_ms(10000);

            if let Ok(mut handle) = GET_EXTENSIONS_CACHED.lock() {
                handle.cache_clear();
                handle.cache_set(false, String::from("fresh"));
                eprintln!("evicted");
            }
        });
    }

    String::from("stale")
}

fn get_extensions(runner: AcmRunner) {
    todo!()
}
