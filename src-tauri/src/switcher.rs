//! The app switcher: rest the pointer in a corner (or the middle of an edge) and a little
//! cat panel of your open windows pops up right there; click one to jump to it. It pops
//! up without taking the keyboard and goes away when the pointer leaves.
//!
//! A shortcut can open it too (a rule that opens `catkey://switch`): then it has the
//! keyboard, so the arrows, Enter, the numbers and typing to search all work.
//!
//! One small thread looks at the pointer a few times a second with the system's own calls
//! (no hook, no window), so it costs next to nothing.

use catkey_core::config::SwitcherSettings;
use catkey_core::switcher::{self as geo, Dwell, Rect};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use crate::shape::SeeThrough;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwitchItem {
    /// The window (Windows: its handle) or the app (Mac: its process id).
    pub id: String,
    pub title: String,
    /// The app's name ("Google Chrome").
    pub app: String,
    pub exe: String,
    pub path: String,
    /// In front right now.
    pub front: bool,
    /// Minimised (Windows), or hidden / with no window on screen (Mac).
    pub min: bool,
    pub can_close: bool,
}

/// A screen, in the units the system's pointer uses (Windows: pixels; Mac: points).
#[derive(Clone, Copy, Debug)]
pub struct Screen {
    pub full: Rect,
    pub work: Rect,
    /// Pixels per point, for sizes: the display scale on Windows, 1 on a Mac.
    pub scale: f64,
}

/// The panel, in points.
const W: f64 = 344.0;
const HEAD: f64 = 66.0;
const ROW: f64 = 48.0;
const FOOT: f64 = 36.0;
const SEARCH: f64 = 40.0;
const MAX_H: f64 = 580.0;
/// Away from the screen's edges, so the very corner stays free for what's there.
const GAP: f64 = 10.0;
/// How far the pointer may wander off the panel before it goes.
const SLACK: f64 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum How {
    /// The pointer rested in the hot spot.
    Spot,
    /// A shortcut: it has the keyboard.
    Keys,
    /// "Try it" in the settings: shown at the spot until the pointer has had a look.
    Try,
}

impl How {
    fn name(self) -> &'static str {
        match self {
            How::Spot => "spot",
            How::Keys => "keys",
            How::Try => "try",
        }
    }
}

struct State {
    settings: SwitcherSettings,
    open: bool,
    how: How,
    /// Where the panel is (system units).
    panel: Rect,
    /// The spot it opened at ("center" for a shortcut) and the pointer then.
    at: String,
    anchor: (f64, f64),
    opened: Instant,
    /// The pointer has been over the panel since it opened.
    visited: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    gen: u64,
    how: &'static str,
    /// The spot it opened at ("top-left" …, or "center"): it grows out of there.
    at: String,
    items: Vec<SwitchItem>,
    /// Icons already known, by `icon_key`.
    icons: HashMap<String, String>,
    platform: &'static str,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
static PAYLOAD: Mutex<Option<Payload>> = Mutex::new(None);
static ICONS: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);
static GEN: AtomicU64 = AtomicU64::new(0);
static READY: AtomicU64 = AtomicU64::new(0);
static BUILDING: AtomicBool = AtomicBool::new(false);
static OPENING: AtomicBool = AtomicBool::new(false);

const PLATFORM: &str = if cfg!(target_os = "macos") { "mac" } else { "windows" };

fn app_handle() -> Option<AppHandle> {
    APP.lock().ok()?.clone()
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    let mut g = STATE.lock().ok()?;
    g.as_mut().map(f)
}

fn icon_key(it: &SwitchItem) -> String {
    if it.path.is_empty() {
        format!("w:{}", it.id)
    } else {
        it.path.to_lowercase()
    }
}

/// The panel's height in points: a row per window, and the search box when it has the keys.
fn height_for(rows: usize, how: How) -> f64 {
    let search = if how == How::Keys { SEARCH } else { 0.0 };
    geo::panel_height(rows, HEAD + search, ROW, FOOT, MAX_H)
}

// ------------------------------------------------------------------ start, settings, window

/// Called once at start-up.
pub fn start(app: &AppHandle, s: &SwitcherSettings) {
    *APP.lock().unwrap() = Some(app.clone());
    *STATE.lock().unwrap() = Some(State {
        settings: s.clone(),
        open: false,
        how: How::Spot,
        panel: Rect::new(0.0, 0.0, 0.0, 0.0),
        at: String::new(),
        anchor: (0.0, 0.0),
        opened: Instant::now(),
        visited: false,
    });
    if s.on {
        prepare(app);
    }
    std::thread::Builder::new().name("catkey-switch".into()).spawn(watch).ok();
}

/// Settings changed in the main window.
pub fn set_settings(s: &SwitcherSettings) {
    let turned_on = with(|st| {
        let was = st.settings.on;
        st.settings = s.clone();
        s.on && !was
    })
    .unwrap_or(false);
    if turned_on {
        if let Some(app) = app_handle() {
            prepare(&app);
        }
    }
}

/// Make the window ahead of time, so the first pop-up is as quick as the rest.
fn prepare(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        build(&app);
    });
}

fn build(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window("switch") {
        return Some(w);
    }
    if BUILDING.swap(true, Ordering::SeqCst) {
        // Someone else is making it: wait for them.
        for _ in 0..80 {
            std::thread::sleep(Duration::from_millis(50));
            if let Some(w) = app.get_webview_window("switch") {
                return Some(w);
            }
        }
        return None;
    }
    let b = WebviewWindowBuilder::new(app, "switch", WebviewUrl::App("switch.html".into()))
        .title("Switch")
        .inner_size(W, 300.0)
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
    // The first click on the panel picks a window (it doesn't just wake Cat Key up).
    #[cfg(target_os = "macos")]
    let b = b.accept_first_mouse(true);
    let made = b.build();
    BUILDING.store(false, Ordering::SeqCst);
    let w = made.map_err(|e| crate::log(&format!("switch window: {e}"))).ok()?;
    #[cfg(windows)]
    crate::dock_drop::tool_window(&w);
    #[cfg(target_os = "macos")]
    let _ = w.set_visible_on_all_workspaces(true);
    w.on_window_event(|e| {
        // Opened with a shortcut: clicking anywhere else puts it away.
        if let tauri::WindowEvent::Focused(false) = e {
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(150));
                let keys = with(|s| s.open && s.how == How::Keys).unwrap_or(false);
                let focused = app_handle()
                    .and_then(|a| a.get_webview_window("switch"))
                    .and_then(|w| w.is_focused().ok())
                    .unwrap_or(false);
                if keys && !focused {
                    hide_panel();
                }
            });
        }
    });
    // Let the page load before the first list arrives.
    std::thread::sleep(Duration::from_millis(350));
    crate::log("switch: window ready");
    Some(w)
}

fn set_rect(w: &WebviewWindow, r: Rect) {
    #[cfg(windows)]
    {
        // Move first, so the size is taken on the screen it ends up on.
        let pos = tauri::PhysicalPosition::new(r.x.round() as i32, r.y.round() as i32);
        let _ = w.set_position(pos);
        let _ = w.set_size(tauri::PhysicalSize::new(r.w.round() as u32, r.h.round() as u32));
        let _ = w.set_position(pos);
    }
    #[cfg(not(windows))]
    {
        let _ = w.set_position(tauri::LogicalPosition::new(r.x, r.y));
        let _ = w.set_size(tauri::LogicalSize::new(r.w, r.h));
    }
}

fn show(w: &WebviewWindow, focus: bool) {
    #[cfg(windows)]
    {
        if focus {
            let _ = w.show();
            let _ = w.set_focus();
        } else {
            crate::coach::show_quietly(w);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = w.show();
        if focus {
            let _ = w.set_focus();
        }
    }
}

/// The part of the screen free of the taskbar (Windows) or the menu bar and Dock (Mac).
fn work_area(app: &AppHandle, scr: &Screen, px: f64, py: f64) -> Rect {
    #[cfg(windows)]
    {
        let _ = (app, px, py);
        scr.work
    }
    #[cfg(not(windows))]
    {
        if let Ok(mons) = app.available_monitors() {
            for m in mons {
                let k = m.scale_factor().max(0.5);
                let (p, s) = (m.position(), m.size());
                let full = Rect::new(p.x as f64 / k, p.y as f64 / k, s.width as f64 / k, s.height as f64 / k);
                if full.contains(px, py) {
                    let wa = m.work_area();
                    return Rect::new(
                        wa.position.x as f64 / k,
                        wa.position.y as f64 / k,
                        wa.size.width as f64 / k,
                        wa.size.height as f64 / k,
                    );
                }
            }
        }
        // No monitor said: keep clear of the menu bar.
        let f = scr.work;
        Rect::new(f.x, f.y + 25.0, f.w, f.h - 25.0)
    }
}

// ------------------------------------------------------------------ open and close

/// Open it (from any thread but the main one: it waits a moment for the page).
fn open(app: &AppHandle, how: How) {
    if OPENING.swap(true, Ordering::SeqCst) {
        return;
    }
    open_inner(app, how);
    OPENING.store(false, Ordering::SeqCst);
}

fn open_inner(app: &AppHandle, how: How) {
    let Some(spot) = with(|s| s.settings.spot.clone()) else { return };
    let Some((px, py)) = crate::win::pointer() else { return };
    let Some(scr) = crate::win::screen_at(px, py) else { return };
    let at = if how == How::Keys || !geo::is_spot(&spot) { "center".to_string() } else { spot };
    let work = work_area(app, &scr, px, py);
    let items = crate::win::switch_windows();
    let k = scr.scale;
    let h = height_for(items.len(), how) * k;
    let rect = geo::panel_rect(&at, work, px, py, W * k, h, GAP * k);
    let gen = GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let (icons, missing) = known_icons(&items);
    let payload = Payload { gen, how: how.name(), at: at.clone(), items, icons, platform: PLATFORM };
    if let Ok(mut p) = PAYLOAD.lock() {
        *p = Some(payload.clone());
    }
    with(|s| {
        s.open = true;
        s.how = how;
        s.panel = rect;
        s.at = at.clone();
        s.anchor = (px, py);
        s.opened = Instant::now();
        s.visited = false;
    });
    let Some(w) = build(app) else {
        with(|s| s.open = false);
        return;
    };
    set_rect(&w, rect);
    let _ = w.emit("ck://switch-open", payload);
    // Wait for the page to draw the new list, so the last one never flashes up.
    let t = Instant::now();
    while READY.load(Ordering::SeqCst) < gen && t.elapsed() < Duration::from_millis(180) {
        std::thread::sleep(Duration::from_millis(4));
    }
    if !with(|s| s.open).unwrap_or(false) || GEN.load(Ordering::SeqCst) != gen {
        return;
    }
    show(&w, how == How::Keys);
    if !missing.is_empty() {
        fetch_icons(missing);
    }
}

/// The list again (after closing a window), keeping the panel where it is.
fn refresh() {
    let Some(app) = app_handle() else { return };
    let Some((open, how, at, (px, py), panel)) = with(|s| (s.open, s.how, s.at.clone(), s.anchor, s.panel)) else { return };
    if !open {
        return;
    }
    let Some(scr) = crate::win::screen_at(panel.x + 1.0, panel.y + 1.0) else { return };
    let work = work_area(&app, &scr, px, py);
    let items = crate::win::switch_windows();
    let k = scr.scale;
    let h = height_for(items.len(), how) * k;
    let rect = geo::panel_rect(&at, work, px, py, W * k, h, GAP * k);
    let (icons, missing) = known_icons(&items);
    let gen = GEN.load(Ordering::SeqCst);
    let payload = Payload { gen, how: how.name(), at, items, icons, platform: PLATFORM };
    if let Ok(mut p) = PAYLOAD.lock() {
        *p = Some(payload.clone());
    }
    with(|s| s.panel = rect);
    if let Some(w) = app.get_webview_window("switch") {
        let _ = w.emit("ck://switch-items", payload);
        std::thread::sleep(Duration::from_millis(30));
        set_rect(&w, rect);
    }
    if !missing.is_empty() {
        fetch_icons(missing);
    }
}

fn hide_panel() {
    let Some(app) = app_handle() else { return };
    let was = with(|s| std::mem::replace(&mut s.open, false)).unwrap_or(false);
    if !was {
        return;
    }
    let Some(w) = app.get_webview_window("switch") else { return };
    let _ = w.emit("ck://switch-close", ());
    std::thread::spawn(move || {
        // Let it fade, unless it opened again meanwhile.
        std::thread::sleep(Duration::from_millis(120));
        if !with(|s| s.open).unwrap_or(false) {
            let _ = w.hide();
        }
    });
}

/// From a shortcut (`catkey://switch`): open it with the keyboard, or put it away.
pub fn from_shortcut() {
    let Some(app) = app_handle() else { return };
    if with(|s| s.open).unwrap_or(false) {
        hide_panel();
        return;
    }
    std::thread::spawn(move || open(&app, How::Keys));
}

// ------------------------------------------------------------------ icons

fn known_icons(items: &[SwitchItem]) -> (HashMap<String, String>, Vec<(String, String, String)>) {
    let mut known = HashMap::new();
    let mut missing = Vec::new();
    let g = ICONS.lock().ok();
    let cache = g.as_ref().and_then(|g| g.as_ref());
    for it in items {
        let key = icon_key(it);
        match cache.and_then(|c| c.get(&key)) {
            Some(Some(url)) => {
                known.insert(key, url.clone());
            }
            Some(None) => {}
            None => {
                if !missing.iter().any(|(k, _, _)| *k == key) {
                    missing.push((key, it.path.clone(), it.id.clone()));
                }
            }
        }
    }
    (known, missing)
}

fn fetch_icons(missing: Vec<(String, String, String)>) {
    std::thread::spawn(move || {
        let mut got = HashMap::new();
        for (key, path, id) in missing {
            let icon = if path.is_empty() { crate::win::window_icon(&id) } else { crate::win::icon_data_url(&path) };
            if let Ok(mut g) = ICONS.lock() {
                let m = g.get_or_insert_with(HashMap::new);
                if m.len() > 600 {
                    m.clear();
                }
                m.insert(key.clone(), icon.clone());
            }
            if let Some(url) = icon {
                got.insert(key, url);
            }
        }
        if got.is_empty() {
            return;
        }
        if let Ok(mut p) = PAYLOAD.lock() {
            if let Some(p) = p.as_mut() {
                p.icons.extend(got.clone());
            }
        }
        if let Some(w) = app_handle().and_then(|a| a.get_webview_window("switch")) {
            let _ = w.emit("ck://switch-icons", got);
        }
    });
}

// ------------------------------------------------------------------ the pointer watcher

fn watch() {
    let mut dwell = Dwell::default();
    let mut away: Option<Instant> = None;
    let t0 = Instant::now();
    loop {
        let Some((s, is_open, how, panel, opened, visited)) =
            with(|st| (st.settings.clone(), st.open, st.how, st.panel, st.opened, st.visited))
        else {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        };
        if !s.on && !(is_open && how != How::Keys) {
            // Nothing to watch: look again now and then, in case it gets turned on.
            dwell = Dwell::default();
            std::thread::sleep(Duration::from_millis(400));
            continue;
        }
        std::thread::sleep(Duration::from_millis(if is_open { 30 } else { 45 }));
        let Some((px, py)) = crate::win::pointer() else { continue };
        let Some(scr) = crate::win::screen_at(px, py) else { continue };
        let k = scr.scale.max(1.0);
        let hot = s.on && geo::in_spot(&s.spot, scr.full, px, py, 2.0 * k);
        if is_open {
            if hot {
                dwell.spend();
            }
            if how == How::Keys {
                continue;
            }
            let over = panel.grow(SLACK * k).contains(px, py);
            if over && !visited {
                with(|st| st.visited = true);
            }
            // "Try it": wait a few seconds for the pointer to come and have a look.
            let waiting = how == How::Try && !visited && opened.elapsed() < Duration::from_millis(4000);
            if geo::still_there(panel, SLACK * k, hot && how == How::Spot, px, py) || waiting {
                away = None;
            } else if away.get_or_insert_with(Instant::now).elapsed() >= Duration::from_millis(280) {
                away = None;
                hide_panel();
            }
            continue;
        }
        away = None;
        let now = t0.elapsed().as_millis() as u64;
        let down = hot && crate::win::mouse_down();
        if dwell.step(hot, down, now, s.delay_ms as u64) {
            // Out of the way of games and films.
            #[cfg(windows)]
            if unsafe { crate::coach::full_screen_app() } {
                continue;
            }
            if let Some(app) = app_handle() {
                open(&app, How::Spot);
            }
        }
    }
}

// ------------------------------------------------------------------ commands

/// The page (re)loaded: what to show.
#[tauri::command]
pub fn switch_state() -> Option<Payload> {
    PAYLOAD.lock().ok()?.clone()
}

/// The page drew list `gen`.
#[tauri::command]
pub fn switch_ready(gen: u64) {
    READY.fetch_max(gen, Ordering::SeqCst);
}

/// Jump to a window (or app).
#[tauri::command]
pub async fn switch_to(id: String, reopen: bool) -> bool {
    let ok = crate::win::switch_to(&id, reopen);
    hide_panel();
    ok
}

/// Close a window (Mac: quit the app), then show the list again.
#[tauri::command]
pub async fn switch_close_window(id: String) -> bool {
    let ok = crate::win::switch_close(&id);
    std::thread::sleep(Duration::from_millis(400));
    refresh();
    ok
}

#[tauri::command]
pub fn switch_hide() {
    hide_panel();
}

/// "Try it" in the settings: show it at the chosen spot.
#[tauri::command]
pub fn switch_try(app: AppHandle) {
    std::thread::spawn(move || open(&app, How::Try));
}

/// For the start-up check: list the windows and open the panel once.
pub fn self_test(app: &AppHandle) {
    let n = crate::win::switch_windows().len();
    crate::log(&format!("switch: {n} windows to switch to"));
    let app = app.clone();
    std::thread::spawn(move || {
        open(&app, How::Try);
        let shown = app.get_webview_window("switch").and_then(|w| w.is_visible().ok()).unwrap_or(false);
        crate::log(&format!("switch: panel shown {shown}"));
    });
}
