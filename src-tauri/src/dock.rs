//! The Cat Dock: a cat ball on the edge of the screen that opens into a little panel.
//! Inside: the food bowl (things you drop on the cat: files, text, links), the litter box
//! (files to move somewhere else), the yarn (what you copied), a focus timer that ends
//! with a cat walking across the screen, the purr, and how hard the computer works.
//!
//! Drops arrive from the system (Windows: our own drop target in `dock_drop`; Mac: the
//! window's file drops). Dragging files back out uses the system's own drag (`drag`).

use crate::app::{self, AppState};
use catkey_core::config::{DockSettings, TimerEnd};
use catkey_core::dock::{self as data, Clip, ShelfItem};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use crate::shape::SeeThrough;

/// The ball's window (the ball itself is a bit smaller, with room for its shadow).
const BALL: f64 = 84.0;
/// How much of the ball shows while it's tucked in at the edge.
const PEEK: f64 = 46.0;
const PANEL_W: f64 = 376.0;
const PANEL_H: f64 = 560.0;
const GAP: f64 = 10.0;

struct Dock {
    dir: PathBuf,
    settings: DockSettings,
    items: Vec<ShelfItem>,
    clips: Vec<Clip>,
    open: bool,
    out: bool,
    pinned: bool,
    /// The tab on show: things dropped while the litter box is open go into it.
    tab: String,
    clip_seq: u32,
    clips_paused: bool,
    hidden: bool,
    timer: Timer,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    /// "idle", "running" or "paused".
    pub state: String,
    pub total_ms: u64,
    /// While running: when it ends (ms since 1970).
    pub ends_at: u64,
    /// While paused: what's left.
    pub left_ms: u64,
    pub label: String,
    #[serde(skip)]
    gen: u64,
}

impl Timer {
    fn idle(gen: u64) -> Self {
        Timer { state: "idle".into(), total_ms: 0, ends_at: 0, left_ms: 0, label: String::new(), gen }
    }
}

static DOCK: Mutex<Option<Dock>> = Mutex::new(None);
static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
static IDS: AtomicU64 = AtomicU64::new(0);
/// While a file is being dragged out, the panel must not close under it.
static DRAGGING_OUT: AtomicBool = AtomicBool::new(false);
static WALKS: AtomicU64 = AtomicU64::new(0);

fn app_handle() -> Option<AppHandle> {
    APP.lock().ok()?.clone()
}

fn with<R>(f: impl FnOnce(&mut Dock) -> R) -> Option<R> {
    let mut g = DOCK.lock().ok()?;
    g.as_mut().map(f)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn new_id() -> String {
    format!("{:x}{:x}", now_ms(), IDS.fetch_add(1, Ordering::SeqCst))
}

fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save<T: Serialize>(path: &Path, v: &T) {
    if let Ok(t) = serde_json::to_string(v) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, t).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

fn save_items(d: &Dock) {
    save(&d.dir.join("shelf.json"), &d.items);
}

fn save_clips(d: &Dock) {
    let path = d.dir.join("clips.json");
    if d.settings.keep_clips {
        save(&path, &d.clips);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

fn emit<S: Serialize + Clone>(event: &str, payload: S) {
    if let Some(app) = app_handle() {
        let _ = app.emit_to("dock", event, payload);
    }
}

fn emit_items() {
    if let Some(items) = with(|d| d.items.clone()) {
        emit("ck://dock-items", items);
    }
}

fn emit_clips() {
    if let Some(clips) = with(|d| d.clips.clone()) {
        emit("ck://dock-clips", clips);
    }
}

fn emit_timer() {
    if let Some(t) = with(|d| d.timer.clone()) {
        if let Some(app) = app_handle() {
            let _ = app.emit("ck://timer", t);
        }
    }
}

// ------------------------------------------------------------------ start, settings, window

/// Called once at start-up.
pub fn start(app: &AppHandle, dir: PathBuf, s: &DockSettings) {
    *APP.lock().unwrap() = Some(app.clone());
    let items: Vec<ShelfItem> = load(&dir.join("shelf.json"));
    let clips: Vec<Clip> = if s.keep_clips { load(&dir.join("clips.json")) } else { Vec::new() };
    *DOCK.lock().unwrap() = Some(Dock {
        dir,
        settings: s.clone(),
        items,
        clips,
        open: false,
        out: false,
        pinned: false,
        tab: "bowl".into(),
        clip_seq: crate::win::clipboard_seq(),
        clips_paused: false,
        hidden: false,
        timer: Timer::idle(0),
    });
    if s.on {
        show(app);
    }
    std::thread::Builder::new().name("catkey-dock".into()).spawn(watch).ok();
}

/// Settings changed (in the main window or by dragging the ball).
pub fn set_settings(s: &DockSettings) {
    let Some(app) = app_handle() else { return };
    let Some((was_on, moved)) = with(|d| {
        let was_on = d.settings.on;
        let moved = d.settings.edge != s.edge || (d.settings.y - s.y).abs() > 1e-6;
        let keep_changed = d.settings.keep_clips != s.keep_clips;
        d.settings = s.clone();
        if keep_changed {
            save_clips(d);
        }
        if !s.clipboard {
            d.clips_paused = false;
        }
        (was_on, moved)
    }) else {
        return;
    };
    if s.on && !was_on {
        show(&app);
    } else if !s.on && was_on {
        if let Some(w) = app.get_webview_window("dock") {
            let _ = w.hide();
        }
    } else if s.on && moved {
        if let Some(w) = app.get_webview_window("dock") {
            place(&w);
        }
    }
    emit("ck://dock-settings", s.clone());
}

fn show(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let w = match app.get_webview_window("dock") {
            Some(w) => w,
            None => match build(&app) {
                Some(w) => w,
                None => return,
            },
        };
        place(&w);
        let _ = w.show();
    });
}

fn build(app: &AppHandle) -> Option<WebviewWindow> {
    let b = WebviewWindowBuilder::new(app, "dock", WebviewUrl::App("dock.html".into()))
        .title("Cat Dock")
        .inner_size(BALL, BALL)
        .decorations(false)
        .see_through()
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false);
    // On Windows the Dock catches drops itself (files, text and links); on a Mac the
    // window's own file drops do.
    #[cfg(windows)]
    let b = b.disable_drag_drop_handler();
    let w = b.build().map_err(|e| crate::log(&format!("dock window: {e}"))).ok()?;
    #[cfg(windows)]
    crate::dock_drop::attach(&w);
    #[cfg(target_os = "macos")]
    let _ = w.set_visible_on_all_workspaces(true);
    w.on_window_event(|e| match e {
        // Clicking somewhere else tucks the cat back in (unless the panel is pinned).
        tauri::WindowEvent::Focused(false) => {
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(250));
                let close = with(|d| d.open && !d.pinned).unwrap_or(false) && !DRAGGING_OUT.load(Ordering::SeqCst);
                if close {
                    if let Some(app) = app_handle() {
                        let focused = app.get_webview_window("dock").and_then(|w| w.is_focused().ok()).unwrap_or(false);
                        if !focused {
                            set_open(&app, false);
                        }
                    }
                }
            });
        }
        #[cfg(not(windows))]
        tauri::WindowEvent::DragDrop(ev) => match ev {
            tauri::DragDropEvent::Enter { .. } => drag_phase("enter"),
            tauri::DragDropEvent::Leave => drag_phase("leave"),
            tauri::DragDropEvent::Drop { paths, .. } => {
                dropped(paths.iter().map(|p| p.to_string_lossy().to_string()).collect(), String::new());
                drag_phase("drop");
            }
            _ => {}
        },
        _ => {}
    });
    std::thread::sleep(Duration::from_millis(300));
    crate::log("dock: window ready");
    Some(w)
}

/// Put the window where it belongs: the ball tucked into its edge (or peeking out), or
/// the open panel next to it.
pub fn place(w: &WebviewWindow) {
    let Some((s, open, out)) = with(|d| (d.settings.clone(), d.open, d.out)) else { return };
    let Some(m) = w.current_monitor().ok().flatten().or_else(|| w.primary_monitor().ok().flatten()) else { return };
    let k = m.scale_factor();
    let wa = m.work_area();
    let (l, t) = (wa.position.x as f64, wa.position.y as f64);
    let (ww, wh) = (wa.size.width as f64, wa.size.height as f64);
    let right = s.edge != "left";
    let ball = BALL * k;
    let y_ball = t + s.y.clamp(0.0, 1.0) * (wh - ball).max(0.0);
    let (wpx, hpx, x, y) = if open {
        let pw = PANEL_W * k;
        let ph = (PANEL_H * k).min(wh - 2.0 * GAP * k);
        let x = if right { l + ww - pw - GAP * k } else { l + GAP * k };
        let y = (y_ball + ball / 2.0 - ph / 2.0).clamp(t + GAP * k, t + wh - ph - GAP * k);
        (pw, ph, x, y)
    } else {
        let shown = if out { BALL } else { PEEK } * k;
        let x = if right { l + ww - shown } else { l - (ball - shown) };
        (ball, ball, x, y_ball)
    };
    let _ = w.set_size(PhysicalSize::new(wpx.round() as u32, hpx.round() as u32));
    let _ = w.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

fn set_open(app: &AppHandle, open: bool) {
    let changed = with(|d| {
        let changed = d.open != open;
        d.open = open;
        if !open {
            d.out = false;
        }
        changed
    })
    .unwrap_or(false);
    let Some(w) = app.get_webview_window("dock") else { return };
    let _ = w.emit("ck://dock-open", open);
    if changed {
        // Let the page switch first, so the panel doesn't flash at ball size.
        std::thread::sleep(Duration::from_millis(open.then_some(16).unwrap_or(120)));
    }
    place(&w);
    if open {
        let _ = w.set_focus();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockInfo {
    items: Vec<ShelfItem>,
    clips: Vec<Clip>,
    open: bool,
    pinned: bool,
    tab: String,
    clips_paused: bool,
    settings: DockSettings,
    timer: Timer,
    purring: bool,
    purr_volume: u32,
    platform: &'static str,
}

#[tauri::command]
pub fn dock_info(state: tauri::State<AppState>) -> Option<DockInfo> {
    let purr_volume = state.config.lock().map(|c| c.settings.purr_volume).unwrap_or(45);
    with(|d| DockInfo {
        items: d.items.clone(),
        clips: d.clips.clone(),
        open: d.open,
        pinned: d.pinned,
        tab: d.tab.clone(),
        clips_paused: d.clips_paused,
        settings: d.settings.clone(),
        timer: d.timer.clone(),
        purring: crate::purr::is_on(),
        purr_volume,
        platform: if cfg!(target_os = "macos") { "mac" } else { "windows" },
    })
}

#[tauri::command]
pub fn dock_open(app: AppHandle, open: bool) {
    let app2 = app.clone();
    std::thread::spawn(move || set_open(&app2, open));
}

/// The ball peeks out of its edge while the pointer is on it.
#[tauri::command]
pub fn dock_peek(app: AppHandle, out: bool) {
    let open = with(|d| {
        d.out = out;
        d.open
    })
    .unwrap_or(true);
    if !open {
        if let Some(w) = app.get_webview_window("dock") {
            place(&w);
        }
    }
}

#[tauri::command]
pub fn dock_pin(pinned: bool) {
    with(|d| d.pinned = pinned);
}

#[tauri::command]
pub fn dock_tab(tab: String) {
    with(|d| d.tab = tab);
}

/// Pressed on the ball: follow the pointer while the button is held, then stick to the
/// nearest side of the screen. A press that hardly moves is a click: open or close.
#[tauri::command]
pub fn dock_press(app: AppHandle) {
    std::thread::spawn(move || {
        let Some(w) = app.get_webview_window("dock") else { return };
        let (Ok(start), Ok(pos)) = (app.cursor_position(), w.outer_position()) else { return };
        let (gx, gy) = (start.x - pos.x as f64, start.y - pos.y as f64);
        let mut moved = false;
        while crate::win::mouse_down() {
            std::thread::sleep(Duration::from_millis(8));
            let Ok(c) = app.cursor_position() else { break };
            if !moved && ((c.x - start.x).abs() > 5.0 || (c.y - start.y).abs() > 5.0) {
                moved = true;
                let _ = w.emit("ck://dock-dragging", true);
            }
            if moved {
                let _ = w.set_position(PhysicalPosition::new((c.x - gx).round() as i32, (c.y - gy).round() as i32));
            }
        }
        if !moved {
            let open = with(|d| d.open).unwrap_or(false);
            set_open(&app, !open);
            return;
        }
        let _ = w.emit("ck://dock-dragging", false);
        let Ok(c) = app.cursor_position() else { return };
        let Some(m) = app.monitor_from_point(c.x, c.y).ok().flatten().or_else(|| w.current_monitor().ok().flatten()) else { return };
        let k = m.scale_factor();
        let wa = m.work_area();
        let (l, t) = (wa.position.x as f64, wa.position.y as f64);
        let (ww, wh) = (wa.size.width as f64, wa.size.height as f64);
        let edge = if c.x - l > ww / 2.0 { "right" } else { "left" };
        let y = ((c.y - gy - t) / (wh - BALL * k).max(1.0)).clamp(0.0, 1.0);
        // Move onto that monitor first, so `place` measures the right one.
        let _ = w.set_position(PhysicalPosition::new((l + ww / 2.0) as i32, (t + y * (wh - BALL * k)) as i32));
        with(|d| {
            d.settings.edge = edge.into();
            d.settings.y = y;
            d.out = false;
        });
        place(&w);
        save_position(&app, edge, y);
    });
}

/// The purr's volume, from the Dock's player (kept in the same setting as Settings → Sounds).
#[tauri::command]
pub fn dock_purr_volume(app: AppHandle, volume: u32) {
    let state = app.state::<AppState>();
    let mut cfg = state.config.lock().unwrap().clone();
    cfg.settings.purr_volume = volume.min(100);
    if let Ok(cfg) = app::apply_config(&app, cfg) {
        let _ = app.emit("ck://settings", &cfg.settings);
    }
}

/// Remember where the ball sits, in the config the main window shows too.
fn save_position(app: &AppHandle, edge: &str, y: f64) {
    let state = app.state::<AppState>();
    let mut cfg = state.config.lock().unwrap().clone();
    cfg.settings.dock.edge = edge.into();
    cfg.settings.dock.y = (y * 1000.0).round() / 1000.0;
    if let Ok(cfg) = app::apply_config(app, cfg) {
        let _ = app.emit("ck://settings", &cfg.settings);
    }
}

// ------------------------------------------------------------------ the food bowl and the litter box

/// Something was dropped on the Dock.
pub fn dropped(paths: Vec<String>, text: String) {
    let paths: Vec<String> = paths.into_iter().filter(|p| !p.trim().is_empty()).collect();
    let kind = if !paths.is_empty() { "files" } else { data::text_kind(&text) };
    let changed = with(|d| {
        let item = ShelfItem {
            id: new_id(),
            kind: kind.into(),
            paths,
            text: data::clip_text(text.trim_end(), data::MAX_TEXT),
            cut: d.tab == "litter" && d.open,
            added: now_ms() / 1000,
        };
        let changed = data::push_shelf(&mut d.items, item);
        if changed {
            save_items(d);
        }
        changed
    })
    .unwrap_or(false);
    if changed {
        emit_items();
    }
}

/// A drag is over the Dock: "enter", "leave" or "drop" (the cat reaches out to catch it).
pub fn drag_phase(phase: &str) {
    if let Some(app) = app_handle() {
        if phase == "enter" {
            let open = with(|d| {
                d.out = true;
                d.open
            })
            .unwrap_or(true);
            if !open {
                if let Some(w) = app.get_webview_window("dock") {
                    place(&w);
                }
            }
        }
        let _ = app.emit_to("dock", "ck://dock-drag", phase.to_string());
    }
}

#[tauri::command]
pub fn dock_add_text(text: String) {
    if !text.trim().is_empty() {
        dropped(Vec::new(), text);
    }
}

#[tauri::command]
pub fn dock_remove(id: String) {
    with(|d| {
        d.items.retain(|x| x.id != id);
        save_items(d);
    });
    emit_items();
}

/// Into the litter box (move when dragged out) or back into the bowl (copy).
#[tauri::command]
pub fn dock_set_cut(id: String, cut: bool) {
    with(|d| {
        if let Some(x) = d.items.iter_mut().find(|x| x.id == id) {
            x.cut = cut;
        }
        save_items(d);
    });
    emit_items();
}

/// Empty the bowl (`cut` false) or the litter box (`cut` true).
#[tauri::command]
pub fn dock_clear(cut: bool) {
    with(|d| {
        d.items.retain(|x| x.cut != cut);
        save_items(d);
    });
    emit_items();
}

fn item(id: &str) -> Option<ShelfItem> {
    with(|d| d.items.iter().find(|x| x.id == id).cloned()).flatten()
}

/// Open a dropped file or link with its usual app.
#[tauri::command]
pub fn dock_open_item(id: String) -> Result<(), String> {
    let it = item(&id).ok_or("It isn't in the bowl any more.")?;
    if it.kind == "link" {
        return crate::win::open_url(it.text.trim());
    }
    let p = it.paths.first().ok_or("Nothing to open.")?;
    crate::win::open_path(p)
}

/// Show a dropped file in its folder.
#[tauri::command]
pub fn dock_reveal(id: String) -> Result<(), String> {
    let it = item(&id).ok_or("It isn't in the bowl any more.")?;
    let p = it.paths.first().ok_or("Nothing to show.")?;
    crate::win::reveal_path(p)
}

/// Put a kept thing on the clipboard (files from the litter box paste as a move).
#[tauri::command]
pub fn dock_copy_item(id: String) -> Result<(), String> {
    let it = item(&id).ok_or("It isn't in the bowl any more.")?;
    let ok = if it.kind == "files" { crate::win::clipboard_write_files(&it.paths, it.cut) } else { crate::win::clipboard_write_text(&it.text) };
    if ok {
        Ok(())
    } else {
        Err("Couldn't reach the clipboard. Try again.".into())
    }
}

static DRAG_ICON: &[u8] = include_bytes!("../icons/64x64.png");

/// Drag files out of the bowl into a folder or an app: a copy, or from the litter box a move.
#[tauri::command]
pub async fn dock_drag_out(app: AppHandle, window: WebviewWindow, id: String) -> Result<bool, String> {
    let it = item(&id).ok_or("It isn't in the bowl any more.")?;
    let paths: Vec<PathBuf> = it.paths.iter().map(PathBuf::from).filter(|p| p.exists()).collect();
    if paths.is_empty() {
        return Err("Those files aren't where they were any more.".into());
    }
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    let mode = if it.cut { drag::DragMode::Move } else { drag::DragMode::Copy };
    DRAGGING_OUT.store(true, Ordering::SeqCst);
    let w = window.clone();
    app.run_on_main_thread(move || {
        let done = tx.clone();
        let r = drag::start_drag(
            &w,
            drag::DragItem::Files(paths),
            drag::Image::Raw(DRAG_ICON.to_vec()),
            move |result, _| {
                let _ = done.send(matches!(result, drag::DragResult::Dropped));
            },
            drag::Options { skip_animatation_on_cancel_or_failure: false, mode },
        );
        if r.is_err() {
            let _ = tx.send(false);
        }
    })
    .map_err(|e| e.to_string())?;
    let dropped = tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(600)).unwrap_or(false))
        .await
        .unwrap_or(false);
    DRAGGING_OUT.store(false, Ordering::SeqCst);
    // Moved out of the litter box: it lives somewhere else now.
    if dropped && it.cut {
        let gone = it.paths.iter().all(|p| !Path::new(p).exists());
        if gone {
            dock_remove(id);
        }
    }
    Ok(dropped)
}

// ------------------------------------------------------------------ the yarn: what you copied

#[tauri::command]
pub fn clip_copy(id: String) -> Result<(), String> {
    let c = with(|d| d.clips.iter().find(|c| c.id == id).cloned()).flatten().ok_or("That copy is gone.")?;
    let ok = if c.kind == "files" { crate::win::clipboard_write_files(&c.paths, false) } else { crate::win::clipboard_write_text(&c.text) };
    if ok {
        Ok(())
    } else {
        Err("Couldn't reach the clipboard. Try again.".into())
    }
}

/// Keep a copy in the bowl (copied files stay where they are; the bowl remembers them).
#[tauri::command]
pub fn dock_keep_clip(id: String) {
    if let Some(c) = with(|d| d.clips.iter().find(|c| c.id == id).cloned()).flatten() {
        dropped(c.paths, c.text);
    }
}

#[tauri::command]
pub fn clip_pin(id: String, pinned: bool) {
    with(|d| {
        if let Some(c) = d.clips.iter_mut().find(|c| c.id == id) {
            c.pinned = pinned;
        }
        save_clips(d);
    });
    emit_clips();
}

#[tauri::command]
pub fn clip_delete(id: String) {
    with(|d| {
        d.clips.retain(|c| c.id != id);
        save_clips(d);
    });
    emit_clips();
}

/// Forget everything copied (pinned ones too when `all`).
#[tauri::command]
pub fn clip_clear(all: bool) {
    with(|d| {
        d.clips.retain(|c| c.pinned && !all);
        save_clips(d);
    });
    emit_clips();
}

#[tauri::command]
pub fn clip_pause(paused: bool) {
    with(|d| {
        d.clips_paused = paused;
        d.clip_seq = crate::win::clipboard_seq();
    });
    emit("ck://dock-clips-paused", paused);
}

/// Watches the clipboard (only while the Dock keeps a history), full-screen apps and the vitals.
fn watch() {
    let mut tick: u64 = 0;
    let mut vitals = crate::vitals::Sampler::new();
    loop {
        std::thread::sleep(Duration::from_millis(500));
        tick += 1;
        let Some((on, history, paused, last, show_vitals, open, hidden)) =
            with(|d| (d.settings.on, d.settings.clipboard, d.clips_paused, d.clip_seq, d.settings.vitals, d.open, d.hidden))
        else {
            continue;
        };
        if !on {
            continue;
        }
        if history && !paused {
            let seq = crate::win::clipboard_seq();
            if seq != last {
                with(|d| d.clip_seq = seq);
                if let Some(c) = crate::win::clipboard_read() {
                    let clip = Clip {
                        id: new_id(),
                        kind: if c.files.is_empty() { "text" } else { "files" }.into(),
                        text: c.text,
                        paths: c.files,
                        pinned: false,
                        at: now_ms() / 1000,
                    };
                    let changed = with(|d| {
                        let changed = data::push_clip(&mut d.clips, clip);
                        if changed {
                            save_clips(d);
                        }
                        changed
                    })
                    .unwrap_or(false);
                    if changed {
                        emit_clips();
                    }
                }
            }
        }
        // Out of the way while a game or a film fills the screen.
        #[cfg(windows)]
        if tick % 2 == 0 {
            let full = unsafe { crate::coach::full_screen_app() };
            if full != hidden && !open {
                with(|d| d.hidden = full);
                if let Some(w) = app_handle().and_then(|a| a.get_webview_window("dock")) {
                    let _ = if full { w.hide() } else { w.show() };
                }
            }
        }
        #[cfg(not(windows))]
        let _ = hidden;
        // How hard the computer works: every 2 s, light enough to never show up in it.
        if show_vitals && tick % 4 == 0 {
            let v = vitals.sample(open);
            if tick == 8 {
                crate::log(&format!("dock: vitals {v:?}"));
            }
            if let Some(app) = app_handle() {
                let _ = app.emit("ck://vitals", v);
            }
        }
    }
}

// ------------------------------------------------------------------ the focus timer

#[tauri::command]
pub fn timer_start(app: AppHandle, minutes: f64, label: String) {
    let ms = (minutes.clamp(0.1, 600.0) * 60_000.0).round() as u64;
    let gen = with(|d| {
        let gen = d.timer.gen + 1;
        d.timer = Timer { state: "running".into(), total_ms: ms, ends_at: now_ms() + ms, left_ms: ms, label, gen };
        gen
    });
    if let Some(gen) = gen {
        run_timer(app, gen);
    }
    emit_timer();
}

#[tauri::command]
pub fn timer_pause() {
    with(|d| {
        if d.timer.state == "running" {
            d.timer.left_ms = d.timer.ends_at.saturating_sub(now_ms());
            d.timer.state = "paused".into();
            d.timer.gen += 1;
        }
    });
    emit_timer();
}

#[tauri::command]
pub fn timer_resume(app: AppHandle) {
    let gen = with(|d| {
        if d.timer.state != "paused" {
            return None;
        }
        d.timer.ends_at = now_ms() + d.timer.left_ms;
        d.timer.state = "running".into();
        d.timer.gen += 1;
        Some(d.timer.gen)
    })
    .flatten();
    if let Some(gen) = gen {
        run_timer(app, gen);
    }
    emit_timer();
}

#[tauri::command]
pub fn timer_stop() {
    with(|d| d.timer = Timer::idle(d.timer.gen + 1));
    emit_timer();
}

fn run_timer(app: AppHandle, gen: u64) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(400));
        let Some(t) = with(|d| d.timer.clone()) else { return };
        if t.gen != gen || t.state != "running" {
            return;
        }
        if now_ms() >= t.ends_at {
            let label = t.label.clone();
            with(|d| d.timer = Timer::idle(d.timer.gen + 1));
            emit_timer();
            let mins = (t.total_ms as f64 / 60_000.0).round() as u64;
            let text = if label.is_empty() {
                format!("{mins} minutes done! Time to stretch like a cat.")
            } else {
                format!("{label}: done! Time to stretch like a cat.")
            };
            let end = with(|d| d.settings.end.clone()).unwrap_or_default();
            let kind = end.pick(now_ms() / 997);
            ending(&app, text, kind, &end);
            return;
        }
    });
}

// ------------------------------------------------------------------ the cat walk

#[derive(Clone, Serialize)]
pub struct WalkPayload {
    text: String,
    seq: u64,
    /// "walk" or "spin".
    kind: &'static str,
    motion: bool,
    sound: bool,
}

static LAST_WALK: Mutex<Option<WalkPayload>> = Mutex::new(None);

/// The cat walk (the classic ending), for callers that just want a message walked across.
pub fn walk(app: &AppHandle, text: String) {
    let end = with(|d| d.settings.end.clone()).unwrap_or_default();
    ending(app, text, "walk", &end);
}

/// The end of a focus timer: a cat walks along the bottom of the screen with a message, or
/// spins in the middle of the screen where the pointer is. Clicks go straight through.
pub fn ending(app: &AppHandle, text: String, kind: &'static str, end: &TimerEnd) {
    let seq = WALKS.fetch_add(1, Ordering::SeqCst) + 1;
    let p = WalkPayload { text, seq, kind, motion: end.motion, sound: end.sound };
    *LAST_WALK.lock().unwrap() = Some(p.clone());
    let app = app.clone();
    std::thread::spawn(move || {
        let w = match app.get_webview_window("walk") {
            Some(w) => w,
            None => {
                let b = WebviewWindowBuilder::new(&app, "walk", WebviewUrl::App("walk.html".into()))
                    .title("Cat walk")
                    .inner_size(800.0, 220.0)
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
                let Ok(w) = b.build() else { return };
                #[cfg(windows)]
                crate::dock_drop::no_activate(&w);
                std::thread::sleep(Duration::from_millis(500));
                w
            }
        };
        let m = if kind == "spin" { monitor_at_pointer(&app) } else { None }.or_else(|| w.primary_monitor().ok().flatten());
        if let Some(m) = m {
            let k = m.scale_factor();
            let wa = m.work_area();
            if kind == "spin" {
                let _ = w.set_size(PhysicalSize::new(wa.size.width, wa.size.height));
                let _ = w.set_position(PhysicalPosition::new(wa.position.x, wa.position.y));
            } else {
                let h = (230.0 * k) as u32;
                let _ = w.set_size(PhysicalSize::new(wa.size.width, h));
                let _ = w.set_position(PhysicalPosition::new(wa.position.x, wa.position.y + wa.size.height as i32 - h as i32));
            }
        }
        let _ = w.set_ignore_cursor_events(true);
        let _ = w.emit("ck://walk", p);
        let _ = w.show();
        std::thread::sleep(Duration::from_secs(if kind == "spin" { 10 } else { 18 }));
        if WALKS.load(Ordering::SeqCst) == seq {
            let _ = w.hide();
        }
    });
}

/// The screen the pointer is on (where the spinning cat shows up).
fn monitor_at_pointer(app: &AppHandle) -> Option<tauri::Monitor> {
    let c = app.cursor_position().ok()?;
    app.available_monitors().ok()?.into_iter().find(|m| {
        let (p, s) = (m.position(), m.size());
        c.x >= p.x as f64 && c.y >= p.y as f64 && c.x < p.x as f64 + s.width as f64 && c.y < p.y as f64 + s.height as f64
    })
}

/// The walk window asks for its walk when it has loaded.
#[tauri::command]
pub fn walk_last() -> Option<WalkPayload> {
    LAST_WALK.lock().ok()?.clone()
}

/// Try an ending: "walk", "spin", or none for the one the settings pick.
#[tauri::command]
pub fn walk_test(app: AppHandle, kind: Option<String>) {
    let end = with(|d| d.settings.end.clone()).unwrap_or_default();
    let kind = match kind.as_deref() {
        Some("spin") => "spin",
        Some("walk") => "walk",
        _ => end.pick(now_ms() / 997),
    };
    let text = if kind == "spin" {
        "This is how a focus timer can end. Stretch like a cat!"
    } else {
        "This is what the end of a focus timer looks like. Stretch like a cat!"
    };
    ending(&app, text.into(), kind, &end);
}

/// How focus timers end (from the Dock's Focus page), kept with the other Dock settings.
#[tauri::command]
pub fn dock_set_end(app: AppHandle, end: TimerEnd) {
    let state = app.state::<AppState>();
    let mut cfg = state.config.lock().unwrap().clone();
    cfg.settings.dock.end = end;
    if let Ok(cfg) = app::apply_config(&app, cfg) {
        let _ = app.emit("ck://settings", &cfg.settings);
    }
}
