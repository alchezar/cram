//! Progress database: connection pool, migrations, and the default user.

pub mod progress;

use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;
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
/// With `url = None`, the database lives at the platform default (see
/// [`defaultldb_path`]).
///
/// # Errors
/// Returns an error if the directory, connection, or migrations fail.
pub async fn connect(url: Option<&str>) -> Result<SqlitePool, Error> {
    let opts = match url {
        Some(url) => SqliteConnectOptions::from_str(url)?,
        None => SqliteConnectOptions::new().filename(default_db_path()?),
    }
    .create_if_missing(true);
    if let Some(parent) = opts.get_filename().parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let path = opts.get_filename().display().to_string();
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    ensure_local_user(&pool).await?;
    tracing::info!("progress database ready at {path}");
    Ok(pool)
}

/// Default database location under the platform data dir, e.g. on macOS
/// `~/Library/Application Support/cram/cram.db`.
fn default_db_path() -> Result<PathBuf, Error> {
    let dir = dirs::data_dir()
        .ok_or_else(|| IoError::other("could not find a platform data directory"))?;
    Ok(dir.join("cram").join("cram.db"))
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
