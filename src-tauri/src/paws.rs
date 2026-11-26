//! Paw prints: where you click, scroll or drag, your cat leaves a little paw print that
//! fades at once. They show on a see-through window that clicks go straight through and
//! that never takes the keyboard; it's only on screen while there are paws on it.
//!
//! The mouse comes from the coach's raw input on Windows and from a listen-only event
//! tap on a Mac, and lands here through a channel, so the mouse never waits on us.

use catkey_core::config::PawSettings;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use crate::shape::SeeThrough;

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Down,
    Up,
    Right,
    Scroll(i32),
    Move,
}

static ON: AtomicBool = AtomicBool::new(false);
static TX: Mutex<Option<Sender<Input>>> = Mutex::new(None);

pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// From the mouse's thread: cheap, never blocks.
pub fn input(i: Input) {
    if !enabled() {
        return;
    }
    if let Ok(g) = TX.lock() {
        if let Some(tx) = g.as_ref() {
            let _ = tx.send(i);
        }
    }
}

pub fn start(app: &AppHandle, s: &PawSettings) {
    let (tx, rx) = channel();
    *TX.lock().unwrap() = Some(tx);
    let app = app.clone();
    std::thread::Builder::new().name("catkey-paws".into()).spawn(move || worker(app, rx)).ok();
    set_settings(s);
}

pub fn set_settings(s: &PawSettings) {
    ON.store(s.on, Ordering::SeqCst);
    #[cfg(target_os = "macos")]
    if s.on {
        crate::mac_paws::start();
    }
    #[cfg(windows)]
    crate::paw_cursor::apply(s.cursor, &s.colors);
}

#[derive(Serialize, Clone)]
struct Paw {
    kind: &'static str,
    x: f64,
    y: f64,
    /// Direction, in degrees (drags), or +1/-1 (scrolling).
    a: f64,
}

fn build(app: &AppHandle) -> Option<WebviewWindow> {
    let b = WebviewWindowBuilder::new(app, "paws", WebviewUrl::App("paws.html".into()))
        .title("Paw prints")
        .inner_size(400.0, 300.0)
        .decorations(false)
        .see_through()
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false);
    #[cfg(windows)]
    let b = b.disable_drag_drop_handler();
    let w = b.build().map_err(|e| crate::log(&format!("paws window: {e}"))).ok()?;
    #[cfg(windows)]
    crate::dock_drop::no_activate(&w);
    #[cfg(target_os = "macos")]
    let _ = w.set_visible_on_all_workspaces(true);
    let _ = w.set_ignore_cursor_events(true);
    std::thread::sleep(Duration::from_millis(400));
    Some(w)
}

fn show(w: &WebviewWindow) {
    #[cfg(windows)]
    crate::coach::show_quietly(w);
    #[cfg(not(windows))]
    let _ = w.show();
}

fn worker(app: AppHandle, rx: Receiver<Input>) {
    let mut win: Option<WebviewWindow> = None;
    // The monitor the window covers: left, top, width, height, scale.
    let mut mon: Option<(i32, i32, u32, u32, f64)> = None;
    let mut shown = false;
    let mut last = Instant::now();
    let mut held = false;
    let mut trail = (0.0f64, 0.0f64);
    let mut last_scroll = Instant::now() - Duration::from_secs(1);
    loop {
        let ev = match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(ev) => ev,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if shown && last.elapsed() > Duration::from_millis(1700) {
                    if let Some(w) = &win {
                        let _ = w.hide();
                    }
                    shown = false;
                }
                continue;
            }
            Err(_) => return,
        };
        if !enabled() {
            continue;
        }
        // Out of the way of games and films.
        #[cfg(windows)]
        if unsafe { crate::coach::full_screen_app() } {
            continue;
        }
        let Ok(c) = app.cursor_position() else { continue };
        let paw = match ev {
            Input::Down => {
                held = true;
                trail = (c.x, c.y);
                Some(("click", 0.0))
            }
            Input::Up => {
                held = false;
                None
            }
            Input::Right => Some(("right", 0.0)),
            Input::Scroll(d) => {
                if last_scroll.elapsed() < Duration::from_millis(140) {
                    None
                } else {
                    last_scroll = Instant::now();
                    Some(("scroll", if d > 0 { -1.0 } else { 1.0 }))
                }
            }
            Input::Move => {
                let (dx, dy) = (c.x - trail.0, c.y - trail.1);
                if held && (dx * dx + dy * dy).sqrt() > 34.0 * mon.map(|m| m.4).unwrap_or(1.0) {
                    trail = (c.x, c.y);
                    Some(("trail", dy.atan2(dx).to_degrees()))
                } else {
                    None
                }
            }
        };
        let Some((kind, a)) = paw else { continue };
        if win.is_none() {
            win = build(&app);
        }
        let Some(w) = &win else { continue };
        // Cover the monitor the pointer is on.
        let here = app.monitor_from_point(c.x, c.y).ok().flatten();
        if let Some(m) = here {
            let r = (m.position().x, m.position().y, m.size().width, m.size().height, m.scale_factor());
            if mon != Some(r) {
                let _ = w.set_position(PhysicalPosition::new(r.0, r.1));
                let _ = w.set_size(PhysicalSize::new(r.2, r.3));
                mon = Some(r);
            }
        }
        let Some((mx, my, _, _, k)) = mon else { continue };
        let _ = w.emit("ck://paw", Paw { kind, x: (c.x - mx as f64) / k, y: (c.y - my as f64) / k, a });
        if !shown {
            show(w);
            let _ = w.set_ignore_cursor_events(true);
            shown = true;
        }
        last = Instant::now();
    }
}

/// Settings → "Try the paws": a few prints around the pointer.
#[tauri::command]
pub fn paws_try() {
    let was = ON.swap(true, Ordering::SeqCst);
    std::thread::spawn(move || {
        for i in [Input::Down, Input::Up] {
            input(i);
            std::thread::sleep(Duration::from_millis(60));
        }
        std::thread::sleep(Duration::from_millis(1800));
        if !was {
            ON.store(false, Ordering::SeqCst);
        }
    });
}

/// The paw prints page asks which cat it is (colours) when it loads.
#[tauri::command]
pub fn paws_colors(app: AppHandle) -> Vec<String> {
    app.state::<crate::app::AppState>().config.lock().map(|c| c.settings.paws.colors.clone()).unwrap_or_default()
}
