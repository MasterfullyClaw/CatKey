#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Cat Key: a purr-fect keyboard shortcut remapper for Windows and macOS.
//!
//! The system-specific parts have the same module names on both systems, so the rest of
//! the app reads the same: `win` (system helpers), `hook` (the keyboard engine's eyes and
//! hands), `coach` and `purr`. On a Mac they come from the `mac*` files.

mod app;
mod commands;
mod dock;
#[cfg(windows)]
mod dock_drop;
#[cfg(target_os = "macos")]
mod mac_paws;
#[cfg(windows)]
mod paw_cursor;
mod mcp;
mod paws;
mod switcher;
mod tray;
mod vitals;
#[cfg(target_os = "macos")]
mod mac_switch;

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
    // If anything goes badly wrong, say what in catkey.log, so a window that never opens can be explained.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let at = info.location().map(|l| format!(" at {}:{}", l.file(), l.line())).unwrap_or_default();
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown problem".into());
        let thread = std::thread::current().name().unwrap_or("unnamed").to_string();
        log(&format!("crashed on thread {thread}{at}: {what}"));
        default_hook(info);
    }));
    let args: Vec<String> = std::env::args().collect();
    // Started by an AI app (Claude Desktop…) as its Cat Key connection: speak MCP and nothing else.
    if args.iter().any(|a| a == "--mcp") {
        mcp::serve();
        return;
    }
    // For checking: write the apps "Your apps" would list to a file, and quit.
    if let Some(i) = args.iter().position(|a| a == "--list-apps") {
        let apps = win::installed_apps();
        let text = serde_json::to_string_pretty(&apps).unwrap_or_default();
        match args.get(i + 1) {
            Some(file) => {
                let _ = std::fs::write(file, text);
            }
            None => println!("{text}"),
        }
        return;
    }
    log(&format!("start {} ({})", env!("CARGO_PKG_VERSION"), std::env::consts::OS));
    // After a relaunch (for example into admin mode), wait for the old copy to exit.
    if let Some(i) = args.iter().position(|a| a == "--wait-pid") {
        if let Some(pid) = args.get(i + 1).and_then(|p| p.parse::<u32>().ok()) {
            win::wait_for_pid(pid, 15_000);
        }
    }
    app::run(args);
}
