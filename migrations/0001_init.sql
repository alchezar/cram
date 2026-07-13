-- Progress schema. Kept portable (no engine-specific types); timestamps are
-- Unix epoch seconds written from Rust, not datetime('now').

CREATE TABLE users (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL UNIQUE,
    created_at INTEGER NOT NULL
);

-- One row per (user, quiz, question); tracks the mastery streak.
CREATE TABLE mastery (
    user_id     INTEGER NOT NULL,
    quiz_id     TEXT    NOT NULL,
    question_id INTEGER NOT NULL,
    streak      INTEGER NOT NULL DEFAULT 0,
    seen        INTEGER NOT NULL DEFAULT 0,
    correct     INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, quiz_id, question_id)
);
