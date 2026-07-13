//! Server error types.

use std::io::Error as IoError;

use config::ConfigError;
use thiserror::Error;

/// Errors that can occur while running the server.
#[derive(Debug, Error)]
pub enum Error {
    /// Config file could not be read or parsed.
    #[error("config: {0}")]
    Config(#[from] ConfigError),

    /// I/O error while binding or serving.
    #[error("io: {0}")]
    Io(#[from] IoError),
}
