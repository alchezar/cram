//! HTML rendering with maud.

use maud::{DOCTYPE, Markup, html};

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
            }
            body {
                main .quiz data-quiz=(id) {
                    a .back href="/" { "<- all topics" }
                    h1 { (quiz.title) }
                    @if !quiz.intro.is_empty() {
                        p .intro { (quiz.intro) }
                    }
                    @for q in &quiz.questions {
                        (question(quiz.kind, q))
                    }
                }
            }
        }
    }
}

/// One question block with answer inputs for the given `kind`.
fn question(kind: Kind, q: &Question) -> Markup {
    html! {
        section .question data-qid=(q.id) {
            p .prompt { (q.prompt) }
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
        }
    }
}
