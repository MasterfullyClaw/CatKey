//! The shortcut coach: notices when something that has a shortcut was done the long
//! way, and shows a short tip that fades away. It never looks at the screen and never
//! keeps what you type.
//!
//! * **Clicks.** Raw mouse input on the coach's own thread (no hook, so nothing ever
//!   waits on us). When the left button goes down it asks Windows what is under the
//!   pointer: a window's ✕ – □ buttons by hit test, everything else through UI
//!   Automation, the description apps give screen readers (a name, a type and often
//!   the shortcut itself). Chrome-based apps build that description only when asked,
//!   which costs them some work, so they are left alone unless the user opts in.
//! * **Key habits.** The keyboard thread counts runs of the same editing key and
//!   posts a candidate here (Backspace ×10 → Ctrl+Backspace deletes a word).
//! * **Shortcut use.** The keyboard thread counts each real modifier+key press per
//!   app. Those counts are the "time saved" side of the stats.

use crate::win;
use catkey_core::coachtext::{clean_name, common_keys, keys_in, looks_like_keys, tidy_keys};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetClassNameW, GetCursorPos, GetForegroundWindow,
    GetGUIThreadInfo, GetMessageW, GetWindowLongPtrW, GetWindowLongW, GetWindowRect, GetWindowThreadProcessId, IsZoomed,
    PeekMessageW, PostThreadMessageW, RegisterClassW, SendMessageTimeoutW, SetTimer, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, WindowFromPoint, GA_ROOT, GUITHREADINFO, GWL_EXSTYLE, GWL_STYLE, HWND_MESSAGE, HWND_TOPMOST, MSG,
    PM_NOREMOVE, SMTO_ABORTIFHUNG, SMTO_BLOCK, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
    SW_SHOWNOACTIVATE, WM_APP, WM_INPUT, WM_NCHITTEST, WM_TIMER, WM_USER, WNDCLASSW, WS_CAPTION, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};

const WM_COACH_WAKE: u32 = WM_APP + 0xC0A;
const TIMER_FLUSH: usize = 1;
/// How often counts from the keyboard thread are folded into the stats.
const FLUSH_MS: u32 = 30_000;
/// Saved at most this often (and when Cat Key quits).
const SAVE_EVERY_MS: u64 = 120_000;
/// Days of stats kept.
const KEEP_DAYS: usize = 120;
/// The same tip again only after this long.
const SAME_TIP_MS: u64 = 30 * 60_000;
/// At least this long between two tips, and at most this many tips an hour.
const GAP_MS: u64 = 8_000;
const PER_HOUR: usize = 10;
/// How long a tip stays up (the page fades it out just before).
/// The tip hides itself (it waits while the pointer is on it); this is only the backstop.
const SHOW_MS: u64 = 45_000;

/// Time the long way costs, from the Keystroke-Level Model (Card, Moran & Newell):
/// reaching for the mouse and back (0.4 s each), pointing (1.1 s) and clicking (0.2 s),
/// minus the shortcut itself (about 0.5 s for two or three keys).
const LOST_MENU_MS: u64 = 2_900; // open a menu, then pick the item
const LOST_BUTTON_MS: u64 = 1_600; // one button
/// Time a shortcut saves, counted the cautious way: as if it replaced one button.
pub const SAVED_PER_SHORTCUT_MS: u64 = LOST_BUTTON_MS;

// ------------------------------------------------------------------ settings

#[derive(Clone, Default)]
pub struct CoachSettings {
    pub on: bool,
    pub chromium: bool,
    pub muted: Vec<String>,
    pub excluded: Vec<String>,
    pub pos: Option<catkey_core::config::CoachPos>,
    /// A soft mrrp when a tip pops up (Settings → Sounds → Coach tips; off unless chosen).
    pub tip_sound: bool,
}

static SETTINGS: Mutex<CoachSettings> =
    Mutex::new(CoachSettings { on: false, chromium: false, muted: Vec::new(), excluded: Vec::new(), pos: None, tip_sound: false });

pub fn set_settings(s: &catkey_core::Settings) {
    if let Ok(mut g) = SETTINGS.lock() {
        g.on = s.coach;
        g.chromium = s.coach_chromium;
        g.muted = s.coach_muted.iter().map(|m| m.to_lowercase()).collect();
        g.excluded = s.excluded_apps.iter().map(|a| a.to_lowercase()).collect();
        g.pos = Some(s.coach_pos.clone());
        g.tip_sound = s.sounds && s.sound_parts.get("tips").copied().unwrap_or(false);
    }
}

fn settings() -> CoachSettings {
    SETTINGS.lock().map(|g| g.clone()).unwrap_or_default()
}

// ------------------------------------------------------------------ what apps call things

/// Names → shortcuts per app ("*" for any app), from the library packs (sent by the UI).
static DICT: Mutex<Option<HashMap<String, HashMap<String, String>>>> = Mutex::new(None);

pub fn set_dict(dir: &Path, dict: HashMap<String, Vec<(String, String)>>) {
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (app, items) in dict {
        let m = map.entry(app.to_lowercase()).or_default();
        for (name, keys) in items {
            let n = clean_name(&name).to_lowercase();
            if !n.is_empty() && !keys.is_empty() {
                m.entry(n).or_insert(keys);
            }
        }
    }
    if let Ok(text) = serde_json::to_string(&map) {
        let _ = std::fs::write(dir.join("coach-dict.json"), text);
    }
    if let Ok(mut g) = DICT.lock() {
        *g = Some(map);
    }
}

fn load_dict(dir: &Path) -> HashMap<String, HashMap<String, String>> {
    std::fs::read_to_string(dir.join("coach-dict.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn dict_lookup(exe: &str, name: &str) -> Option<String> {
    let key = name.to_lowercase();
    if let Ok(g) = DICT.lock() {
        if let Some(d) = g.as_ref() {
            if let Some(k) = d.get(exe).and_then(|m| m.get(&key)) {
                return Some(k.clone());
            }
        }
    }
    common_keys(&key).map(|k| k.to_string())
}

// ------------------------------------------------------------------ stats

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppDay {
    #[serde(default)]
    pub used: u32,
    #[serde(default)]
    pub caught: u32,
    #[serde(default)]
    pub lost_ms: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TipStat {
    pub app: String,
    pub name: String,
    pub keys: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub n: u32,
    #[serde(default)]
    pub last: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoachStats {
    /// Day ("2026-11-26") → app exe → counts.
    #[serde(default)]
    pub days: BTreeMap<String, HashMap<String, AppDay>>,
    /// Things done the long way, by tip id.
    #[serde(default)]
    pub tips: HashMap<String, TipStat>,
    #[serde(default)]
    pub saved_per_shortcut_ms: u64,
    #[serde(skip)]
    dirty: bool,
}

static STATS: Mutex<Option<CoachStats>> = Mutex::new(None);
static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static LAST_SAVE: AtomicU64 = AtomicU64::new(0);
/// Real modifier+key presses per app since the last flush (from the keyboard thread).
static COMBOS: Mutex<Vec<(String, u32)>> = Mutex::new(Vec::new());
/// Key-habit candidates: (vk, modifiers, presses in a row).
static HABITS: Mutex<Vec<(u8, u8, u32)>> = Mutex::new(Vec::new());

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn ticks() -> u64 {
    unsafe { GetTickCount64() }
}

fn own_exe() -> &'static str {
    static OWN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OWN.get_or_init(|| {
        std::env::current_exe().ok().and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_lowercase())).unwrap_or_default()
    })
}

/// From the keyboard thread: never blocks.
pub fn count_combo(exe: &str) {
    if exe.is_empty() || exe == own_exe() {
        return;
    }
    if let Ok(mut c) = COMBOS.try_lock() {
        if let Some(e) = c.iter_mut().find(|(a, _)| a == exe) {
            e.1 += 1;
        } else if c.len() < 256 {
            c.push((exe.to_string(), 1));
        }
    }
}

/// From the keyboard thread: a run of the same editing key. Never blocks.
pub fn habit(vk: u8, mods: u8, presses: u32) {
    if let Ok(mut q) = HABITS.try_lock() {
        if q.len() < 8 {
            q.push((vk, mods, presses));
        }
    }
    wake();
}

fn wake() {
    let t = TID.load(Ordering::SeqCst);
    if t != 0 {
        unsafe { PostThreadMessageW(t, WM_COACH_WAKE, 0, 0) };
    }
}

fn load_stats(dir: &Path) -> CoachStats {
    std::fs::read_to_string(dir.join("coach.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Fold in the counts from the keyboard thread; save if it's time (or `force`).
pub fn flush(force: bool) {
    let combos: Vec<(String, u32)> = COMBOS.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default();
    let snapshot = {
        let Ok(mut g) = STATS.lock() else { return };
        let Some(s) = g.as_mut() else { return };
        if !combos.is_empty() {
            let day = s.days.entry(win::local_date()).or_default();
            for (exe, n) in combos {
                day.entry(exe).or_default().used += n;
            }
            s.dirty = true;
        }
        while s.days.len() > KEEP_DAYS {
            let first = s.days.keys().next().cloned();
            if let Some(k) = first {
                s.days.remove(&k);
            }
        }
        if s.tips.len() > 400 {
            // keep the most frequent
            let mut v: Vec<(String, u32)> = s.tips.iter().map(|(k, t)| (k.clone(), t.n)).collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            for (k, _) in v.into_iter().skip(300) {
                s.tips.remove(&k);
            }
        }
        let due = force || ticks().saturating_sub(LAST_SAVE.load(Ordering::Relaxed)) >= SAVE_EVERY_MS;
        if !s.dirty || !due {
            return;
        }
        s.dirty = false;
        s.saved_per_shortcut_ms = SAVED_PER_SHORTCUT_MS;
        s.clone()
    };
    LAST_SAVE.store(ticks(), Ordering::Relaxed);
    if crate::app::NO_SAVE.load(Ordering::SeqCst) {
        return;
    }
    let dir = DIR.lock().ok().and_then(|d| d.clone());
    if let (Some(dir), Ok(text)) = (dir, serde_json::to_string(&snapshot)) {
        let tmp = dir.join("coach.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, dir.join("coach.json"));
        }
    }
}

/// Everything the UI shows, with the latest counts folded in.
pub fn stats_json() -> serde_json::Value {
    flush(false);
    let s = STATS.lock().ok().and_then(|g| g.clone()).unwrap_or_default();
    let mut v = serde_json::to_value(&s).unwrap_or_default();
    v["savedPerShortcutMs"] = SAVED_PER_SHORTCUT_MS.into();
    v["lostMenuMs"] = LOST_MENU_MS.into();
    v["lostButtonMs"] = LOST_BUTTON_MS.into();
    v
}

pub fn forget_tip(id: &str) {
    if let Ok(mut g) = STATS.lock() {
        if let Some(s) = g.as_mut() {
            s.tips.remove(id);
            s.dirty = true;
        }
    }
}

// ------------------------------------------------------------------ tips

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Menu,
    Button,
    Window,
    Habit,
}

#[derive(Clone)]
struct Tip {
    id: String,
    app: String,
    name: String,
    keys: String,
    kind: Kind,
    lost_ms: u64,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TipPayload {
    pub id: String,
    pub app: String,
    pub app_name: String,
    pub name: String,
    pub keys: String,
    pub saves_ms: u64,
    pub times: u32,
    pub seq: u32,
    pub demo: bool,
    /// Where on the screen it shows ("bottom-right", "top-center"…), so it slides in from that side.
    pub anchor: String,
    /// Whether the tip makes its little sound.
    pub sound: bool,
}

static LAST_TIP: Mutex<Option<TipPayload>> = Mutex::new(None);
static SEQ: AtomicU32 = AtomicU32::new(0);
static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
static TID: AtomicU32 = AtomicU32::new(0);

pub fn last_tip() -> Option<TipPayload> {
    LAST_TIP.lock().ok().and_then(|g| g.clone())
}

fn app_name(exe: &str) -> String {
    let known = [
        ("explorer.exe", "File Explorer"),
        ("chrome.exe", "Chrome"),
        ("msedge.exe", "Edge"),
        ("firefox.exe", "Firefox"),
        ("code.exe", "VS Code"),
        ("notepad.exe", "Notepad"),
        ("winword.exe", "Word"),
        ("excel.exe", "Excel"),
        ("powerpnt.exe", "PowerPoint"),
        ("outlook.exe", "Outlook"),
        ("adobe premiere pro.exe", "Premiere Pro"),
        ("photoshop.exe", "Photoshop"),
        ("illustrator.exe", "Illustrator"),
        ("afterfx.exe", "After Effects"),
        ("resolve.exe", "DaVinci Resolve"),
        ("blender.exe", "Blender"),
        ("figma.exe", "Figma"),
        ("obsidian.exe", "Obsidian"),
        ("capcut.exe", "CapCut"),
        ("lightroom.exe", "Lightroom Classic"),
        ("systemsettings.exe", "Settings"),
    ];
    if let Some((_, n)) = known.iter().find(|(e, _)| *e == exe) {
        return n.to_string();
    }
    let base = exe.strip_suffix(".exe").unwrap_or(exe);
    let mut c = base.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn record(tip: &Tip) -> u32 {
    let Ok(mut g) = STATS.lock() else { return 0 };
    let Some(s) = g.as_mut() else { return 0 };
    let day = s.days.entry(win::local_date()).or_default().entry(tip.app.clone()).or_default();
    day.caught += 1;
    day.lost_ms += tip.lost_ms;
    let kind = match tip.kind {
        Kind::Menu => "menu",
        Kind::Button => "button",
        Kind::Window => "window",
        Kind::Habit => "habit",
    };
    let t = s.tips.entry(tip.id.clone()).or_insert_with(|| TipStat {
        app: tip.app.clone(),
        name: tip.name.clone(),
        keys: tip.keys.clone(),
        kind: kind.into(),
        n: 0,
        last: 0,
    });
    t.n += 1;
    t.last = now_secs();
    t.keys = tip.keys.clone();
    s.dirty = true;
    t.n
}

/// Show a sample tip (the Settings button and tests).
pub fn demo() {
    let p = TipPayload {
        id: "demo".into(),
        app: "explorer.exe".into(),
        app_name: "File Explorer".into(),
        name: "Back".into(),
        keys: "Alt+Left".into(),
        saves_ms: LOST_BUTTON_MS,
        times: 1,
        seq: 0,
        demo: true,
        anchor: String::new(),
        sound: false,
    };
    show(p);
}

/// The tip's own buttons, the window's Dismiss and the timer all end here.
pub fn hide() {
    let Some(app) = APP.lock().ok().and_then(|g| g.clone()) else { return };
    std::thread::spawn(move || {
        if let Some(w) = app.get_webview_window("coach") {
            if let Ok(h) = w.hwnd() {
                unsafe { ShowWindow(h.0 as HWND, SW_HIDE) };
            }
        }
    });
}

/// What a tip was about, for Adjust (the last tip, or one from the list of catches).
pub fn tip_info(id: &str) -> Option<TipPayload> {
    if let Some(t) = last_tip().filter(|t| t.id == id) {
        return Some(t);
    }
    let g = STATS.lock().ok()?;
    let t = g.as_ref()?.tips.get(id)?.clone();
    Some(TipPayload { id: id.to_string(), app_name: app_name(&t.app), app: t.app, name: t.name, keys: t.keys, ..Default::default() })
}

fn show(mut p: TipPayload) {
    let Some(app) = APP.lock().ok().and_then(|g| g.clone()) else { return };
    let seq = SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    p.seq = seq;
    let s = settings();
    p.anchor = s.pos.map(|q| q.anchor).unwrap_or_else(|| "bottom-right".into());
    p.sound = s.tip_sound;
    if let Ok(mut g) = LAST_TIP.lock() {
        *g = Some(p.clone());
    }
    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) };
    // Windows are made and moved from their own thread (never from a window callback).
    std::thread::spawn(move || {
        let w = match app.get_webview_window("coach") {
            Some(w) => w,
            None => match build(&app) {
                Some(w) => w,
                None => return,
            },
        };
        place(&w, pt);
        let _ = app.emit_to("coach", "ck://coach-tip", &p);
        if crate::app::UI_OPEN.load(Ordering::Relaxed) {
            let _ = app.emit_to("main", "ck://coach-tip", &p);
        }
        show_quietly(&w);
        std::thread::sleep(Duration::from_millis(SHOW_MS));
        if SEQ.load(Ordering::SeqCst) == seq {
            if let Ok(h) = w.hwnd() {
                unsafe { ShowWindow(h.0 as HWND, SW_HIDE) };
            }
        }
    });
}

fn build(app: &AppHandle) -> Option<WebviewWindow> {
    let w = WebviewWindowBuilder::new(app, "coach", WebviewUrl::App("coach.html".into()))
        .title("Cat Key tip")
        .inner_size(TIP_W, TIP_H)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|e| crate::log(&format!("coach window: {e}")))
        .ok()?;
    // The tip has buttons now (Ignore, Dismiss, Adjust), so it takes clicks, but it
    // never takes the keyboard from the app you're in (no activation).
    if let Ok(h) = w.hwnd() {
        unsafe {
            let hwnd = h.0 as HWND;
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize);
        }
    }
    // Give the page a moment to load; it also asks for the latest tip when it's ready.
    std::thread::sleep(Duration::from_millis(600));
    Some(w)
}

const TIP_W: f64 = 440.0;
const TIP_H: f64 = 156.0;

/// Where the user wants tips (bottom right unless chosen), on the screen the pointer is on.
fn place(w: &WebviewWindow, pt: POINT) {
    unsafe {
        let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(mon, &mut mi) == 0 {
            return;
        }
        let scale = w
            .monitor_from_point(pt.x as f64, pt.y as f64)
            .ok()
            .flatten()
            .map(|m| m.scale_factor())
            .unwrap_or(1.0);
        let (cw, ch) = ((TIP_W * scale) as i32, (TIP_H * scale) as i32);
        let margin = (10.0 * scale) as i32;
        let work = mi.rcWork;
        let area = (work.left, work.top, work.right - work.left, work.bottom - work.top);
        let pos = settings().pos.unwrap_or_default();
        let (x, y) = pos.place(area, cw, ch, margin);
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
        let _ = w.set_size(tauri::PhysicalSize::new(cw as u32, ch as u32));
    }
}

fn show_quietly(w: &WebviewWindow) {
    if let Ok(h) = w.hwnd() {
        unsafe {
            let hwnd = h.0 as HWND;
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW);
        }
    }
}

/// A game or a video playing full screen: count, but don't pop up.
unsafe fn full_screen_app() -> bool {
    let fg = GetForegroundWindow();
    if fg.is_null() {
        return false;
    }
    let class = class_of(fg);
    if matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd") {
        return false;
    }
    let mut r: RECT = zeroed();
    if GetWindowRect(fg, &mut r) == 0 {
        return false;
    }
    let mut mi: MONITORINFO = zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST), &mut mi) == 0 {
        return false;
    }
    let covers = r.left <= mi.rcMonitor.left && r.top <= mi.rcMonitor.top && r.right >= mi.rcMonitor.right && r.bottom >= mi.rcMonitor.bottom;
    covers && (GetWindowLongW(fg, GWL_STYLE) as u32 & WS_CAPTION) != WS_CAPTION
}

unsafe fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

// ------------------------------------------------------------------ the coach thread

struct Coach {
    uia: Option<Uia>,
    uia_failed: bool,
    /// When tips were shown (ms), newest last.
    shown: VecDeque<u64>,
    /// When each tip was last shown.
    shown_tip: HashMap<String, u64>,
}

thread_local! {
    static COACH: std::cell::RefCell<Option<Coach>> = const { std::cell::RefCell::new(None) };
}

pub fn start(app: AppHandle, dir: PathBuf, s: &catkey_core::Settings) {
    set_settings(s);
    if let Ok(mut g) = DIR.lock() {
        *g = Some(dir.clone());
    }
    if let Ok(mut g) = STATS.lock() {
        *g = Some(load_stats(&dir));
    }
    if let Ok(mut g) = DICT.lock() {
        *g = Some(load_dict(&dir));
    }
    if let Ok(mut g) = APP.lock() {
        *g = Some(app);
    }
    LAST_SAVE.store(ticks(), Ordering::Relaxed);
    let _ = std::thread::Builder::new().name("catkey-coach".into()).spawn(|| unsafe { run() });
}

unsafe fn run() {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let mut msg: MSG = zeroed();
    PeekMessageW(&mut msg, null_mut(), WM_USER, WM_USER, PM_NOREMOVE);
    COACH.with(|c| {
        *c.borrow_mut() = Some(Coach { uia: None, uia_failed: false, shown: VecDeque::new(), shown_tip: HashMap::new() })
    });
    TID.store(GetCurrentThreadId(), Ordering::SeqCst);

    let class = win::wide("CatKeyCoach");
    let hinst = GetModuleHandleW(null());
    let mut wc: WNDCLASSW = zeroed();
    wc.lpfnWndProc = Some(proc_);
    wc.hInstance = hinst;
    wc.lpszClassName = class.as_ptr();
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(0, class.as_ptr(), null(), 0, 0, 0, 0, 0, HWND_MESSAGE, null_mut(), hinst, null());
    if hwnd.is_null() {
        crate::log("coach: no window");
        return;
    }
    SetTimer(hwnd, TIMER_FLUSH, FLUSH_MS, None);
    // Mouse (usage page 1, usage 2), even when another app has the focus.
    let rid = RAWINPUTDEVICE { usUsagePage: 0x01, usUsage: 0x02, dwFlags: RIDEV_INPUTSINK, hwndTarget: hwnd };
    if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
        crate::log("coach: couldn't listen to the mouse");
    }

    loop {
        let r = GetMessageW(&mut msg, null_mut(), 0, 0);
        if r == 0 || r == -1 {
            break;
        }
        if msg.hwnd.is_null() && msg.message == WM_COACH_WAKE {
            on_habits();
            continue;
        }
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn proc_(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_INPUT => on_input(lparam),
        WM_TIMER => {
            flush(false);
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn on_input(lparam: LPARAM) {
    let mut ri: RAWINPUT = zeroed();
    let mut size = size_of::<RAWINPUT>() as u32;
    let got = GetRawInputData(lparam as HRAWINPUT, RID_INPUT, &mut ri as *mut RAWINPUT as *mut _, &mut size, size_of::<RAWINPUTHEADER>() as u32);
    if got == u32::MAX || got == 0 || ri.header.dwType != 0 {
        return; // 0 = RIM_TYPEMOUSE
    }
    let flags = ri.data.mouse.Anonymous.Anonymous.usButtonFlags;
    if flags & 0x0001 != 0 {
        // RI_MOUSE_LEFT_BUTTON_DOWN: ask now, while a menu item is still open under the pointer.
        on_click();
    }
}

unsafe fn on_click() {
    let s = settings();
    if !s.on {
        return;
    }
    let mut pt = POINT { x: 0, y: 0 };
    if GetCursorPos(&mut pt) == 0 {
        return;
    }
    let hwnd = WindowFromPoint(pt);
    if hwnd.is_null() {
        return;
    }
    let root = GetAncestor(hwnd, GA_ROOT);
    let root = if root.is_null() { hwnd } else { root };
    let mut pid = 0u32;
    GetWindowThreadProcessId(root, &mut pid);
    if pid == 0 || pid == GetCurrentProcessId() {
        return;
    }
    let exe = win::window_exe(root, pid).unwrap_or_default();
    if exe.is_empty() || s.excluded.iter().any(|e| *e == exe) {
        return;
    }

    // The window's own ✕ – □ buttons: a hit test answers that, in any app.
    let lp = (((pt.y as u16 as u32) << 16) | (pt.x as u16 as u32)) as isize;
    let mut hit: usize = 0;
    if SendMessageTimeoutW(root, WM_NCHITTEST, 0, lp, SMTO_ABORTIFHUNG | SMTO_BLOCK, 80, &mut hit) != 0 {
        let w = match hit as u32 {
            20 => Some(("close", "Close the window", "Alt+F4")),
            8 => Some(("min", "Minimize", "Win+Down")),
            9 if IsZoomed(root) != 0 => Some(("restore", "Restore down", "Win+Down")),
            9 => Some(("max", "Maximize", "Win+Up")),
            _ => None,
        };
        if let Some((id, name, keys)) = w {
            deliver(Tip {
                id: format!("*|window-{id}"),
                app: exe,
                name: name.into(),
                keys: keys.into(),
                kind: Kind::Window,
                lost_ms: LOST_BUTTON_MS,
            });
            return;
        }
    }

    if class_of(root).starts_with("Chrome_WidgetWin") && !s.chromium {
        return;
    }
    let found = COACH.with(|c| c.borrow_mut().as_mut().and_then(|co| co.lookup(pt)));
    if let Some((name, keys, kind)) = found {
        let clean = clean_name(&name);
        let keys = keys.or_else(|| dict_lookup(&exe, &clean));
        if let Some(keys) = keys {
            if clean.is_empty() || clean.len() > 60 {
                return;
            }
            deliver(Tip {
                id: format!("{exe}|{}", clean.to_lowercase()),
                app: exe,
                name: clean,
                keys,
                kind,
                lost_ms: if kind == Kind::Menu { LOST_MENU_MS } else { LOST_BUTTON_MS },
            });
        }
    }
}

unsafe fn on_habits() {
    let list: Vec<(u8, u8, u32)> = HABITS.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    if list.is_empty() || !settings().on {
        return;
    }
    let fg = GetForegroundWindow();
    let mut pid = 0u32;
    GetWindowThreadProcessId(fg, &mut pid);
    if pid == GetCurrentProcessId() {
        return;
    }
    let exe = win::window_exe(fg, pid).unwrap_or_default();
    if exe.is_empty() || settings().excluded.iter().any(|e| *e == exe) {
        return;
    }
    if !COACH.with(|c| c.borrow_mut().as_mut().map(|co| co.typing_here(fg)).unwrap_or(false)) {
        return; // a game or a list, not text: arrows and Backspace mean something else there
    }
    for (vk, mods, n) in list {
        let shift = mods & (catkey_core::keys::bit::LSHIFT | catkey_core::keys::bit::RSHIFT) != 0;
        let (id, name, keys): (&str, &str, &str) = match (vk, shift) {
            (0x08, _) => ("backspace", "Delete a whole word", "Ctrl+Backspace"),
            (0x2E, _) => ("delete", "Delete the next word", "Ctrl+Delete"),
            (0x25, false) => ("left", "Jump a word to the left", "Ctrl+Left"),
            (0x27, false) => ("right", "Jump a word to the right", "Ctrl+Right"),
            (0x25, true) => ("sel-left", "Select a word at a time", "Ctrl+Shift+Left"),
            (0x27, true) => ("sel-right", "Select a word at a time", "Ctrl+Shift+Right"),
            (0x26, _) => ("up", "Jump a page up", "PageUp"),
            (0x28, _) => ("down", "Jump a page down", "PageDown"),
            _ => continue,
        };
        deliver(Tip {
            id: format!("*|habit-{id}"),
            app: exe.clone(),
            name: name.into(),
            keys: keys.into(),
            kind: Kind::Habit,
            // Each press beyond the few a shortcut takes: one keystroke (0.2 s) of time.
            lost_ms: (n.saturating_sub(3) as u64) * 200,
        });
    }
}

fn deliver(tip: Tip) {
    let times = record(&tip);
    let s = settings();
    let id = tip.id.to_lowercase();
    let app_wide = format!("{}|*", tip.app);
    if s.muted.iter().any(|m| *m == id || *m == app_wide) {
        return;
    }
    let now = ticks();
    let ok = COACH.with(|c| {
        let mut g = c.borrow_mut();
        let Some(co) = g.as_mut() else { return false };
        while co.shown.front().is_some_and(|t| now.saturating_sub(*t) > 3_600_000) {
            co.shown.pop_front();
        }
        if co.shown.len() >= PER_HOUR || co.shown.back().is_some_and(|t| now.saturating_sub(*t) < GAP_MS) {
            return false;
        }
        if co.shown_tip.get(&id).is_some_and(|t| now.saturating_sub(*t) < SAME_TIP_MS) {
            return false;
        }
        co.shown.push_back(now);
        co.shown_tip.insert(id.clone(), now);
        true
    });
    if !ok || unsafe { full_screen_app() } {
        return;
    }
    show(TipPayload {
        id: tip.id.clone(),
        app_name: app_name(&tip.app),
        app: tip.app,
        name: tip.name,
        keys: tip.keys,
        saves_ms: tip.lost_ms,
        times,
        seq: 0,
        demo: false,
        anchor: String::new(),
        sound: false,
    });
}

// ------------------------------------------------------------------ UI Automation

struct Uia {
    auto: windows::Win32::UI::Accessibility::IUIAutomation,
    req: windows::Win32::UI::Accessibility::IUIAutomationCacheRequest,
    walker: windows::Win32::UI::Accessibility::IUIAutomationTreeWalker,
}

impl Coach {
    fn uia(&mut self) -> Option<&Uia> {
        if self.uia.is_none() && !self.uia_failed {
            match unsafe { make_uia() } {
                Ok(u) => self.uia = Some(u),
                Err(e) => {
                    self.uia_failed = true;
                    crate::log(&format!("coach: UI Automation unavailable ({e})"));
                }
            }
        }
        self.uia.as_ref()
    }

    /// What's under the pointer: (name, shortcut if the app says it, kind).
    fn lookup(&mut self, pt: POINT) -> Option<(String, Option<String>, Kind)> {
        use windows::Win32::UI::Accessibility::*;
        let uia = self.uia()?;
        unsafe {
            let mut el = uia.auto.ElementFromPointBuildCache(windows::Win32::Foundation::POINT { x: pt.x, y: pt.y }, &uia.req).ok()?;
            for _ in 0..3 {
                let ct = el.CachedControlType().ok()?;
                let kind = if ct == UIA_MenuItemControlTypeId {
                    Some(Kind::Menu)
                } else if ct == UIA_ButtonControlTypeId || ct == UIA_SplitButtonControlTypeId || ct == UIA_CheckBoxControlTypeId {
                    Some(Kind::Button)
                } else {
                    None
                };
                if let Some(kind) = kind {
                    let name = el.CachedName().map(|b| b.to_string()).unwrap_or_default();
                    let acc = el.CachedAcceleratorKey().map(|b| b.to_string()).unwrap_or_default();
                    let help = el.CachedHelpText().map(|b| b.to_string()).unwrap_or_default();
                    let keys = if looks_like_keys(&acc) { Some(tidy_keys(&acc)) } else { keys_in(&name).or_else(|| keys_in(&help)) };
                    return Some((name, keys, kind));
                }
                // A label or icon inside a button: look at its parent. Anything else isn't a command.
                if !(ct == UIA_TextControlTypeId || ct == UIA_ImageControlTypeId || ct == UIA_CustomControlTypeId) {
                    return None;
                }
                el = uia.walker.GetParentElementBuildCache(&el, &uia.req).ok()?;
            }
        }
        None
    }

    /// Is the keyboard in a text field in this window?
    fn typing_here(&mut self, fg: HWND) -> bool {
        use windows::Win32::UI::Accessibility::*;
        unsafe {
            let tid = GetWindowThreadProcessId(fg, null_mut());
            let mut gti: GUITHREADINFO = zeroed();
            gti.cbSize = size_of::<GUITHREADINFO>() as u32;
            if GetGUIThreadInfo(tid, &mut gti) != 0 && !gti.hwndCaret.is_null() {
                return true;
            }
            if class_of(fg).starts_with("Chrome_WidgetWin") && !settings().chromium {
                return false;
            }
            let Some(uia) = self.uia() else { return false };
            let Ok(el) = uia.auto.GetFocusedElementBuildCache(&uia.req) else { return false };
            matches!(el.CachedControlType(), Ok(ct) if ct == UIA_EditControlTypeId || ct == UIA_DocumentControlTypeId)
        }
    }
}

unsafe fn make_uia() -> windows::core::Result<Uia> {
    use windows::core::Interface;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Accessibility::*;
    let auto: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
        .or_else(|_| CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER))?;
    // A frozen app must not hold the coach up.
    if let Ok(a2) = auto.cast::<IUIAutomation2>() {
        let _ = a2.SetConnectionTimeout(700);
        let _ = a2.SetTransactionTimeout(700);
    }
    let req = auto.CreateCacheRequest()?;
    for p in [UIA_NamePropertyId, UIA_ControlTypePropertyId, UIA_AcceleratorKeyPropertyId, UIA_HelpTextPropertyId] {
        req.AddProperty(p)?;
    }
    let walker = auto.ControlViewWalker()?;
    Ok(Uia { auto, req, walker })
}
