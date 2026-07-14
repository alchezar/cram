//! HTTP routes and their handlers.

use std::collections::HashMap;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    routing,
};
use axum_extra::extract::Form;
use maud::Markup;
use serde::Deserialize;

use crate::{AppState, db::progress, render};

/// Build the application router with all routes and shared state.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(index))
        .route("/quiz/{id}", routing::get(quiz_page))
        .route("/quiz/{id}/check/{qid}", routing::post(check))
        .route("/quiz/{id}/reset", routing::post(reset))
        .with_state(state)
}

/// What a handler returns: a rendered page/fragment, an htmx redirect, or a 404.
enum AppResponse {
    Html(Markup),
    Redirect(String),
    NotFound(&'static str),
}

impl IntoResponse for AppResponse {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Html(markup) => markup.into_response(),
            // htmx does a full-page navigation when it sees this header.
            Self::Redirect(to) => [("HX-Redirect", to)].into_response(),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
        }
    }
}

// `GET /`
//
/// Render the index page listing every quiz by section.
async fn index(State(state): State<AppState>) -> AppResponse {
    let totals = progress::mastery_totals(&state.db)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("failed to load mastery totals: {e}");
            HashMap::new()
        });
    AppResponse::Html(render::index_page(&state.quizzes, &totals))
}

// `GET /quiz/{id}`
//
/// Render one quiz page, or 404 if the id is unknown.
async fn quiz_page(State(state): State<AppState>, Path(id): Path<String>) -> AppResponse {
    let Some(quiz) = state.quizzes.get(&id) else {
        return AppResponse::NotFound("unknown quiz");
    };
    let streaks = progress::streaks(&state.db, &id).await.unwrap_or_else(|e| {
        tracing::error!("failed to load streaks for {id}: {e}");
        HashMap::new()
    });
    AppResponse::Html(render::quiz_page(&id, quiz, &streaks))
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
    State(state): State<AppState>,
    Path((id, qid)): Path<(String, u32)>,
    Form(answer): Form<Answer>,
) -> AppResponse {
    let Some(quiz) = state.quizzes.get(&id) else {
        return AppResponse::NotFound("unknown quiz");
    };
    let Some(question) = quiz.question(qid) else {
        return AppResponse::NotFound("unknown question");
    };
    let correct = question.is_correct(quiz.kind, &answer.opt, &answer.answer);
    let streak = progress::record(&state.db, &id, qid, correct)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("failed to record progress for {id}/{qid}: {e}");
            i64::from(correct)
        });
    AppResponse::Html(render::result(quiz.kind, question, correct, streak))
}

// `POST /quiz/{id}/reset`
//
/// Wipe this quiz's progress, then send htmx to reload the fresh quiz page.
async fn reset(State(state): State<AppState>, Path(id): Path<String>) -> AppResponse {
    if state.quizzes.get(&id).is_none() {
        return AppResponse::NotFound("unknown quiz");
    }
    if let Err(e) = progress::reset(&state.db, &id).await {
        tracing::error!("failed to reset progress for {id}: {e}");
    }
    AppResponse::Redirect(format!("/quiz/{id}"))
}
