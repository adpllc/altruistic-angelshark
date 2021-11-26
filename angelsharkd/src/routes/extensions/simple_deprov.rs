use warp::{Filter, Rejection, Reply};

pub fn filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("deprov").map(|| -> &str { todo!() }) // TODO:
}
