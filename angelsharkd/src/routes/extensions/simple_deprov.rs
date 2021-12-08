use warp::{
    body::{content_length_limit, json},
    post, Filter, Rejection, Reply,
};

pub fn filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("deprov")
        .and(post())
        .and(content_length_limit(1024 * 16))
        .and(json())
        .map(|_: String| -> &str { todo!() }) // TODO:
}
