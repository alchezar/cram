//! cram - local server for the English trainers.

mod config;
mod db;
mod error;
mod quiz;
mod render;
mod route;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::{HeaderValue, header};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::quiz::Quizzes;

pub use crate::error::Error;

/// Shared application state passed to every handler.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    /// All quizzes, loaded once at startup.
    pub(crate) quizzes: Arc<Quizzes>,
    /// Progress database pool.
    pub(crate) db: SqlitePool,
}

/// Load config and quizzes, open the database, and serve requests until the
/// `shutdown` future resolves.
///
/// # Errors
/// Returns an error if config, quiz loading, the database, or binding fails.
pub async fn main<F>(shutdown: F) -> Result<(), Error>
where
    F: Future<Output = ()> + Send + 'static,
{
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::load("cram")?;

    let quizzes = Arc::new(Quizzes::load(&config.quizzes_dir)?);
    tracing::info!("loaded {} quizzes", quizzes.iter().count());

    let db = db::connect(&config.database_url).await?;
    tracing::info!("progress database ready at {}", config.database_url);
    let state = AppState { quizzes, db };

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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}
