#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Cat Key: a purr-fect keyboard shortcut remapper for Windows and macOS.
//!
//! The system-specific parts have the same module names on both systems, so the rest of
//! the app reads the same: `win` (system helpers), `hook` (the keyboard engine's eyes and
//! hands), `coach` and `purr`. On a Mac they come from the `mac*` files.

mod app;
mod commands;
mod tray;

#[cfg(windows)]
mod coach;
#[cfg(windows)]
mod hook;
#[cfg(windows)]
mod purr;
#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
#[path = "mac_coach.rs"]
mod coach;
#[cfg(target_os = "macos")]
#[path = "mac_hook.rs"]
mod hook;
#[cfg(target_os = "macos")]
#[path = "mac_purr.rs"]
mod purr;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod win;

use std::io::Write;

/// Append a line to catkey.log in the data folder (kept small).
pub fn log(msg: &str) {
    let path = app::config_dir().join("catkey.log");
    if std::fs::metadata(&path).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{}] {}", win::local_date(), msg);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // After a relaunch (for example into admin mode), wait for the old copy to exit.
    if let Some(i) = args.iter().position(|a| a == "--wait-pid") {
        if let Some(pid) = args.get(i + 1).and_then(|p| p.parse::<u32>().ok()) {
            win::wait_for_pid(pid, 15_000);
        }
    }
    app::run(args);
}
