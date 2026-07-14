//! cram binary: thin entry point that wires shutdown to the library.

use tokio::signal;

use cram::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    cram::main(shutdown_signal()).await
}

/// Wait for Ctrl-C to trigger a clean shutdown.
async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    tracing::info!("shutting down");
}
