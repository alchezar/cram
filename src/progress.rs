//! Mastery progress: thresholds and streak updates in the database.

use sqlx::SqlitePool;

use crate::db::{LOCAL_USER, now};
use crate::error::Error;

/// Record one answer: bump `seen`/`correct` and update the streak
/// (correct -> +1, wrong -> reset to 0). Creates the row on first sight.
///
/// # Errors
/// Returns an error if the database write fails.
pub async fn record(
    db: &SqlitePool,
    quiz_id: &str,
    question_id: u32,
    correct: bool,
) -> Result<(), Error> {
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
    .execute(&mut *tx)
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
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
