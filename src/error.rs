//! Server error types.

use std::io::Error as IoError;

use config::ConfigError;
use thiserror::Error;
use toml::de::Error as TomlError;

/// Errors that can occur while running the server.
#[derive(Debug, Error)]
pub enum Error {
    /// Config file could not be read or parsed.
    #[error("config: {0}")]
    Config(#[from] ConfigError),

    /// I/O error while binding, serving, or reading quiz files.
    #[error("io: {0}")]
    Io(#[from] IoError),

    /// A quiz file could not be parsed as TOML.
    #[error("toml: {0}")]
    Toml(#[from] TomlError),

    /// A quiz failed content validation.
    #[error("content: {0}")]
    Content(String),
}
