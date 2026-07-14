//! HTML rendering with maud.

use std::collections::{BTreeMap, HashMap};

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    progress::{HARD, SOFT},
    quiz::{Kind, Question, Quiz, Quizzes},
};

/// A quiz on the index with its mastery percentage (0..=100).
struct Topic<'a> {
    id: &'a str,
    title: &'a str,
    percent: u8,
}

/// Index page: all quizzes grouped by section, each with a mastery bar.
pub fn index_page(quizzes: &Quizzes, totals: &HashMap<String, i64>) -> Markup {
    let mut sections = BTreeMap::<&str, Vec<Topic<'_>>>::new();
    for (id, quiz) in quizzes.iter() {
        let count = i64::try_from(quiz.questions.len()).unwrap_or(0);
        let capped = totals.get(id.as_str()).copied().unwrap_or(0);
        // Percentage of full mastery: capped streaks over count * HARD.
        let percent = if count > 0 {
            u8::try_from(capped * 100 / (count * HARD)).unwrap_or(100)
        } else {
            0
        };
        sections
            .entry(quiz.section.as_str())
            .or_default()
            .push(Topic {
                id: id.as_str(),
                title: quiz.title.as_str(),
                percent,
            });
    }
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "cram" }
                link rel="stylesheet" href="/style.css";
            }
            body {
                main .index {
                    h1 { "cram" }
                    @for (name, topics) in &sections {
                        section .group {
                            h2 { (name) }
                            ul .topics {
                                @for topic in topics {
                                    li { (topic_link(topic)) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One index entry: a card linking to the quiz with a mastery progress bar.
fn topic_link(topic: &Topic<'_>) -> Markup {
    html! {
        a href=(format!("/quiz/{}", topic.id)) {
            span .topic-head {
                span .topic-name { (topic.title) }
                span .topic-pct { (topic.percent) "%" }
            }
            span .bar { span .bar-fill style=(format!("width:{}%", topic.percent)) {} }
        }
    }
}

/// A visible question on a quiz page with its optional group heading and streak.
struct Row<'a> {
    /// Heading to show before this question when it starts a new section.
    heading: Option<&'a str>,
    /// The question to render.
    question: &'a Question,
    /// Current mastery streak.
    streak: i64,
}

/// Full HTML page for one quiz. Mastered questions (streak >= `HARD`) are hidden.
pub fn quiz_page(id: &str, quiz: &Quiz, streaks: &HashMap<u32, i64>) -> Markup {
    // Visible questions with an optional group heading. The section is carried
    // forward so its heading still shows even if its first question is mastered.
    let mut rows = Vec::<Row<'_>>::new();
    let mut section = "";
    let mut shown = "";
    for q in &quiz.questions {
        if !q.group.is_empty() {
            section = q.group.as_str();
        }
        let streak = streaks.get(&q.id).copied().unwrap_or(0);
        if streak >= HARD {
            continue;
        }
        let heading = if !section.is_empty() && section != shown {
            shown = section;
            Some(section)
        } else {
            None
        };
        rows.push(Row {
            heading,
            question: q,
            streak,
        });
    }

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
                    @if !quiz.rules.is_empty() {
                        div .rules { (PreEscaped(&quiz.rules)) }
                    }
                    @if rows.is_empty() {
                        p .done { "All questions mastered." }
                    } @else {
                        @for row in &rows {
                            @if let Some(name) = row.heading {
                                h2 .group-head { (name) }
                            }
                            (question(id, quiz.kind, row.question, row.streak))
                        }
                    }
                }
            }
        }
    }
}

/// One question block: prompt, mastery stars, answer inputs, and an htmx form
/// that posts to the check endpoint and swaps the result into `.result`.
fn question(id: &str, kind: Kind, q: &Question, streak: i64) -> Markup {
    let action = format!("/quiz/{id}/check/{}", q.id);
    html! {
        section .question data-qid=(q.id) {
            p .prompt { (q.prompt) " " (stars(streak)) }
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
pub fn result(kind: Kind, q: &Question, correct: bool, streak: i64) -> Markup {
    html! {
        div .verdict .ok[correct] .bad[!correct] {
            p .head {
                @if correct { "Correct!" } @else { "Not quite." }
                " " (stars(streak))
            }
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
            @if streak >= HARD {
                p .mastered { "Mastered - it leaves the quiz on reload." }
            }
        }
    }
}

/// Mastery stars for a streak: filled up to `HARD`, with a divider at `SOFT`.
fn stars(streak: i64) -> Markup {
    let filled = streak.min(HARD);
    html! {
        span .streak {
            @for k in 0..HARD {
                @if k < filled { span .on { "★" } } @else { span .off { "★" } }
                @if k == SOFT - 1 { span .div { "·" } }
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
