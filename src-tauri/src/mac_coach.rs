//! The shortcut coach on a Mac. When you click a menu item that has a shortcut, a window's
//! own close or minimise button, or a button the Library knows, a small note shows its
//! keys. What was clicked is read through Accessibility (the permission Cat Key's keyboard
//! already has), the way VoiceOver reads it: only its name, nothing else is recorded.
//!
//! It also keeps the counts the Cat Warrior and Coach pages show (shortcuts used, things
//! done the long way, per app per day), in coach.json, just like on Windows.

use catkey_core::coachtext::{clean_name, common_keys};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const KEEP_DAYS: usize = 120;
const SAVE_EVERY: Duration = Duration::from_secs(120);
/// The same tip at most every half hour, a few seconds between tips, ten an hour.
const SAME_TIP: Duration = Duration::from_secs(30 * 60);
const GAP: Duration = Duration::from_secs(8);
const PER_HOUR: usize = 10;
/// The tip hides itself (it waits while the pointer is on it); this is only the backstop.
const SHOW_FOR: Duration = Duration::from_secs(45);

/// Time the long way costs (Keystroke-Level Model): reaching for the mouse and back,
/// pointing and clicking, minus the shortcut itself.
const LOST_MENU_MS: u64 = 2_900;
const LOST_BUTTON_MS: u64 = 1_600;
pub const SAVED_PER_SHORTCUT_MS: u64 = LOST_BUTTON_MS;

const TIP_W: f64 = 440.0;
const TIP_H: f64 = 156.0;

// ------------------------------------------------------------------ settings

#[derive(Clone, Default)]
struct CoachSettings {
    on: bool,
    /// Also in Chrome-based apps (asking them what's under the pointer wakes up their
    /// accessibility tree, which costs them some work).
    chromium: bool,
    muted: Vec<String>,
    excluded: Vec<String>,
    pos: catkey_core::config::CoachPos,
    tip_sound: bool,
}

static SETTINGS: Mutex<Option<CoachSettings>> = Mutex::new(None);
/// Read from the mouse tap's thread without taking a lock.
static ON: AtomicBool = AtomicBool::new(false);

pub fn set_settings(s: &catkey_core::Settings) {
    let c = CoachSettings {
        on: s.coach,
        chromium: s.coach_chromium,
        muted: s.coach_muted.iter().map(|m| m.to_lowercase()).collect(),
        excluded: s.excluded_apps.iter().map(|a| a.to_lowercase()).collect(),
        pos: s.coach_pos.clone(),
        tip_sound: s.sounds && s.sound_parts.get("tips").copied().unwrap_or(false),
    };
    ON.store(c.on, Ordering::SeqCst);
    if let Ok(mut g) = SETTINGS.lock() {
        *g = Some(c);
    }
    if s.coach {
        // The coach needs the mouse (the same quiet listener the paw prints use).
        crate::mac_paws::start();
    }
}

fn settings() -> CoachSettings {
    SETTINGS.lock().ok().and_then(|g| g.clone()).unwrap_or_default()
}

// ------------------------------------------------------------------ the apps' own words

/// Names → shortcuts per app (bundle id, lower case), from the Library's packs.
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
        let _ = crate::app::write_atomic(&dir.join("coach-dict.json"), &text);
    }
    if let Ok(mut g) = DICT.lock() {
        *g = Some(map);
    }
}

fn load_dict(dir: &Path) -> HashMap<String, HashMap<String, String>> {
    std::fs::read_to_string(dir.join("coach-dict.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn dict_lookup(app: &str, name: &str) -> Option<String> {
    let key = name.to_lowercase();
    let g = DICT.lock().ok()?;
    let d = g.as_ref()?;
    d.get(app).and_then(|m| m.get(&key)).cloned()
}

// ------------------------------------------------------------------ stats

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct AppDay {
    #[serde(default)]
    used: u32,
    #[serde(default)]
    caught: u32,
    #[serde(default)]
    lost_ms: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct TipStat {
    app: String,
    name: String,
    keys: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    n: u32,
    #[serde(default)]
    last: u64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct CoachStats {
    #[serde(default)]
    days: BTreeMap<String, HashMap<String, AppDay>>,
    #[serde(default)]
    tips: HashMap<String, TipStat>,
    #[serde(default)]
    saved_per_shortcut_ms: u64,
    #[serde(skip)]
    dirty: bool,
}

static STATS: Mutex<Option<CoachStats>> = Mutex::new(None);
static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static LAST_SAVE: Mutex<Option<Instant>> = Mutex::new(None);
/// Real ⌘/⌥/⌃ + key presses per app since the last flush (from the keyboard thread).
static COMBOS: Mutex<Vec<(String, u32)>> = Mutex::new(Vec::new());

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// From the keyboard thread: never blocks.
pub fn count_combo(app: &str) {
    if app.is_empty() || app.starts_with("com.masterfully.catkey") {
        return;
    }
    if let Ok(mut c) = COMBOS.try_lock() {
        if let Some(e) = c.iter_mut().find(|(a, _)| a == app) {
            e.1 += 1;
        } else if c.len() < 256 {
            c.push((app.to_string(), 1));
        }
    }
}

/// Fold in the counts from the keyboard thread; save if it's time (or `force`).
pub fn flush(force: bool) {
    let combos: Vec<(String, u32)> = COMBOS.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default();
    let snapshot = {
        let Ok(mut g) = STATS.lock() else { return };
        let Some(s) = g.as_mut() else { return };
        if !combos.is_empty() {
            let day = s.days.entry(crate::win::local_date()).or_default();
            for (app, n) in combos {
                day.entry(app).or_default().used += n;
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
            let mut v: Vec<(String, u32)> = s.tips.iter().map(|(k, t)| (k.clone(), t.n)).collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            for (k, _) in v.into_iter().skip(300) {
                s.tips.remove(&k);
            }
        }
        let due = force || LAST_SAVE.lock().ok().and_then(|g| *g).map_or(true, |t| t.elapsed() >= SAVE_EVERY);
        if !s.dirty || !due {
            return;
        }
        s.dirty = false;
        s.saved_per_shortcut_ms = SAVED_PER_SHORTCUT_MS;
        s.clone()
    };
    if let Ok(mut g) = LAST_SAVE.lock() {
        *g = Some(Instant::now());
    }
    if crate::app::NO_SAVE.load(Ordering::SeqCst) {
        return;
    }
    let dir = DIR.lock().ok().and_then(|g| g.clone());
    if let (Some(dir), Ok(text)) = (dir, serde_json::to_string(&snapshot)) {
        let _ = crate::app::write_atomic(&dir.join("coach.json"), &text);
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
    pub anchor: String,
    pub sound: bool,
}

static LAST_TIP: Mutex<Option<TipPayload>> = Mutex::new(None);
static SEQ: AtomicU32 = AtomicU32::new(0);
static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
static APP_NAMES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

pub fn last_tip() -> Option<TipPayload> {
    LAST_TIP.lock().ok().and_then(|g| g.clone())
}

fn app_name(bundle: &str) -> String {
    APP_NAMES
        .lock()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(bundle).cloned()))
        .unwrap_or_else(|| bundle.rsplit('.').next().map(|s| {
            let mut c = s.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        }).unwrap_or_default())
}

fn record(tip: &Tip) -> u32 {
    let Ok(mut g) = STATS.lock() else { return 0 };
    let Some(s) = g.as_mut() else { return 0 };
    let day = s.days.entry(crate::win::local_date()).or_default().entry(tip.app.clone()).or_default();
    day.caught += 1;
    day.lost_ms += tip.lost_ms;
    let kind = match tip.kind {
        Kind::Menu => "menu",
        Kind::Button => "button",
        Kind::Window => "window",
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

/// Show a sample tip (the button on the Coach page, and the start-up check).
pub fn demo() {
    show(TipPayload {
        id: "demo".into(),
        app: "com.apple.finder".into(),
        app_name: "Finder".into(),
        name: "New Folder".into(),
        keys: "Shift+Win+N".into(),
        saves_ms: LOST_MENU_MS,
        times: 1,
        demo: true,
        ..Default::default()
    });
}

/// The tip's own buttons, the window's Dismiss and the timer all end here.
pub fn hide() {
    let Some(app) = APP.lock().ok().and_then(|g| g.clone()) else { return };
    std::thread::spawn(move || {
        if let Some(w) = app.get_webview_window("coach") {
            let _ = w.hide();
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
    p.anchor = s.pos.anchor.clone();
    p.sound = s.tip_sound;
    if let Ok(mut g) = LAST_TIP.lock() {
        *g = Some(p.clone());
    }
    let at = crate::win::pointer().unwrap_or((0.0, 0.0));
    std::thread::spawn(move || {
        let w = match app.get_webview_window("coach") {
            Some(w) => w,
            None => match build(&app) {
                Some(w) => w,
                None => return,
            },
        };
        place(&app, &w, at, &s.pos);
        let _ = app.emit_to("coach", "ck://coach-tip", &p);
        if crate::app::UI_OPEN.load(Ordering::Relaxed) {
            let _ = app.emit_to("main", "ck://coach-tip", &p);
        }
        let _ = w.show();
        std::thread::sleep(SHOW_FOR);
        if SEQ.load(Ordering::SeqCst) == seq {
            let _ = w.hide();
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
        // Ignore, Dismiss and Adjust answer the first click.
        .accept_first_mouse(true)
        .build()
        .map_err(|e| crate::log(&format!("coach window: {e}")))
        .ok()?;
    let _ = w.set_visible_on_all_workspaces(true);
    // Give the page a moment to load; it also asks for the latest tip when it's ready.
    std::thread::sleep(Duration::from_millis(600));
    Some(w)
}

/// Where the user wants tips (bottom right unless chosen), on the screen the pointer is on.
fn place(app: &AppHandle, w: &WebviewWindow, (px, py): (f64, f64), pos: &catkey_core::config::CoachPos) {
    let mut area = (0, 25, 1440, 875);
    if let Ok(mons) = app.available_monitors() {
        let mut first = None;
        for m in mons {
            let k = m.scale_factor().max(0.5);
            let (p, s) = (m.position(), m.size());
            let (x, y, ww, hh) = (p.x as f64 / k, p.y as f64 / k, s.width as f64 / k, s.height as f64 / k);
            let wa = m.work_area();
            let work = (
                (wa.position.x as f64 / k) as i32,
                (wa.position.y as f64 / k) as i32,
                (wa.size.width as f64 / k) as i32,
                (wa.size.height as f64 / k) as i32,
            );
            first.get_or_insert(work);
            if px >= x && px < x + ww && py >= y && py < y + hh {
                first = Some(work);
                break;
            }
        }
        if let Some(a) = first {
            area = a;
        }
    }
    let (x, y) = pos.place(area, TIP_W as i32, TIP_H as i32, 10);
    let _ = w.set_size(tauri::LogicalSize::new(TIP_W, TIP_H));
    let _ = w.set_position(tauri::LogicalPosition::new(x as f64, y as f64));
}

// ------------------------------------------------------------------ what was clicked (Accessibility)

type CFTypeRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> CFTypeRef;
    fn AXUIElementCopyElementAtPosition(app: CFTypeRef, x: f32, y: f32, element: *mut CFTypeRef) -> i32;
    fn AXUIElementCopyAttributeValue(element: CFTypeRef, attribute: CFTypeRef, value: *mut CFTypeRef) -> i32;
    fn AXUIElementSetMessagingTimeout(element: CFTypeRef, seconds: f32) -> i32;
    fn AXUIElementGetPid(element: CFTypeRef, pid: *mut i32) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFNumberGetValue(number: CFTypeRef, kind: isize, out: *mut c_void) -> bool;
}

/// An Accessibility element we own (released when dropped).
struct El(CFTypeRef);

impl Drop for El {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) };
        }
    }
}

impl El {
    fn copy(&self, attr: &str) -> Option<CFTypeRef> {
        let name = CFString::new(attr);
        let mut v: CFTypeRef = null();
        let ok = unsafe { AXUIElementCopyAttributeValue(self.0, name.as_concrete_TypeRef() as CFTypeRef, &mut v) } == 0;
        (ok && !v.is_null()).then_some(v)
    }
    fn string(&self, attr: &str) -> Option<String> {
        let v = self.copy(attr)?;
        unsafe {
            if CFGetTypeID(v) == CFStringGetTypeID() {
                Some(CFString::wrap_under_create_rule(v as _).to_string())
            } else {
                CFRelease(v);
                None
            }
        }
    }
    fn number(&self, attr: &str) -> Option<i64> {
        let v = self.copy(attr)?;
        unsafe {
            let mut out: i64 = 0;
            let ok = CFGetTypeID(v) == CFNumberGetTypeID() && CFNumberGetValue(v, 4, &mut out as *mut i64 as *mut c_void);
            CFRelease(v);
            ok.then_some(out)
        }
    }
    fn parent(&self) -> Option<El> {
        self.copy("AXParent").map(El)
    }
    fn pid(&self) -> i32 {
        let mut pid = 0;
        unsafe { AXUIElementGetPid(self.0, &mut pid) };
        pid
    }
}

/// What's under a point: role, subrole, name, and a menu item's own shortcut.
#[derive(Default, Debug)]
struct Hit {
    role: String,
    subrole: String,
    name: String,
    keys: Option<String>,
    pid: i32,
}

fn hit_at(x: f64, y: f64) -> Option<Hit> {
    unsafe {
        if !AXIsProcessTrusted() {
            return None;
        }
        let sys = El(AXUIElementCreateSystemWide());
        if sys.0.is_null() {
            return None;
        }
        AXUIElementSetMessagingTimeout(sys.0, 0.15);
        let mut raw: CFTypeRef = null();
        if AXUIElementCopyElementAtPosition(sys.0, x as f32, y as f32, &mut raw) != 0 || raw.is_null() {
            return None;
        }
        let mut el = El(raw);
        // A button's picture or label: the button is its parent.
        for _ in 0..3 {
            let role = el.string("AXRole").unwrap_or_default();
            if matches!(role.as_str(), "AXImage" | "AXStaticText" | "AXGroup") {
                match el.parent() {
                    Some(p) => el = p,
                    None => break,
                }
            } else {
                break;
            }
        }
        let role = el.string("AXRole").unwrap_or_default();
        let subrole = el.string("AXSubrole").unwrap_or_default();
        let name = el
            .string("AXTitle")
            .filter(|t| !t.trim().is_empty())
            .or_else(|| el.string("AXDescription"))
            .unwrap_or_default();
        let keys = if role == "AXMenuItem" { menu_keys(&el) } else { None };
        let pid = el.pid();
        Some(Hit { role, subrole, name, keys, pid })
    }
}

/// A menu item's shortcut, written the way Cat Key writes keys ("Shift+Win+N" is ⇧⌘N),
/// in the Mac's own order: ⌃ ⌥ ⇧ ⌘, then the key.
fn menu_keys(el: &El) -> Option<String> {
    let ch = el.string("AXMenuItemCmdChar").filter(|c| !c.trim().is_empty());
    let vk = el.number("AXMenuItemCmdVirtualKey");
    let glyph = el.number("AXMenuItemCmdGlyph");
    let mods = el.number("AXMenuItemCmdModifiers").unwrap_or(0);
    let (key, shifted) = match ch.as_deref().and_then(char_key) {
        Some(k) => k,
        None => match vk.and_then(vk_key).or_else(|| glyph.and_then(glyph_key)) {
            Some(k) => (k, false),
            None => return None,
        },
    };
    let mut parts: Vec<&str> = Vec::new();
    if mods & 4 != 0 {
        parts.push("Ctrl");
    }
    if mods & 2 != 0 {
        parts.push("Alt");
    }
    if mods & 1 != 0 || shifted {
        parts.push("Shift");
    }
    if mods & 8 == 0 {
        parts.push("Win");
    }
    if parts.is_empty() {
        return None; // a bare key isn't "the long way"
    }
    Some(format!("{}+{}", parts.join("+"), key))
}

/// The key a menu's shortcut character stands for, and whether it needs Shift ("?" is ⇧/).
fn char_key(c: &str) -> Option<(String, bool)> {
    let mut it = c.chars();
    let ch = it.next()?;
    if it.next().is_some() {
        return None;
    }
    if ch.is_ascii_alphabetic() {
        return Some((ch.to_ascii_uppercase().to_string(), false));
    }
    if ch.is_ascii_digit() {
        return Some((ch.to_string(), false));
    }
    let plain = |s: &str| Some((s.to_string(), false));
    let shift = |s: &str| Some((s.to_string(), true));
    match ch {
        ',' => plain("Comma"),
        '.' => plain("Period"),
        '/' => plain("Slash"),
        ';' => plain("Semicolon"),
        '\'' => plain("Quote"),
        '[' => plain("LBracket"),
        ']' => plain("RBracket"),
        '\\' => plain("Backslash"),
        '-' => plain("Minus"),
        '=' => plain("Equals"),
        '`' => plain("Backtick"),
        ' ' => plain("Space"),
        '?' => shift("Slash"),
        '+' => shift("Equals"),
        '_' => shift("Minus"),
        '{' => shift("LBracket"),
        '}' => shift("RBracket"),
        ':' => shift("Semicolon"),
        '"' => shift("Quote"),
        '<' => shift("Comma"),
        '>' => shift("Period"),
        '|' => shift("Backslash"),
        '~' => shift("Backtick"),
        '!' => shift("1"),
        '@' => shift("2"),
        '#' => shift("3"),
        '$' => shift("4"),
        '%' => shift("5"),
        '^' => shift("6"),
        '&' => shift("7"),
        '*' => shift("8"),
        '(' => shift("9"),
        ')' => shift("0"),
        _ => None,
    }
}

/// A Mac key code (arrows, F-keys, delete…) as Cat Key names it.
fn vk_key(code: i64) -> Option<String> {
    let vk = crate::hook::mac_to_vk(u16::try_from(code).ok()?);
    (vk != 0).then(|| catkey_core::keys::key_name(vk))
}

/// The menu's drawn key symbols (⌫, ⎋, ←…), for items that only give a symbol.
fn glyph_key(g: i64) -> Option<String> {
    let k = match g {
        0x02 => "Tab",
        0x04 | 0x0B => "Enter",
        0x09 => "Space",
        0x0A => "Delete",
        0x17 => "Backspace",
        0x1B => "Esc",
        0x62 => "PageUp",
        0x64 => "Left",
        0x65 => "Right",
        0x66 => "Home",
        0x68 => "Up",
        0x69 => "End",
        0x6A => "Down",
        0x6B => "PageDown",
        0x6F..=0x7A => return Some(format!("F{}", g - 0x6F + 1)),
        _ => return None,
    };
    Some(k.into())
}

/// An app's bundle id (lower case), its name, and where it lives.
fn bundle_of(pid: i32) -> Option<(String, String, String)> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSRunningApplication;
    autoreleasepool(|_| {
        let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
        let id = app.bundleIdentifier()?.to_string().to_lowercase();
        let name = app.localizedName().map(|s| s.to_string()).unwrap_or_default();
        let path = app.bundleURL().and_then(|u| u.path()).map(|p| p.to_string()).unwrap_or_default();
        Some((id, name, path))
    })
}

static CHROMIUM: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);

/// Chrome, Edge, Brave and the apps made with Electron (Slack, VS Code, Discord…).
fn is_chromium(bundle: &str, path: &str) -> bool {
    if let Some(v) = CHROMIUM.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(bundle).copied())) {
        return v;
    }
    let frameworks = Path::new(path).join("Contents/Frameworks");
    let v = std::fs::read_dir(&frameworks)
        .map(|rd| {
            rd.flatten().any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.ends_with(".framework") && ["electron", "chrome", "chromium", "edge", "brave", "opera", "vivaldi", "arccore"].iter().any(|w| n.contains(w))
            })
        })
        .unwrap_or(false);
    if let Ok(mut g) = CHROMIUM.lock() {
        g.get_or_insert_with(HashMap::new).insert(bundle.to_string(), v);
    }
    v
}

/// What a click on `hit` should have been, if it has a shortcut.
fn tip_for(hit: &Hit, app: &str) -> Option<Tip> {
    match (hit.role.as_str(), hit.subrole.as_str()) {
        ("AXMenuItem", _) => {
            let keys = hit.keys.clone()?;
            let name = clean_name(&hit.name);
            if name.is_empty() || name.len() > 60 {
                return None;
            }
            Some(Tip {
                id: format!("{app}|{}", name.to_lowercase()),
                app: app.into(),
                name,
                keys,
                kind: Kind::Menu,
                lost_ms: LOST_MENU_MS,
            })
        }
        ("AXButton", sub @ ("AXCloseButton" | "AXMinimizeButton" | "AXFullScreenButton")) => {
            let (id, name, keys) = match sub {
                "AXCloseButton" => ("close", "Close the window", "Win+W"),
                "AXMinimizeButton" => ("min", "Minimise", "Win+M"),
                _ => ("max", "Full screen", "Ctrl+Win+F"),
            };
            Some(Tip {
                id: format!("*|window-{id}"),
                app: app.into(),
                name: name.into(),
                keys: keys.into(),
                kind: Kind::Window,
                lost_ms: LOST_BUTTON_MS,
            })
        }
        ("AXButton" | "AXMenuButton" | "AXPopUpButton", _) => {
            let name = clean_name(&hit.name);
            if name.is_empty() || name.len() > 40 {
                return None;
            }
            let keys = dict_lookup(app, &name).or_else(|| common_mac_keys(&name.to_lowercase()))?;
            Some(Tip {
                id: format!("{app}|{}", name.to_lowercase()),
                app: app.into(),
                name,
                keys,
                kind: Kind::Button,
                lost_ms: LOST_BUTTON_MS,
            })
        }
        _ => None,
    }
}

/// Buttons whose shortcut is the same in nearly every Mac app.
fn common_mac_keys(name: &str) -> Option<String> {
    // The Windows table speaks Ctrl; on a Mac the same jobs are ⌘.
    let k = common_keys(name)?;
    if k.contains("Alt") || k.contains("Win") || k.starts_with('F') {
        return None;
    }
    Some(k.replace("Ctrl", "Win"))
}

fn deliver(tip: Tip, shown: &mut VecDeque<Instant>, shown_tip: &mut HashMap<String, Instant>) {
    let times = record(&tip);
    let s = settings();
    let id = tip.id.to_lowercase();
    let app_wide = format!("{}|*", tip.app);
    if s.muted.iter().any(|m| *m == id || *m == app_wide) {
        return;
    }
    while shown.front().is_some_and(|t| t.elapsed() > Duration::from_secs(3600)) {
        shown.pop_front();
    }
    if shown.len() >= PER_HOUR || shown.back().is_some_and(|t| t.elapsed() < GAP) {
        return;
    }
    if shown_tip.get(&id).is_some_and(|t| t.elapsed() < SAME_TIP) {
        return;
    }
    shown.push_back(Instant::now());
    shown_tip.insert(id, Instant::now());
    show(TipPayload {
        id: tip.id.clone(),
        app_name: app_name(&tip.app),
        app: tip.app,
        name: tip.name,
        keys: tip.keys,
        saves_ms: tip.lost_ms,
        times,
        ..Default::default()
    });
}

// ------------------------------------------------------------------ the coach's thread

enum Click {
    Down(f64, f64),
    Up(f64, f64),
}

static TX: Mutex<Option<Sender<Click>>> = Mutex::new(None);

/// From the mouse tap: never blocks.
pub fn click(x: f64, y: f64, down: bool) {
    if !ON.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(g) = TX.try_lock() {
        if let Some(tx) = g.as_ref() {
            let _ = tx.send(if down { Click::Down(x, y) } else { Click::Up(x, y) });
        }
    }
}

pub fn start(app: AppHandle, dir: PathBuf, s: &catkey_core::Settings) {
    if let Ok(mut g) = STATS.lock() {
        *g = Some(std::fs::read_to_string(dir.join("coach.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default());
    }
    if let Ok(mut g) = DICT.lock() {
        *g = Some(load_dict(&dir));
    }
    if let Ok(mut g) = DIR.lock() {
        *g = Some(dir);
    }
    if let Ok(mut g) = APP.lock() {
        *g = Some(app);
    }
    let (tx, rx) = channel();
    if let Ok(mut g) = TX.lock() {
        *g = Some(tx);
    }
    std::thread::Builder::new().name("catkey-coach".into()).spawn(move || run(rx)).ok();
    set_settings(s);
}

fn run(rx: Receiver<Click>) {
    let own = std::process::id() as i32;
    let mut shown: VecDeque<Instant> = VecDeque::new();
    let mut shown_tip: HashMap<String, Instant> = HashMap::new();
    // A press on a menu bar title, so the release (on an item) is the pick.
    let mut menu_drag = false;
    loop {
        let ev = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(ev) => ev,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                flush(false);
                continue;
            }
            Err(_) => return,
        };
        let s = settings();
        if !s.on {
            continue;
        }
        let (x, y, down) = match ev {
            Click::Down(x, y) => (x, y, true),
            Click::Up(x, y) => (x, y, false),
        };
        if !down && !menu_drag {
            continue;
        }
        // Which app it is, before asking it anything (some apps pay for being asked).
        let target = crate::mac_switch::pid_at(x, y)
            .filter(|p| *p != own)
            .or_else(|| crate::win::front_app().map(|(_, p)| p));
        let Some((app, name, path)) = target.and_then(bundle_of) else { continue };
        if s.excluded.iter().any(|e| *e == app) || (!s.chromium && is_chromium(&app, &path)) {
            continue;
        }
        let Some(hit) = hit_at(x, y) else { continue };
        if down {
            menu_drag = hit.role == "AXMenuBarItem";
        } else {
            menu_drag = false;
            if hit.role != "AXMenuItem" {
                continue;
            }
        }
        if hit.pid == 0 || hit.pid == own {
            continue;
        }
        // The element can belong to another app than the window looked like (a pop-up).
        let (app, name) = if Some(hit.pid) == target {
            (app, name)
        } else {
            match bundle_of(hit.pid) {
                Some((a, n, p)) if !s.excluded.iter().any(|e| *e == a) && (s.chromium || !is_chromium(&a, &p)) => (a, n),
                _ => continue,
            }
        };
        if !name.is_empty() {
            if let Ok(mut g) = APP_NAMES.lock() {
                g.get_or_insert_with(HashMap::new).insert(app.clone(), name);
            }
        }
        if let Some(tip) = tip_for(&hit, &app) {
            deliver(tip, &mut shown, &mut shown_tip);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_characters_become_keys() {
        assert_eq!(char_key("c"), Some(("C".into(), false)));
        assert_eq!(char_key("?"), Some(("Slash".into(), true)));
        assert_eq!(char_key(","), Some(("Comma".into(), false)));
        assert_eq!(char_key("ab"), None);
        assert_eq!(glyph_key(0x17).as_deref(), Some("Backspace"));
        assert_eq!(glyph_key(0x70).as_deref(), Some("F2"));
    }
}
