//! Progress database: connection pool, migrations, and the default user.

use std::fs;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::error::Error;

/// Single-user id used until real login lands (phase 3).
pub const LOCAL_USER: i64 = 1;

/// Open the pool, create the file/dir if missing, migrate, ensure the local user.
///
/// # Errors
/// Returns an error if the directory, connection, or migrations fail.
pub async fn connect(url: &str) -> Result<SqlitePool, Error> {
    let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    if let Some(parent) = opts.get_filename().parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    ensure_local_user(&pool).await?;
    Ok(pool)
}

/// Insert the default `local` user if it is not there yet.
async fn ensure_local_user(pool: &SqlitePool) -> Result<(), Error> {
    let ts = now();
    sqlx::query!(
        r"
            INSERT OR IGNORE INTO users (id, name, created_at)
            VALUES ($1, 'local', $2)
        ",
        LOCAL_USER,
        ts
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Current time as Unix epoch seconds.
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}
