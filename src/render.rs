//! HTML rendering with maud.

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::quiz::{Kind, Question, Quiz};

/// Full HTML page for one quiz.
pub fn quiz_page(id: &str, quiz: &Quiz) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (quiz.title) " - cram" }
                link rel="stylesheet" href="/style.css";
                script src="/htmx.min.js" defer {}
            }
            body {
                main .quiz data-quiz=(id) {
                    a .back href="/" { "<- all topics" }
                    h1 { (quiz.title) }
                    @if !quiz.intro.is_empty() {
                        p .intro { (quiz.intro) }
                    }
                    @for q in &quiz.questions {
                        (question(id, quiz.kind, q))
                    }
                }
            }
        }
    }
}

/// One question block: prompt, answer inputs, and an htmx form that posts to
/// the check endpoint and swaps the result into `.result`.
fn question(id: &str, kind: Kind, q: &Question) -> Markup {
    let action = format!("/quiz/{id}/check/{}", q.id);
    html! {
        section .question data-qid=(q.id) {
            p .prompt { (q.prompt) }
            form .answer-form hx-post=(action) hx-target="find .result" hx-swap="innerHTML" {
                div .options {
                    @match kind {
                        Kind::Multi | Kind::Single => {
                            @let ty = if matches!(kind, Kind::Multi) { "checkbox" } else { "radio" };
                            @for (i, o) in q.options.iter().enumerate() {
                                label .option {
                                    input type=(ty) name="opt" value=(i);
                                    span { (o.text) }
                                }
                            }
                        }
                        Kind::Text => {
                            input .text-answer type="text" name="answer"
                                autocomplete="off" placeholder="type your answer";
                        }
                    }
                }
                button .check type="submit" { "Check" }
                div .result {}
            }
        }
    }
}

/// Result fragment swapped into a question's `.result` after checking.
pub fn result(kind: Kind, q: &Question, correct: bool) -> Markup {
    html! {
        div .verdict .ok[correct] .bad[!correct] {
            p .head { @if correct { "Correct!" } @else { "Not quite." } }
            p .solution {
                "Answer: "
                @match kind {
                    Kind::Text => (q.answers.join(" / ")),
                    _ => (correct_options(q)),
                }
            }
            @if !q.explain.is_empty() {
                p .explain { (PreEscaped(&q.explain)) }
            }
        }
    }
}

/// Comma-joined text of the correct options.
fn correct_options(q: &Question) -> String {
    q.options
        .iter()
        .filter(|o| o.correct)
        .map(|o| o.text.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
