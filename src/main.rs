//! cram binary: wires the server's outward behavior (browser, keep-awake, shutdown).

use keepawake::{Builder, KeepAwake};
use tokio::signal;

use cram::Error;

/// Open the index page in the browser on startup (dev convenience).
const OPEN_BROWSER: bool = true;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let server = cram::main().await?;
    // Keep the display and system awake while serving; released on drop.
    let _awake = keep_awake();

    if OPEN_BROWSER {
        open::that(format!("http://localhost:{}", server.local_addr()?.port()))
            .inspect_err(|err| tracing::warn!("could not open browser: {err}"))
            .ok();
    }

    server.with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

/// Wait for Ctrl-C to trigger a clean shutdown.
async fn shutdown_signal() {
    signal::ctrl_c().await.ok();
    tracing::info!("shutting down");
}

/// Keep the display and system awake while the server runs, cross-platform via
/// `keepawake`. Returns a guard that releases the request when dropped; `None`
/// if the platform request failed (best-effort - the server runs regardless).
fn keep_awake() -> Option<KeepAwake> {
    Builder::default()
        .display(true)
        .idle(true)
        .reason("serving quizzes")
        .app_name("cram")
        .create()
        .inspect_err(|e| tracing::warn!("could not prevent sleep: {e}"))
        .ok()
}
