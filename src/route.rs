//! HTTP routes and their handlers.

use std::sync::Arc;

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

use crate::{quiz::Quizzes, render};

/// Build the application router with all routes and shared state.
pub fn router(quizzes: Arc<Quizzes>) -> Router {
    Router::new()
        .route("/", routing::get(index))
        .route("/quiz/{id}", routing::get(quiz_page))
        .route("/quiz/{id}/check/{qid}", routing::post(check))
        .with_state(quizzes)
}

/// What a handler returns: a rendered HTML page/fragment or a 404.
enum AppResponse {
    Html(Markup),
    NotFound(&'static str),
}

impl IntoResponse for AppResponse {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Html(markup) => markup.into_response(),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
        }
    }
}

// `GET /`
//
/// Render the index page listing every quiz by section.
async fn index(State(quizzes): State<Arc<Quizzes>>) -> AppResponse {
    AppResponse::Html(render::index_page(&quizzes))
}

// `GET /quiz/{id}`
//
/// Render one quiz page, or 404 if the id is unknown.
async fn quiz_page(State(quizzes): State<Arc<Quizzes>>, Path(id): Path<String>) -> AppResponse {
    match quizzes.get(&id) {
        Some(quiz) => AppResponse::Html(render::quiz_page(&id, quiz)),
        None => AppResponse::NotFound("unknown quiz"),
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
) -> AppResponse {
    let Some(quiz) = quizzes.get(&id) else {
        return AppResponse::NotFound("unknown quiz");
    };
    let Some(question) = quiz.question(qid) else {
        return AppResponse::NotFound("unknown question");
    };
    let correct = question.is_correct(quiz.kind, &answer.opt, &answer.answer);
    AppResponse::Html(render::result(quiz.kind, question, correct))
}
