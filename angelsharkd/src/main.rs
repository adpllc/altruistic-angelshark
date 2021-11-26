use crate::config::Config;
use anyhow::{Context, Result};
use log::{debug, error, info, LevelFilter};
use tokio::{signal, task};
use warp::{hyper::Method, Filter};

mod config;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    // Init config.
    let config = Config::init().with_context(|| "Failed to initialize config.")?;

    // Init logging.
    env_logger::Builder::new()
        .filter(
            None,
            if config.debug_mode {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
        )
        .init();

    if config.debug_mode {
        debug!("**** DEBUGGING MODE ENABLED ****");
    }

    let routes = routes::index()
        .or(routes::ossi(&config))
        .with(if config.debug_mode || config.origin == "*" {
            warp::cors()
                .allow_any_origin()
                .allow_methods(&[Method::GET, Method::POST])
        } else {
            warp::cors()
                .allow_origin(config.origin.as_str())
                .allow_methods(&[Method::GET, Method::POST])
        })
        .with(warp::log("angelsharkd"));

    #[cfg(feature = "extensions")]
    let routes = routes.or(routes::extensions::filter(&config));

    // Create server with shutdown signal.
    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(config.bind_addr, async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler.");
    });

    // Run server to completion.
    info!("Starting server on {} ...", addr);
    if let Err(e) = task::spawn(server).await {
        error!("Server died unexpectedly: {}", e.to_string());
    }
    info!("Stopping server...");

    Ok(())
}
