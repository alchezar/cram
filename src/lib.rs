//! cram - local server for the English trainers.

mod config;
mod db;
mod error;
mod models;
mod render;
mod route;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, header},
    serve::Serve,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::{
    config::Config,
    models::{quiz::Quizzes, roadmap::Roadmap},
};

pub use crate::error::Error;

/// Shared application state passed to every handler.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    /// All quizzes, loaded once at startup.
    pub(crate) quizzes: Arc<Quizzes>,
    /// Index roadmap, loaded once at startup.
    pub(crate) roadmap: Arc<Roadmap>,
    /// Progress database pool.
    pub(crate) db: SqlitePool,
}

/// A bound-but-not-yet-serving axum server for this app.
type Server = Serve<TcpListener, Router, Router>;

/// Load config and quizzes, open the database, and bind the listener.
/// Returns the ready-to-run server; the caller drives it (e.g. with graceful
/// shutdown) and can read its address via `Serve::local_addr`.
///
/// # Errors
/// Returns an error if config, quiz loading, the database, or binding fails.
pub async fn main() -> Result<Server, Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::load("cram")?;

    let quizzes = Arc::new(Quizzes::load(&config.quizzes_dir)?);
    tracing::info!("loaded {} quizzes", quizzes.iter().count());
    let roadmap = Arc::new(Roadmap::load(&config.roadmap_file)?);
    let db = db::connect(config.database_url.as_deref()).await?;

    let state = AppState {
        quizzes,
        roadmap,
        db,
    };

    // Disable caching so browsers always fetch the latest pages.
    let no_store = SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, must-revalidate"),
    );

    let app = route::router(state)
        .fallback_service(ServeDir::new(&config.web_dir))
        .layer(no_store)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("cram listening on http://{addr}");
    if let Some(ip) = local_ip() {
        tracing::info!("on your network (phone): http://{ip}:{}", config.port);
    }

    Ok(axum::serve(listener, app))
}

/// Best-effort LAN IP for reaching the server from another device on the same
/// network (e.g. a phone). Connecting a UDP socket reveals the outbound route's
/// local address without sending any packets.
fn local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect(("8.8.8.8", 80)).ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}
