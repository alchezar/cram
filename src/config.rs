//! Server configuration loaded from `cram.toml`.

use std::env;
use std::path::{Path, PathBuf};

use config::{Config as RawConfig, File};
use serde::Deserialize;

use crate::error::Error;

/// Config filename, resolved relative to the working directory.
const CONFIG_FILE: &str = "cram.toml";

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
    /// Load and parse the config from [`CONFIG_FILE`] in the working directory.
    /// Using the exact filename (not a stem) avoids format auto-detection, which
    /// would otherwise match an extensionless `cram` binary sitting next to the
    /// config in a bundle.
    ///
    /// # Errors
    /// Returns [`Error::Config`] if the file is missing or does not match the schema.
    pub fn load() -> Result<Self, Error> {
        let raw = RawConfig::builder()
            .add_source(File::from(Path::new(CONFIG_FILE)))
            .build()?;
        Ok(raw.try_deserialize()?)
    }
}

/// Anchor relative paths (config, quizzes, web assets) to the executable's own
/// directory when the working directory has no `cram.toml`. This lets an
/// installed `cram` (binary and files together in one dir on PATH) run from
/// anywhere; dev runs and bundle runs that already sit next to the config are
/// left as-is. `canonicalize` resolves the `~/.cargo/bin` symlink to the real
/// install dir.
pub(crate) fn anchor_to_exe_dir() {
    if Path::new(CONFIG_FILE).exists() {
        return;
    }
    if let Ok(exe) = env::current_exe().and_then(|exe| exe.canonicalize())
        && let Some(dir) = exe.parent()
    {
        let _ = env::set_current_dir(dir);
    }
}
