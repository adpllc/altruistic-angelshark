use crate::config::Config;
use warp::{path, Filter, Rejection, Reply};

#[cfg(feature = "simple_deprov")]
mod simple_deprov;
#[cfg(feature = "simple_search")]
mod simple_search;

pub fn filter(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Note: this next line deals with the common edge case of having no
    // extensions loaded with feature flags. It ensures that the the type
    // checking is right when the return `.and()` is called below.
    let filters = default().or(default());

    #[cfg(feature = "simple_search")]
    let filters = filters
        .or(simple_search::search(config))
        .or(simple_search::refresh());

    #[cfg(feature = "simple_deprov")]
    let filters = filters.or(simple_deprov::filter());

    path("extensions").and(filters)
}

fn default() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().map(|| "Angelshark extension route index. Enable extensions with feature switches and access them at `/extensions/<feature>`.")
}
