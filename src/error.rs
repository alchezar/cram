//! Server error types.

use config::ConfigError;
use thiserror::Error;

/// Errors that can occur while running the server.
#[derive(Debug, Error)]
pub enum Error {
    /// Config file could not be read or parsed.
    #[error("config: {0}")]
    Config(#[from] ConfigError),
}
