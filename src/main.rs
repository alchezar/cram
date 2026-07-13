//! cram - local server for the English trainers.

mod config;
mod error;
mod quiz;
mod render;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing,
};
use axum_extra::extract::Form;
use maud::Markup;
use serde::Deserialize;
use tokio::{net::TcpListener, signal};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::{config::Config, error::Error, quiz::Quizzes};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::load("cram")?;

    let quizzes = Arc::new(Quizzes::load(&config.quizzes_dir)?);
    tracing::info!("loaded {} quizzes", quizzes.iter().count());

    // Disable caching so browsers always fetch the latest pages.
    let no_store = SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, must-revalidate"),
    );

    let app = Router::new()
        .route("/", routing::get(index))
        .route("/quiz/{id}", routing::get(quiz_page))
        .route("/quiz/{id}/check/{qid}", routing::post(check))
        .fallback_service(ServeDir::new(&config.web_dir))
        .layer(no_store)
        .layer(TraceLayer::new_for_http())
        .with_state(quizzes);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("cram listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// `GET /`
//
/// Render the index page listing every quiz by section.
async fn index(State(quizzes): State<Arc<Quizzes>>) -> Markup {
    render::index_page(&quizzes)
}

// `GET /quiz/{id}`
//
/// Render one quiz page, or 404 if the id is unknown.
async fn quiz_page(State(quizzes): State<Arc<Quizzes>>, Path(id): Path<String>) -> Response {
    match quizzes.get(&id) {
        Some(quiz) => render::quiz_page(&id, quiz).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown quiz").into_response(),
    }
}

/// A submitted answer: selected option indices (multi/single) or free text.
#[derive(Debug, Deserialize)]
struct Answer {
    #[serde(default)]
    opt: Vec<usize>,
    #[serde(default)]
    answer: String,
}

// `POST /quiz/{id}/check/{qid}`
//
/// Check one answer and return an HTML result fragment for htmx to swap in.
async fn check(
    State(quizzes): State<Arc<Quizzes>>,
    Path((id, qid)): Path<(String, u32)>,
    Form(answer): Form<Answer>,
) -> Response {
    let Some(quiz) = quizzes.get(&id) else {
        return (StatusCode::NOT_FOUND, "unknown quiz").into_response();
    };
    let Some(question) = quiz.question(qid) else {
        return (StatusCode::NOT_FOUND, "unknown question").into_response();
    };
    let correct = question.is_correct(quiz.kind, &answer.opt, &answer.answer);
    render::result(quiz.kind, question, correct).into_response()
}

/// Wait for Ctrl-C to trigger a clean shutdown.
async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("shutting down");
}
