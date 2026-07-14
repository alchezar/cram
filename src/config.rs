//! Server configuration loaded from `cram.toml`.

use std::path::PathBuf;

use config::{Config as RawConfig, File};
use serde::Deserialize;

use crate::error::Error;

/// Server settings.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Port the server listens on.
    pub port: u16,
    /// Directory of static files to serve.
    pub web_dir: PathBuf,
    /// Directory of quiz TOML files.
    pub quizzes_dir: PathBuf,
    /// Path to the index roadmap TOML file.
    pub roadmap_file: PathBuf,
    /// sqlx connection URL for the progress database. When omitted, a default
    /// path under the platform data dir is used (see `db::connect`).
    #[serde(default)]
    pub database_url: Option<String>,
}

impl Config {
    /// Load and parse the config from the TOML file `path`.
    ///
    /// # Errors
    /// Returns [`Error::Config`] if the file is missing or does not match the schema.
    pub fn load(path: &str) -> Result<Self, Error> {
        let raw = RawConfig::builder()
            .add_source(File::with_name(path))
            .build()?;
        Ok(raw.try_deserialize()?)
    }
}
