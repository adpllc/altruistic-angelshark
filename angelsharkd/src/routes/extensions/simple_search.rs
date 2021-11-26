use crate::config::Config;
use std::convert::Infallible;
use warp::{path, post, Filter, Rejection, Reply};

const CMD_LIST_EXT: &str = "list extension-type";
const CMD_LIST_STAT: &str = "list station";

pub fn filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("search").and(post()).and_then(handle_simple_search)
}

async fn handle_simple_search() -> Result<impl Reply, Infallible> {
    Ok("Search!")
}
