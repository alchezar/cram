//! cram - local server for the English trainers.

mod config;
mod error;

use std::net::SocketAddr;

use axum::{
    Router,
    http::{HeaderValue, header},
};
use tokio::{net::TcpListener, signal};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::{config::Config, error::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::load("cram")?;

    // Disable caching so browsers always fetch the latest pages.
    let no_store = SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, must-revalidate"),
    );

    let app = Router::new()
        .fallback_service(ServeDir::new(&config.web_dir))
        .layer(no_store)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("cram listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Wait for Ctrl-C to trigger a clean shutdown.
async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("shutting down");
}
