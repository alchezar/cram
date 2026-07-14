//! HTML rendering with maud.

use std::collections::HashMap;

use maud::{DOCTYPE, Markup, PreEscaped, html};
use rand::seq::SliceRandom;

use crate::{
    db::progress::{HARD, SOFT},
    models::{
        quiz::{Kind, Question, Quiz, Quizzes},
        roadmap::{Roadmap, Topic},
    },
};

/// Index page: the roadmap of B2 topics with overall and per-topic progress.
pub fn index_page(roadmap: &Roadmap, quizzes: &Quizzes, totals: &HashMap<String, i64>) -> Markup {
    let total = roadmap.topics().count();
    // Live = topics backed by a quiz that actually exists.
    let live = roadmap
        .topics()
        .filter(|t| {
            t.quiz
                .as_deref()
                .is_some_and(|id| quizzes.get(id).is_some())
        })
        .count();
    let overall = live
        .checked_mul(100)
        .and_then(|n| n.checked_div(total))
        .unwrap_or(0);
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (roadmap.title) }
                link rel="stylesheet" href="/style.css";
            }
            body {
                main .index {
                    h1 { (roadmap.title) }
                    p .lead { (roadmap.lead) }
                    div .prog {
                        "Ready: " b { (live) } " / " b { (total) } " topics · work through them one by one."
                    }
                    div .mbar { div .mbar-fill style=(format!("width:{overall}%")) {} }
                    @for section in &roadmap.sections {
                        h2 .sec { (section.name) }
                        div .grid {
                            @for topic in &section.topics {
                                (topic_card(topic, quizzes, totals))
                            }
                        }
                    }
                    footer { (roadmap.footer) }
                }
            }
        }
    }
}

/// One roadmap card: a live topic links to its quiz and shows mastery; a
/// planned topic is a dimmed, non-clickable tile.
fn topic_card(topic: &Topic, quizzes: &Quizzes, totals: &HashMap<String, i64>) -> Markup {
    let live = topic
        .quiz
        .as_deref()
        .and_then(|id| quizzes.get(id).map(|quiz| (id, quiz)));
    if let Some((id, quiz)) = live {
        let count = i64::try_from(quiz.questions.len()).unwrap_or(0);
        let capped = totals.get(id).copied().unwrap_or(0);
        // Weighted mastery: capped streaks over count * HARD.
        let pct = (capped * 100)
            .checked_div(count * HARD)
            .and_then(|p| u8::try_from(p).ok())
            .unwrap_or(0);
        html! {
            a .card .done href=(format!("/quiz/{id}")) {
                div .top {
                    span .num { (topic.n) }
                    span .badges {
                        span .badge .prog .zero[pct == 0] .full[pct >= 100] {
                            @if pct == 0 { "not started" }
                            @else if pct >= 100 { "100% ✓" }
                            @else { (pct) "% done" }
                        }
                    }
                }
                div .title { (topic.title) }
                div .desc { (topic.desc) }
                div .cardbar { div .cardbar-fill style=(format!("width:{pct}%")) {} }
            }
        }
    } else {
        html! {
            div .card .todo {
                div .top {
                    span .num { (topic.n) }
                    span .badges { span .badge .soon { "Planned" } }
                }
                div .title { (topic.title) }
                div .desc { (topic.desc) }
            }
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
    // Segment questions into groups (a non-empty group name starts a new one),
    // shuffle within each group, then show the heading before its first visible
    // question. Mastered questions (streak >= HARD) are dropped.
    let mut groups = Vec::<(&str, Vec<&Question>)>::new();
    for q in &quiz.questions {
        if groups.is_empty() || !q.group.is_empty() {
            groups.push((q.group.as_str(), Vec::new()));
        }
        if let Some(group) = groups.last_mut() {
            group.1.push(q);
        }
    }

    let mut rng = rand::rng();
    let mut rows = Vec::<Row<'_>>::new();
    for (name, questions) in &mut groups {
        questions.shuffle(&mut rng);
        let mut heading = (!name.is_empty()).then_some(*name);
        for &q in questions.iter() {
            let streak = streaks.get(&q.id).copied().unwrap_or(0);
            if streak >= HARD {
                continue;
            }
            rows.push(Row {
                heading: heading.take(),
                question: q,
                streak,
            });
        }
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
                    a .back href="/" { "← all topics" }
                    h1 { (quiz.title) }
                    @if !quiz.intro.is_empty() {
                        p .intro { (quiz.intro) }
                    }
                    @if !quiz.rules.is_empty() {
                        div .rules { (PreEscaped(&quiz.rules)) }
                    }
                    @if rows.is_empty() {
                        p .finished { "All questions mastered." }
                    } @else {
                        @for row in &rows {
                            @if let Some(name) = row.heading {
                                h2 .group-head { (name) }
                            }
                            (question(id, quiz.kind, row.question, row.streak))
                        }
                    }
                    div .quiz-actions {
                        button .reset
                            hx-post=(format!("/quiz/{id}/reset"))
                            hx-confirm="Reset all progress for this topic?"
                            hx-swap="none" { "Reset progress" }
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
    // Shuffle the display order; each option's `value` keeps its original index.
    let mut order = (0..q.options.len()).collect::<Vec<_>>();
    order.shuffle(&mut rand::rng());
    html! {
        section .question data-qid=(q.id) {
            p .prompt { (q.prompt) " " span id=(format!("stars-{}", q.id)) { (stars(streak)) } }
            form .answer-form hx-post=(action) hx-target="find .result" hx-swap="innerHTML" {
                div .options {
                    @match kind {
                        Kind::Multi | Kind::Single => {
                            @let ty = if matches!(kind, Kind::Multi) { "checkbox" } else { "radio" };
                            @for &i in &order {
                                @let o = &q.options[i];
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
        // Out-of-band swap: refresh the stars shown next to the prompt too.
        span id=(format!("stars-{}", q.id)) hx-swap-oob="true" { (stars(streak)) }
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
