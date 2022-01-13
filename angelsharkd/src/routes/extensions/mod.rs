use crate::config::Config;
use warp::{path, Filter, Rejection, Reply};

#[cfg(feature = "simple_busy")]
mod simple_busy;
#[cfg(feature = "simple_deprov")]
mod simple_deprov;
#[cfg(feature = "simple_search")]
mod simple_search;

/// The extension filter; consists of all compiled optional Angelshark extension
/// filters combined under `/extensions`.
pub fn filter(config: &Config) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Note: this next line deals with the common edge case of having no
    // extensions loaded with feature flags. It ensures that the the type
    // checking is right when the return `.and()` is called below.
    let filters = default().or(default());

    // Block to enable simple_search extension feature. Instantiates a
    // searchable haystack and configures filters to handle search requests.
    #[cfg(feature = "simple_search")]
    let haystack = simple_search::Haystack::new(config.runner.clone());
    #[cfg(feature = "simple_search")]
    let filters = filters
        .or(simple_search::search_filter(haystack.clone()))
        .or(simple_search::refresh_filter(haystack));

    #[cfg(feature = "simple_deprov")]
    let filters = filters.or(simple_deprov::filter(config.runner.clone()));

    #[cfg(feature = "simple_busy")]
    let filters = filters
        .or(simple_busy::busy_filter(config.runner.to_owned()))
        .or(simple_busy::release_filter(config.runner.to_owned()))
        .or(simple_busy::toggle_filter(config.runner.to_owned()));

    path("extensions").and(filters)
}

/// The default, informational extension route.
fn default() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().map(|| "Angelshark extension route index. Enable extensions with feature switches and access them at `/extensions/<feature>`.")
}
