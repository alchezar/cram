//! Mastery progress: thresholds and streak updates in the database.

use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::{
    db::{LOCAL_USER, now},
    error::Error,
};

/// Consecutive correct answers that master a question (it then drops out).
pub const HARD: i64 = 5;
/// Consecutive correct answers for a soft-mastery mark.
pub const SOFT: i64 = 3;

/// Record one answer: bump `seen`/`correct` and update the streak
/// (correct -> +1, wrong -> reset to 0). Creates the row on first sight.
/// Returns the question's new streak.
///
/// # Errors
/// Returns an error if the database write fails.
pub async fn record(
    db: &SqlitePool,
    quiz_id: &str,
    question_id: u32,
    correct: bool,
) -> Result<i64, Error> {
    let qid = i64::from(question_id);
    let flag = i64::from(correct);
    let ts = now();

    let mut tx = db.begin().await?;

    sqlx::query!(
        r"
            INSERT OR IGNORE INTO mastery
               (user_id, quiz_id, question_id, streak, seen, correct, updated_at)
            VALUES ($1, $2, $3, 0, 0, 0, $4)
        ",
        LOCAL_USER,
        quiz_id,
        qid,
        ts
    )
    .execute(tx.as_mut())
    .await?;

    // streak: correct -> (streak + 1) * 1; wrong -> (streak + 1) * 0 = 0.
    sqlx::query!(
        r"
            UPDATE mastery
              SET streak = (streak + 1) * $1,
                  seen = seen + 1,
                  correct = correct + $2,
                  updated_at = $3
            WHERE user_id = $4 AND quiz_id = $5 AND question_id = $6
        ",
        flag,
        flag,
        ts,
        LOCAL_USER,
        quiz_id,
        qid
    )
    .execute(tx.as_mut())
    .await?;

    let row = sqlx::query!(
        r"
            SELECT streak FROM mastery
            WHERE user_id = $1 AND quiz_id = $2 AND question_id = $3
        ",
        LOCAL_USER,
        quiz_id,
        qid
    )
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(row.streak)
}

/// Per-question streaks for a quiz: `question_id -> streak` (default user).
///
/// # Errors
/// Returns an error if the query fails.
pub async fn streaks(db: &SqlitePool, quiz_id: &str) -> Result<HashMap<u32, i64>, Error> {
    let rows = sqlx::query!(
        r"
            SELECT question_id, streak FROM mastery
            WHERE user_id = $1 AND quiz_id = $2
        ",
        LOCAL_USER,
        quiz_id
    )
    .fetch_all(db)
    .await?;

    let mut map = HashMap::with_capacity(rows.len());
    for row in rows {
        if let Ok(qid) = u32::try_from(row.question_id) {
            map.insert(qid, row.streak);
        }
    }
    Ok(map)
}

/// Capped mastery total per quiz: `sum(min(streak, HARD))` keyed by quiz id.
/// A quiz's percentage is this over `question_count * HARD`.
///
/// # Errors
/// Returns an error if the query fails.
pub async fn mastery_totals(db: &SqlitePool) -> Result<HashMap<String, i64>, Error> {
    let rows = sqlx::query!(
        r"
            SELECT quiz_id, streak FROM mastery
            WHERE user_id = $1
        ",
        LOCAL_USER
    )
    .fetch_all(db)
    .await?;

    let mut totals = HashMap::new();
    for row in rows {
        *totals.entry(row.quiz_id).or_insert(0) += row.streak.min(HARD);
    }
    Ok(totals)
}
