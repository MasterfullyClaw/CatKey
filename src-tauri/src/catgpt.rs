//! CatGPT: show Claude or ChatGPT what's on your screen, fast.
//!
//! A shortcut opens a spinning wheel at the pointer: a screenshot, drawing on the screen,
//! a very light recording, a note, a file, the basket, and Claude or ChatGPT. What you make
//! goes into the basket (Cat Key's own folder, cleared with one button). Sending opens the
//! app (a new chat, the one that's open, or a recent one), puts everything in the message
//! box and stops there, so you can still pick the model or effort; or sends it right away.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
static ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
static SEQ: AtomicU64 = AtomicU64::new(0);
static WHEEL_SEQ: AtomicU64 = AtomicU64::new(0);
static BUSY: AtomicBool = AtomicBool::new(false);
/// What was in front before the wheel or the canvas came up (to give it back).
static BEFORE: Mutex<Option<String>> = Mutex::new(None);
/// The background picture for the canvas (a screenshot), and the screen it covers.
static CANVAS_BG: Mutex<Option<(PathBuf, u32, u32)>> = Mutex::new(None);

struct Rec {
    job: crate::cg::Recording,
    item: Item,
    started: Instant,
    gen: u64,
}
static RECORDING: Mutex<Option<Rec>> = Mutex::new(None);
static REC_GEN: AtomicU64 = AtomicU64::new(0);
static REC_STARTING: AtomicBool = AtomicBool::new(false);

/// The longest recording (it stops by itself after this).
const MAX_REC_SECS: u64 = 600;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Item {
    pub id: String,
    /// "shot", "canvas", "rec", "note" or "file".
    pub kind: String,
    /// The file in CatGPT's folder (empty for a note).
    pub path: String,
    pub name: String,
    /// A note's words.
    pub text: String,
    /// A recording's stills (AI chats read pictures, not videos).
    pub frames: Vec<String>,
    pub bytes: u64,
    pub added: u64,
    pub w: u32,
    pub h: u32,
    pub secs: f64,
    /// Already sent once (the basket sends the new things by default).
    pub sent: bool,
}

fn app() -> Option<AppHandle> {
    APP.lock().ok()?.clone()
}

pub fn dir() -> PathBuf {
    crate::app::config_dir().join("catgpt")
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn new_id() -> String {
    format!("{:x}{:02x}", now_ms(), SEQ.fetch_add(1, Ordering::SeqCst) % 256)
}

fn stamp() -> String {
    crate::win::local_stamp().replace([':', ' '], "-")
}

fn folder(kind: &str) -> PathBuf {
    let d = dir().join(kind);
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn start(app: &AppHandle) {
    *APP.lock().unwrap() = Some(app.clone());
    // Look for Claude and ChatGPT once, early and quietly, so the wheel never waits for it.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(6));
        detect_apps();
    });
    let list: Vec<Item> = std::fs::read(dir().join("basket.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let list: Vec<Item> = list.into_iter().filter(|i| i.kind == "note" || Path::new(&i.path).exists()).collect();
    *ITEMS.lock().unwrap() = list;
}

fn save(items: &[Item]) {
    let _ = std::fs::create_dir_all(dir());
    if let Ok(b) = serde_json::to_vec_pretty(items) {
        let tmp = dir().join("basket.json.tmp");
        if std::fs::write(&tmp, b).is_ok() {
            let _ = std::fs::rename(&tmp, dir().join("basket.json"));
        }
    }
}

fn items() -> Vec<Item> {
    ITEMS.lock().map(|i| i.clone()).unwrap_or_default()
}

fn emit() {
    if let Some(a) = app() {
        let _ = a.emit("ck://catgpt", items());
    }
}

fn add(item: Item) -> Item {
    {
        let mut list = ITEMS.lock().unwrap();
        list.insert(0, item.clone());
        list.truncate(300);
        save(&list);
    }
    emit();
    item
}

fn size_of(p: &str) -> u64 {
    std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

fn folder_bytes(d: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(d) else { return 0 };
    rd.flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => folder_bytes(&e.path()),
            Ok(m) => m.len(),
            _ => 0,
        })
        .sum()
}

// ------------------------------------------------------------------ the AI apps

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiApp {
    pub id: &'static str,
    pub name: &'static str,
    pub installed: bool,
    /// Opens a new chat with words already typed in (its own link).
    pub deep: bool,
}

static APPS: Mutex<Option<(Instant, Vec<AiApp>)>> = Mutex::new(None);

#[cfg(target_os = "macos")]
fn chatgpt_bundle() -> Option<&'static str> {
    // The newer ChatGPT app (it took over from the Codex app) first, then the classic one.
    ["com.openai.codex", "com.openai.chat"].into_iter().find(|b| bundle_installed(b))
}

#[cfg(target_os = "macos")]
fn bundle_installed(id: &str) -> bool {
    use objc2::rc::autoreleasepool;
    autoreleasepool(|_| {
        objc2_app_kit::NSWorkspace::sharedWorkspace().URLForApplicationWithBundleIdentifier(&objc2_foundation::NSString::from_str(id)).is_some()
    })
}

/// The apps as last seen, at once (the wheel can't wait for a slow look). The first time,
/// a quick look at their links; the full look runs in the background.
fn apps_now() -> Vec<AiApp> {
    let cached = APPS.lock().unwrap().as_ref().map(|(at, l)| (at.elapsed(), l.clone()));
    match cached {
        Some((age, list)) => {
            if age > Duration::from_secs(90) {
                std::thread::spawn(detect_apps);
            }
            list
        }
        None => {
            std::thread::spawn(detect_apps);
            let (claude, codex) = (crate::win::has_protocol("claude"), crate::win::has_protocol("codex"));
            vec![
                AiApp { id: "claude", name: "Claude", installed: claude, deep: claude },
                AiApp { id: "chatgpt", name: "ChatGPT", installed: codex, deep: codex },
            ]
        }
    }
}

fn detect_apps() -> Vec<AiApp> {
    if let Some((at, list)) = APPS.lock().unwrap().as_ref() {
        if at.elapsed() < Duration::from_secs(90) {
            return list.clone();
        }
    }
    let claude_link = crate::win::has_protocol("claude");
    let codex_link = crate::win::has_protocol("codex");
    #[cfg(target_os = "macos")]
    let (claude, chatgpt) = (claude_link || bundle_installed("com.anthropic.claudefordesktop"), chatgpt_bundle().is_some());
    #[cfg(windows)]
    let (claude, chatgpt) = (
        claude_link || crate::win::start_app_id(&["Claude"]).is_some(),
        codex_link || crate::win::start_app_id(&["ChatGPT"]).is_some(),
    );
    let list = vec![
        AiApp { id: "claude", name: "Claude", installed: claude, deep: claude_link },
        AiApp { id: "chatgpt", name: "ChatGPT", installed: chatgpt, deep: codex_link },
    ];
    *APPS.lock().unwrap() = Some((Instant::now(), list.clone()));
    list
}

/// Is this app the one in front?
fn in_front(which: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let b = crate::cg::front_bundle();
        match which {
            "claude" => b == "com.anthropic.claudefordesktop",
            _ => b == "com.openai.codex" || b == "com.openai.chat",
        }
    }
    #[cfg(windows)]
    {
        let e = crate::cg::front_exe();
        match which {
            "claude" => e.contains("claude"),
            _ => e.contains("chatgpt") || e.contains("codex") || e.contains("openai"),
        }
    }
}

fn wait_front(which: &str, ms: u64) -> bool {
    let until = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < until {
        if in_front(which) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    in_front(which)
}

/// Bring the app forward as it is (the chat that's open stays open). Opens it if it isn't running.
fn bring(which: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let bundle = if which == "claude" { Some("com.anthropic.claudefordesktop") } else { chatgpt_bundle() };
        let bundle = bundle.ok_or_else(|| missing(which))?;
        if crate::cg::activate(bundle) {
            Ok(())
        } else {
            Err(missing(which))
        }
    }
    #[cfg(windows)]
    {
        let words: &[&str] = if which == "claude" { &["claude"] } else { &["chatgpt", "codex", "openai"] };
        if crate::cg::activate(words) {
            return Ok(());
        }
        let name = if which == "claude" { "Claude" } else { "ChatGPT" };
        let id = crate::win::start_app_id(&[name]).ok_or_else(|| missing(which))?;
        crate::win::launch_app_id(&id)
    }
}

fn missing(which: &str) -> String {
    format!("The {} app isn't on this computer. Get it from {}.", if which == "claude" { "Claude" } else { "ChatGPT" },
        if which == "claude" { "claude.com/download" } else { "chatgpt.com/download" })
}

fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Open an app's own link (claude://, codex://) whatever its length.
fn open_link(scheme: &str, link: &str) -> Result<(), String> {
    if let Some((exe, args)) = crate::win::protocol_command(scheme, link) {
        if crate::win::run_as_user(&exe, &args).is_ok() {
            return Ok(());
        }
    }
    crate::win::open_url(link)
}

fn mods_cmd() -> &'static str {
    if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    }
}

/// Open the chat to send to. Returns true when the words already went in through the link.
fn open_chat(which: &str, dest: &str, text: &str) -> Result<bool, String> {
    let apps = detect_apps();
    let a = apps.iter().find(|a| a.id == which).ok_or("unknown app")?;
    if !a.installed {
        return Err(missing(which));
    }
    let q = |key: &str| if text.is_empty() { String::new() } else { format!("?{key}={}", enc(text)) };
    match (which, dest) {
        ("claude", "new") if a.deep => {
            open_link("claude", &format!("claude://claude.ai/new{}", q("q")))?;
            Ok(!text.is_empty())
        }
        ("claude", d) if d.starts_with("chat:") => {
            let id: String = d[5..].chars().filter(|c| c.is_ascii_hexdigit() || *c == '-').collect();
            open_link("claude", &format!("claude://claude.ai/chat/{id}"))?;
            Ok(false)
        }
        ("chatgpt", "new") if a.deep => {
            open_link("codex", &format!("codex://threads/new{}", q("prompt")))?;
            Ok(!text.is_empty())
        }
        (_, "new") => {
            bring(which)?;
            if wait_front(which, 8000) {
                std::thread::sleep(Duration::from_millis(350));
                crate::cg::press(&[mods_cmd()], "n");
            }
            Ok(false)
        }
        ("chatgpt", d) if d.starts_with("recent:") => {
            let n: u8 = d[7..].parse().unwrap_or(1).clamp(1, 6);
            bring(which)?;
            if wait_front(which, 8000) {
                std::thread::sleep(Duration::from_millis(350));
                crate::cg::press(&[mods_cmd(), "alt"], &n.to_string());
            }
            Ok(false)
        }
        _ => {
            bring(which)?;
            Ok(false)
        }
    }
}

// ------------------------------------------------------------------ sending

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SendReq {
    /// "claude" or "chatgpt".
    pub app: String,
    /// "new", "open", "recent:N" (ChatGPT), "chat:<id>" (Claude).
    pub dest: String,
    /// Which basket items; none = the new ones (or a screenshot taken now, when there are none).
    pub ids: Vec<String>,
    /// The question to ask with them.
    pub text: String,
    /// Press Enter after (otherwise it stays as a draft).
    pub now: bool,
    /// Attach the video files of recordings too (most chats only read the stills).
    pub video: bool,
}

fn pick_frames(frames: &[String], max: usize) -> Vec<String> {
    if frames.len() <= max {
        return frames.to_vec();
    }
    (0..max).map(|i| frames[i * (frames.len() - 1) / (max - 1)].clone()).collect()
}

fn send_blocking(req: SendReq) -> Result<String, String> {
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("Still sending the last one.".into());
    }
    struct Done;
    impl Drop for Done {
        fn drop(&mut self) {
            BUSY.store(false, Ordering::SeqCst);
        }
    }
    let _done = Done;
    let mut chosen: Vec<Item> = {
        let list = ITEMS.lock().unwrap();
        if req.ids.is_empty() {
            list.iter().filter(|i| !i.sent).cloned().collect()
        } else {
            req.ids.iter().filter_map(|id| list.iter().find(|i| &i.id == id).cloned()).collect()
        }
    };
    if chosen.is_empty() {
        // Nothing in the basket: show the AI the screen as it is right now.
        hide_window("catgpt");
        std::thread::sleep(Duration::from_millis(160));
        chosen.push(shot_blocking()?);
    }
    chosen.reverse(); // oldest first, the way they were made
    let mut text = req.text.trim().to_string();
    let notes: Vec<String> = chosen.iter().filter(|i| i.kind == "note" && !i.text.trim().is_empty()).map(|i| i.text.trim().to_string()).collect();
    if !notes.is_empty() {
        let n = notes.join("\n\n");
        text = if text.is_empty() { n } else { format!("{text}\n\n{n}") };
    }
    let mut files: Vec<String> = Vec::new();
    for i in &chosen {
        match i.kind.as_str() {
            "shot" | "canvas" | "file" => files.push(i.path.clone()),
            "rec" => {
                files.extend(pick_frames(&i.frames, 8));
                if req.video {
                    files.push(i.path.clone());
                }
            }
            _ => {}
        }
    }
    files.retain(|f| Path::new(f).exists());
    files.truncate(20);
    let which = if req.app == "chatgpt" { "chatgpt" } else { "claude" };
    let dest = if req.dest.is_empty() { "new" } else { req.dest.as_str() };
    let old_clip = crate::win::clipboard_text();
    let filled = open_chat(which, dest, &text)?;
    if !wait_front(which, 12000) {
        return Err(format!("{} didn't come to the front. Open it once, then try again.", if which == "claude" { "Claude" } else { "ChatGPT" }));
    }
    // Give the message box a moment to be ready.
    std::thread::sleep(Duration::from_millis(if dest == "new" || dest.starts_with("chat:") || dest.starts_with("recent:") { 1400 } else { 500 }));
    if !files.is_empty() {
        if !crate::win::clipboard_write_files(&files, false) {
            return Err("The clipboard was busy. Try again.".into());
        }
        std::thread::sleep(Duration::from_millis(140));
        if !in_front(which) {
            return Err("Something else came to the front, so nothing was pasted.".into());
        }
        crate::cg::paste();
        std::thread::sleep(Duration::from_millis(700 + 160 * files.len().min(10) as u64));
    }
    if !filled && !text.is_empty() {
        if crate::win::clipboard_write_text(&text) && in_front(which) {
            std::thread::sleep(Duration::from_millis(120));
            crate::cg::paste();
            std::thread::sleep(Duration::from_millis(300));
        }
    }
    if req.now {
        // Let the attachments upload first.
        std::thread::sleep(Duration::from_millis(1200 + 500 * files.len().min(8) as u64));
        if in_front(which) {
            crate::cg::press(&[], "enter");
        }
    }
    // Put back what was on the clipboard before (if it was words).
    if let Some(t) = old_clip.filter(|t| !t.is_empty()) {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1500));
            crate::win::clipboard_write_text(&t);
        });
    }
    {
        let mut list = ITEMS.lock().unwrap();
        for i in list.iter_mut() {
            if chosen.iter().any(|c| c.id == i.id) {
                i.sent = true;
            }
        }
        save(&list);
    }
    emit();
    let what = match (files.len(), text.is_empty()) {
        (0, _) => "your words".to_string(),
        (1, true) => "1 picture".to_string(),
        (n, true) => format!("{n} files"),
        (1, false) => "1 picture and your words".to_string(),
        (n, false) => format!("{n} files and your words"),
    };
    let mut c = what.chars();
    let what_cap = c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
    Ok(if req.now { format!("Sent {what}.") } else { format!("{what_cap} in the message box. Pick the model if you like, then press Enter.") })
}

// ------------------------------------------------------------------ capturing

fn shot_blocking() -> Result<Item, String> {
    let path = folder("shots").join(format!("Screenshot {}.jpg", stamp()));
    #[cfg(windows)]
    let (w, h) = crate::cg::screenshot(&path, 1920, 80)?;
    #[cfg(target_os = "macos")]
    let (w, h) = crate::cg::screenshot(&path, 1920, 78)?;
    let p = path.to_string_lossy().to_string();
    Ok(add(Item {
        id: new_id(),
        kind: "shot".into(),
        name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        bytes: size_of(&p),
        path: p,
        added: now_ms() / 1000,
        w,
        h,
        ..Default::default()
    }))
}

fn hide_window(label: &str) {
    if let Some(w) = app().and_then(|a| a.get_webview_window(label)) {
        let _ = w.hide();
    }
}

/// Give the front back to what was there before the wheel or the canvas.
fn give_back() {
    if let Some(id) = BEFORE.lock().unwrap().take() {
        let _ = crate::win::switch_to(&id, false);
    }
}

fn remember_front() {
    if let Some(id) = crate::cg::front_id() {
        *BEFORE.lock().unwrap() = Some(id);
    }
}

// ------------------------------------------------------------------ windows: the wheel, the canvas, the recording pill

/// The screen under the pointer: (position, size) in physical pixels, and its scale.
fn screen_here(a: &AppHandle) -> Option<(PhysicalPosition<i32>, PhysicalSize<u32>, f64)> {
    let c = a.cursor_position().ok()?;
    let mons = a.available_monitors().ok()?;
    let m = mons
        .iter()
        .find(|m| {
            let (p, s) = (m.position(), m.size());
            c.x >= p.x as f64 && c.y >= p.y as f64 && c.x < p.x as f64 + s.width as f64 && c.y < p.y as f64 + s.height as f64
        })
        .or(mons.first())?;
    Some((*m.position(), *m.size(), m.scale_factor()))
}

/// Bring one of CatGPT's windows to the front with the keyboard (Windows only lets an app
/// in the background do that through the same trick the app switcher uses).
fn focus_hard(w: &WebviewWindow) {
    let _ = w.set_focus();
    #[cfg(windows)]
    if let Ok(h) = w.hwnd() {
        let _ = crate::win::switch_to(&(h.0 as usize).to_string(), false);
    }
}

fn window(a: &AppHandle, label: &str, page: &str, focus: bool) -> Option<WebviewWindow> {
    if let Some(w) = a.get_webview_window(label) {
        return Some(w);
    }
    let b = WebviewWindowBuilder::new(a, label, WebviewUrl::App(page.into()))
        .title("CatGPT")
        .inner_size(440.0, 440.0)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(focus)
        .visible(false);
    #[cfg(windows)]
    let b = b.disable_drag_drop_handler();
    #[cfg(target_os = "macos")]
    let b = b.accept_first_mouse(true);
    let w = b.build().map_err(|e| crate::log(&format!("catgpt {label} window: {e}"))).ok()?;
    #[cfg(windows)]
    crate::dock_drop::tool_window(&w);
    #[cfg(target_os = "macos")]
    let _ = w.set_visible_on_all_workspaces(true);
    std::thread::sleep(Duration::from_millis(350));
    Some(w)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WheelPayload {
    seq: u64,
    recording: bool,
    rec_secs: f64,
    fresh: usize,
    apps: Vec<AiApp>,
}

static LAST_WHEEL: Mutex<Option<WheelPayload>> = Mutex::new(None);

/// Open the wheel at the pointer (the CatGPT shortcut).
pub fn wheel() {
    let Some(a) = app() else { return };
    std::thread::spawn(move || {
        // While recording, the shortcut stops the recording.
        if RECORDING.lock().map(|r| r.is_some()).unwrap_or(false) {
            let _ = record_stop();
            return;
        }
        let Some(w) = window(&a, "catgpt", "catgpt.html", true) else { return };
        if !w.is_visible().unwrap_or(false) {
            remember_front();
        }
        let (Ok(c), Some((mp, ms, k))) = (a.cursor_position(), screen_here(&a)) else { return };
        let size = (440.0 * k).round() as i32;
        let x = (c.x.round() as i32 - size / 2).clamp(mp.x, mp.x + ms.width as i32 - size);
        let y = (c.y.round() as i32 - size / 2).clamp(mp.y, mp.y + ms.height as i32 - size);
        let _ = w.set_size(PhysicalSize::new(size as u32, size as u32));
        let _ = w.set_position(PhysicalPosition::new(x, y));
        let fresh = ITEMS.lock().map(|l| l.iter().filter(|i| !i.sent).count()).unwrap_or(0);
        let p = WheelPayload { seq: WHEEL_SEQ.fetch_add(1, Ordering::SeqCst) + 1, recording: false, rec_secs: 0.0, fresh, apps: apps_now() };
        *LAST_WHEEL.lock().unwrap() = Some(p.clone());
        let _ = w.emit("ck://wheel", p);
        let _ = w.show();
        focus_hard(&w);
    });
}

/// From the shortcut: a tool straight away ("shot", "draw", "record", "note"), or the wheel.
pub fn from_shortcut(what: &str) {
    match what {
        "shot" | "screenshot" => {
            std::thread::spawn(|| {
                if let Err(e) = shot_blocking() {
                    notify(&e);
                } else {
                    notify("Screenshot in the basket.");
                }
            });
        }
        "draw" | "canvas" => {
            remember_front();
            std::thread::spawn(|| {
                let _ = canvas_open(false);
            });
        }
        "record" => {
            std::thread::spawn(|| {
                let on = RECORDING.lock().map(|r| r.is_some()).unwrap_or(false);
                let r = if on { record_stop().map(|_| ()) } else { record_start() };
                if let Err(e) = r {
                    notify(&e);
                }
            });
        }
        _ => wheel(),
    }
}

/// A short message from CatGPT (shown by the main window or the wheel).
fn notify(text: &str) {
    if let Some(a) = app() {
        let _ = a.emit("ck://catgpt-note", text.to_string());
    }
}

#[tauri::command]
pub fn catgpt_wheel_last() -> Option<WheelPayload> {
    LAST_WHEEL.lock().ok()?.clone()
}

/// The wheel closes; `back` gives the front back to the app that had it.
static CI: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn catgpt_wheel_hide(back: bool) {
    if CI.load(Ordering::Relaxed) {
        crate::log(&format!("catgpt: the wheel asked to close (back {back})"));
    }
    hide_window("catgpt");
    if back {
        give_back();
    }
}

fn canvas_open(blank: bool) -> Result<(), String> {
    let a = app().ok_or("not ready")?;
    hide_window("catgpt");
    std::thread::sleep(Duration::from_millis(180));
    let Some((mp, ms, _)) = screen_here(&a) else { return Err("No screen under the pointer.".into()) };
    if blank {
        *CANVAS_BG.lock().unwrap() = None;
    } else {
        let p = folder("tmp").join("canvas-bg.jpg");
        #[cfg(windows)]
        let (w, h) = crate::cg::screenshot(&p, 3840, 88)?;
        #[cfg(target_os = "macos")]
        let (w, h) = crate::cg::screenshot(&p, 3840, 86)?;
        *CANVAS_BG.lock().unwrap() = Some((p, w, h));
    }
    let w = window(&a, "canvas", "canvas.html", true).ok_or("The canvas didn't open.")?;
    let _ = w.set_position(mp);
    let _ = w.set_size(ms);
    #[cfg(target_os = "macos")]
    crate::cg::float_above_all(&a, &w, 101);
    let _ = w.emit("ck://canvas", WHEEL_SEQ.fetch_add(1, Ordering::SeqCst) + 1);
    let _ = w.show();
    focus_hard(&w);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasBg {
    /// A data: URL of the screenshot, or empty for a blank canvas.
    image: String,
    w: u32,
    h: u32,
}

/// The canvas asks for its background (the screen as it was).
#[tauri::command]
pub fn catgpt_canvas_bg() -> CanvasBg {
    match CANVAS_BG.lock().unwrap().clone() {
        Some((p, w, h)) => CanvasBg { image: data_url(&p).unwrap_or_default(), w, h },
        None => CanvasBg { image: String::new(), w: 0, h: 0 },
    }
}

/// A fresh canvas: the screen again, or blank paper.
#[tauri::command]
pub async fn catgpt_canvas_new(blank: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        hide_window("canvas");
        std::thread::sleep(Duration::from_millis(220));
        canvas_open(blank)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The drawing is done: keep it in the basket (a JPEG made by the canvas page).
#[tauri::command]
pub fn catgpt_canvas_save(data: String, w: u32, h: u32) -> Result<Item, String> {
    let b64 = data.split_once(',').map(|(_, b)| b).unwrap_or(&data);
    let bytes = b64_decode(b64).ok_or("The drawing couldn't be read.")?;
    let path = folder("canvases").join(format!("Canvas {}.jpg", stamp()));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    let p = path.to_string_lossy().to_string();
    Ok(add(Item {
        id: new_id(),
        kind: "canvas".into(),
        name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        bytes: bytes.len() as u64,
        path: p,
        added: now_ms() / 1000,
        w,
        h,
        ..Default::default()
    }))
}

#[tauri::command]
pub fn catgpt_canvas_close(back: bool) {
    hide_window("canvas");
    let _ = std::fs::remove_file(folder("tmp").join("canvas-bg.jpg"));
    if back {
        give_back();
    }
}

fn record_start() -> Result<(), String> {
    if RECORDING.lock().unwrap().is_some() || REC_STARTING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    struct Starting;
    impl Drop for Starting {
        fn drop(&mut self) {
            REC_STARTING.store(false, Ordering::SeqCst);
        }
    }
    let _s = Starting;
    hide_window("catgpt");
    std::thread::sleep(Duration::from_millis(150));
    let name = format!("Recording {}", stamp());
    let path = folder("recordings").join(format!("{name}.mp4"));
    let frames = folder("recordings").join(&name);
    let _ = std::fs::create_dir_all(&frames);
    let job = crate::cg::record_start(&path, &frames, MAX_REC_SECS)?;
    let item = Item {
        id: new_id(),
        kind: "rec".into(),
        name: format!("{name}.mp4"),
        path: path.to_string_lossy().to_string(),
        added: now_ms() / 1000,
        ..Default::default()
    };
    let gen = REC_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    *RECORDING.lock().unwrap() = Some(Rec { job, item, started: Instant::now(), gen });
    pill(true);
    give_back();
    // Stop by itself at the limit (only this recording, not a later one).
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(MAX_REC_SECS + 2));
        if RECORDING.lock().map(|r| r.as_ref().map(|r| r.gen) == Some(gen)).unwrap_or(false) {
            let _ = record_stop();
        }
    });
    Ok(())
}

fn record_stop() -> Result<Item, String> {
    let Some(r) = RECORDING.lock().unwrap().take() else { return Err("Nothing is recording.".into()) };
    pill(false);
    let secs = r.started.elapsed().as_secs_f64();
    let done = r.job.stop()?;
    let mut item = r.item;
    item.bytes = size_of(&item.path);
    item.w = done.w;
    item.h = done.h;
    item.secs = if done.secs > 0.0 { done.secs } else { secs };
    item.frames = done.frames;
    let item = add(item);
    notify(&format!("Recording in the basket ({}).", mb(item.bytes)));
    Ok(item)
}

fn mb(b: u64) -> String {
    if b >= 1_000_000 {
        format!("{:.1} MB", b as f64 / 1_000_000.0)
    } else {
        format!("{} KB", (b / 1000).max(1))
    }
}

/// The little "recording" pill at the top of the screen, with its Stop button.
fn pill(on: bool) {
    let Some(a) = app() else { return };
    if !on {
        hide_window("rec");
        return;
    }
    let Some(w) = window(&a, "rec", "rec.html", false) else { return };
    if let Some((mp, ms, k)) = screen_here(&a) {
        let (pw, ph) = ((230.0 * k) as u32, (54.0 * k) as u32);
        let _ = w.set_size(PhysicalSize::new(pw, ph));
        let _ = w.set_position(PhysicalPosition::new(mp.x + (ms.width as i32 - pw as i32) / 2, mp.y + (14.0 * k) as i32));
    }
    let _ = w.emit("ck://rec", now_ms());
    #[cfg(windows)]
    crate::coach::show_quietly(&w);
    #[cfg(target_os = "macos")]
    let _ = w.show();
}

// ------------------------------------------------------------------ commands

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    apps: Vec<AiApp>,
    items: Vec<Item>,
    dir: String,
    bytes: u64,
    recording: bool,
    rec_started: u64,
    /// Mac: Screen Recording is allowed (Windows: always true).
    can_capture: bool,
}

#[tauri::command]
pub async fn catgpt_info() -> Result<Info, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let (recording, started) = {
            let rec = RECORDING.lock().unwrap();
            (rec.is_some(), rec.as_ref().map(|r| now_ms().saturating_sub(r.started.elapsed().as_millis() as u64)).unwrap_or(0))
        };
        Info {
            apps: detect_apps(),
            items: items(),
            dir: dir().to_string_lossy().to_string(),
            bytes: folder_bytes(&dir()),
            recording,
            rec_started: started,
            #[cfg(target_os = "macos")]
            can_capture: crate::cg::can_record(),
            #[cfg(windows)]
            can_capture: true,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

/// Mac: ask for Screen Recording (macOS shows its prompt), and open its page in System Settings.
#[tauri::command]
pub fn catgpt_ask_capture() -> bool {
    #[cfg(target_os = "macos")]
    {
        let ok = crate::cg::ask_to_record();
        if !ok {
            crate::cg::open_record_settings();
        }
        ok
    }
    #[cfg(windows)]
    true
}

#[tauri::command]
pub async fn catgpt_shot() -> Result<Item, String> {
    tauri::async_runtime::spawn_blocking(|| {
        hide_window("catgpt");
        std::thread::sleep(Duration::from_millis(170));
        let r = shot_blocking();
        give_back();
        r
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn catgpt_draw(blank: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || canvas_open(blank)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn catgpt_record(on: bool) -> Result<Option<Item>, String> {
    tauri::async_runtime::spawn_blocking(move || if on { record_start().map(|_| None) } else { record_stop().map(Some) })
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn catgpt_note(text: String) -> Result<Item, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("The note is empty.".into());
    }
    let t: String = t.chars().take(20_000).collect();
    Ok(add(Item {
        id: new_id(),
        kind: "note".into(),
        name: t.lines().next().unwrap_or("Note").chars().take(60).collect(),
        bytes: t.len() as u64,
        text: t,
        added: now_ms() / 1000,
        ..Default::default()
    }))
}

/// Files dropped on CatGPT or picked: copied into its folder (so the originals stay put).
#[tauri::command]
pub fn catgpt_add_files(paths: Vec<String>) -> Result<Vec<Item>, String> {
    let mut out = Vec::new();
    for p in paths.into_iter().take(20) {
        let src = PathBuf::from(&p);
        let Ok(meta) = std::fs::metadata(&src) else { continue };
        if !meta.is_file() {
            continue;
        }
        if meta.len() > 200 * 1024 * 1024 {
            return Err(format!("{} is over 200 MB.", src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
        }
        let name = src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
        let sub = folder("files").join(new_id());
        let _ = std::fs::create_dir_all(&sub);
        let dst = sub.join(&name);
        std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
        let d = dst.to_string_lossy().to_string();
        out.push(add(Item { id: new_id(), kind: "file".into(), name, bytes: meta.len(), path: d, added: now_ms() / 1000, ..Default::default() }));
    }
    Ok(out)
}

#[tauri::command]
pub async fn catgpt_pick_files(app: AppHandle) -> Result<Vec<Item>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().set_title("Add to the CatGPT basket").blocking_pick_files();
    let paths: Vec<String> = picked.unwrap_or_default().into_iter().filter_map(|f| f.into_path().ok()).map(|p| p.to_string_lossy().to_string()).collect();
    catgpt_add_files(paths)
}

fn remove_files(i: &Item) {
    if i.path.is_empty() {
        return;
    }
    let p = Path::new(&i.path);
    let _ = std::fs::remove_file(p);
    if i.kind == "rec" {
        let _ = std::fs::remove_dir_all(p.with_extension(""));
    }
    if i.kind == "file" {
        if let Some(parent) = p.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
}

#[tauri::command]
pub fn catgpt_remove(ids: Vec<String>) {
    {
        let mut list = ITEMS.lock().unwrap();
        for i in list.iter().filter(|i| ids.contains(&i.id)) {
            remove_files(i);
        }
        list.retain(|i| !ids.contains(&i.id));
        save(&list);
    }
    emit();
}

/// Empty the basket and CatGPT's whole folder.
#[tauri::command]
pub fn catgpt_clear() -> Result<(), String> {
    if RECORDING.lock().map(|r| r.is_some()).unwrap_or(false) {
        return Err("Stop the recording first.".into());
    }
    ITEMS.lock().unwrap().clear();
    let d = dir();
    if d.exists() {
        std::fs::remove_dir_all(&d).map_err(|e| e.to_string())?;
    }
    emit();
    Ok(())
}

#[tauri::command]
pub fn catgpt_open_folder() -> Result<(), String> {
    let _ = std::fs::create_dir_all(dir());
    crate::win::open_path(&dir().to_string_lossy())
}

#[tauri::command]
pub fn catgpt_open_item(id: String, reveal: bool) -> Result<(), String> {
    let it = items().into_iter().find(|i| i.id == id).ok_or("That's gone from the basket.")?;
    if reveal {
        crate::win::reveal_path(&it.path)
    } else {
        crate::win::open_path(&it.path)
    }
}

/// A picture from the basket as a data: URL (for the thumbnails), or one of a recording's stills.
#[tauri::command]
pub fn catgpt_image(id: String, frame: Option<usize>) -> Option<String> {
    let it = items().into_iter().find(|i| i.id == id)?;
    let p = match it.kind.as_str() {
        "rec" => it.frames.get(frame.unwrap_or(0)).cloned()?,
        "shot" | "canvas" => it.path.clone(),
        "file" if is_picture(&it.path) => it.path.clone(),
        _ => return None,
    };
    data_url(Path::new(&p))
}

fn is_picture(p: &str) -> bool {
    let l = p.to_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp"].iter().any(|e| l.ends_with(e))
}

#[tauri::command]
pub async fn catgpt_send(req: SendReq) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let r = send_blocking(req);
        if let Err(e) = &r {
            crate::log(&format!("catgpt send: {e}"));
        }
        r
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recent {
    pub title: String,
    /// Where to send: "recent:N" or "chat:<id>".
    pub dest: String,
}

/// Recent chats to pick from on the go.
#[tauri::command]
pub async fn catgpt_recent(app: String) -> Vec<Recent> {
    tauri::async_runtime::spawn_blocking(move || recent_chats(&app)).await.unwrap_or_default()
}

fn recent_chats(which: &str) -> Vec<Recent> {
    if which == "chatgpt" {
        // The ChatGPT app opens its six most recent chats with ⌘⌥1–6 / Ctrl+Alt+1–6.
        return (1..=6).map(|n| Recent { title: format!("Recent chat {n}"), dest: format!("recent:{n}") }).collect();
    }
    crate::cg::claude_recent().into_iter().map(|(title, id)| Recent { title, dest: format!("chat:{id}") }).collect()
}

#[tauri::command]
pub fn catgpt_wheel_open() {
    wheel();
}

/// The release check (--dock-test): every part once, each result in the log.
pub fn ci_test() {
    CI.store(true, Ordering::Relaxed);
    crate::log(&format!("catgpt: {}", self_test()));
    match shot_blocking() {
        Ok(i) => {
            crate::log(&format!("catgpt: screenshot {}x{} {} KB", i.w, i.h, i.bytes / 1000));
            catgpt_remove(vec![i.id]);
        }
        Err(e) => crate::log(&format!("catgpt: screenshot failed: {e}")),
    }
    let vis = |l: &str| app().and_then(|a| a.get_webview_window(l)).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    let t = Instant::now();
    wheel();
    let mut shown = false;
    while t.elapsed() < Duration::from_secs(6) && !shown {
        std::thread::sleep(Duration::from_millis(200));
        shown = vis("catgpt");
    }
    crate::log(&format!("catgpt: wheel shown {shown} after {} ms", t.elapsed().as_millis()));
    std::thread::sleep(Duration::from_millis(800));
    CI.store(false, Ordering::Relaxed);
    catgpt_wheel_hide(false);
    CI.store(true, Ordering::Relaxed);
    let c = canvas_open(true);
    std::thread::sleep(Duration::from_secs(3));
    crate::log(&format!("catgpt: canvas shown {} {:?}", vis("canvas"), c.err()));
    catgpt_canvas_close(false);
    match record_start() {
        Ok(()) => {
            std::thread::sleep(Duration::from_secs(4));
            crate::log(&format!("catgpt: recording pill shown {}", vis("rec")));
            match record_stop() {
                Ok(i) => {
                    crate::log(&format!("catgpt: recording {:.1} s, {} KB, {} stills, {}x{}", i.secs, i.bytes / 1000, i.frames.len(), i.w, i.h));
                    catgpt_remove(vec![i.id]);
                }
                Err(e) => crate::log(&format!("catgpt: recording failed to stop: {e}")),
            }
        }
        Err(e) => crate::log(&format!("catgpt: recording failed to start: {e}")),
    }
    let apps = detect_apps();
    crate::log(&format!("catgpt: apps {}", apps.iter().map(|a| format!("{}={}", a.id, a.installed)).collect::<Vec<_>>().join(" ")));
}

/// What a test run checks: the folder works and a note goes in and out of the basket.
pub fn self_test() -> String {
    match catgpt_note("CatGPT self-test".into()) {
        Ok(it) => {
            let ok = items().iter().any(|i| i.id == it.id);
            catgpt_remove(vec![it.id]);
            format!("basket {}", if ok { "ok" } else { "missing" })
        }
        Err(e) => format!("basket failed: {e}"),
    }
}

// ------------------------------------------------------------------ small helpers

fn data_url(p: &Path) -> Option<String> {
    let bytes = std::fs::read(p).ok()?;
    let mime = match p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "image/jpeg",
    };
    Some(format!("data:{mime};base64,{}", b64_encode(&bytes)))
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut val = [255u8; 256];
    for (i, c) in B64.iter().enumerate() {
        val[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let (mut buf, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        if c == b'=' || c.is_ascii_whitespace() {
            continue;
        }
        let v = val[c as usize];
        if v == 255 {
            return None;
        }
        buf = buf << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        for s in [&b""[..], b"a", b"ab", b"abc", b"abcd", b"\x00\xff\x10\x80hello world"] {
            assert_eq!(b64_decode(&b64_encode(s)).unwrap(), s);
        }
        assert_eq!(b64_encode(b"Cat"), "Q2F0");
    }

    #[test]
    fn link_text_is_encoded() {
        assert_eq!(enc("a b&c/ü"), "a%20b%26c%2F%C3%BC");
    }

    #[test]
    fn frames_are_spread() {
        let f: Vec<String> = (0..20).map(|i| i.to_string()).collect();
        let p = pick_frames(&f, 8);
        assert_eq!(p.len(), 8);
        assert_eq!(p.first().unwrap(), "0");
        assert_eq!(p.last().unwrap(), "19");
    }
}
