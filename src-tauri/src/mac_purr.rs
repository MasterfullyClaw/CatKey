//! The background purr on a Mac: the same recording, looped by macOS's own player
//! (afplay), so it keeps going with the window closed.

use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

static PURR: &[u8] = include_bytes!("../sounds/purr.wav");
static ON: AtomicBool = AtomicBool::new(false);
static VOLUME: AtomicU32 = AtomicU32::new(45);
static PLAYER: Mutex<Option<Child>> = Mutex::new(None);
static GEN: AtomicU32 = AtomicU32::new(0);

pub fn is_on() -> bool {
    ON.load(Ordering::SeqCst)
}

pub fn set_volume_only(v: u32) {
    VOLUME.store(v.min(100), Ordering::SeqCst);
}

fn file() -> Option<std::path::PathBuf> {
    let p = std::env::temp_dir().join("catkey-purr.wav");
    let ok = std::fs::metadata(&p).map(|m| m.len() == PURR.len() as u64).unwrap_or(false);
    if ok || std::fs::write(&p, PURR).is_ok() {
        Some(p)
    } else {
        None
    }
}

fn stop_player() {
    if let Ok(mut g) = PLAYER.lock() {
        if let Some(mut c) = g.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

/// Turn the purr on or off (and set its volume). Returns whether it's on.
pub fn set(on: bool, volume: Option<u32>) -> bool {
    if let Some(v) = volume {
        set_volume_only(v);
    }
    let gen = GEN.fetch_add(1, Ordering::SeqCst) + 1;
    stop_player();
    ON.store(on, Ordering::SeqCst);
    if !on {
        return false;
    }
    let Some(path) = file() else {
        ON.store(false, Ordering::SeqCst);
        return false;
    };
    std::thread::spawn(move || {
        // Loop the recording until turned off (or replaced by a newer call).
        while ON.load(Ordering::SeqCst) && GEN.load(Ordering::SeqCst) == gen {
            let v = VOLUME.load(Ordering::SeqCst) as f64 / 100.0;
            let child = Command::new("/usr/bin/afplay")
                .arg("-v")
                .arg(format!("{:.3}", v * v))
                .arg(&path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            let Ok(child) = child else { break };
            if let Ok(mut g) = PLAYER.lock() {
                *g = Some(child);
            }
            loop {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let done = PLAYER
                    .lock()
                    .ok()
                    .map(|mut g| g.as_mut().map(|c| c.try_wait().ok().flatten().is_some()).unwrap_or(true))
                    .unwrap_or(true);
                if done || GEN.load(Ordering::SeqCst) != gen {
                    break;
                }
            }
        }
    });
    true
}
