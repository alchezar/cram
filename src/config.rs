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
