//! cram binary: wires the server's outward behavior (browser, keep-awake, shutdown).

use std::process::Command;

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
        let addr = server.local_addr()?;
        open_browser(&format!("http://localhost:{}", addr.port()));
    }

    server.with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

/// Wait for Ctrl-C to trigger a clean shutdown.
async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
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

/// Best-effort: open `url` in the system's default browser (dev convenience).
/// Any failure is logged and ignored - the server runs regardless.
fn open_browser(url: &str) {
    // Pick the platform's URL opener; every branch compiles on every target.
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", ""]);
        cmd
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };
    if let Err(e) = command.arg(url).status() {
        tracing::warn!("could not open browser: {e}");
    }
}
