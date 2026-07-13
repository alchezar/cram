//! cram - local server for the English trainers.

mod config;
mod error;

use crate::config::Config;
use crate::error::Error;

// Temporary smoke test: load the config and print it. Replaced by the server next step.
fn main() -> Result<(), Error> {
    let config = Config::load("cram")?;
    println!("port={}, web_dir={}", config.port, config.web_dir.display());
    Ok(())
}
