//! Commands the UI calls through `invoke`.

use crate::app::{self, AppState, Stats, POWERTOYS_ENGINE};
use crate::{hook, win};
use catkey_core::config::rule_problem;
use catkey_core::config::Rule;
use catkey_core::{builtin_templates, powertoys, Config, Template};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    version: String,
    admin: bool,
    powertoys_running: bool,
    /// PowerToys Grab And Move is on (it listens to Alt too).
    grab_and_move: bool,
    first_run: bool,
    config_reset: bool,
    config_dir: String,
    /// Rule id -> why it can't run (shown on the card).
    problems: HashMap<String, String>,
    /// The keyboard hook is installed and its thread is alive.
    engine_running: bool,
    /// "on" | "off" | "blocked" (Windows turned it off in Startup apps) | "missing"
    startup: String,
    /// "windows" or "mac": which key names and app ids this copy uses.
    platform: &'static str,
    /// Keys the watchdog caught that the keyboard hook missed (since Cat Key started).
    watch_fixes: u32,
    /// Mac: Cat Key may use the keyboard (Privacy & Security → Accessibility).
    keyboard_allowed: bool,
    /// Mac: running straight from the disk image or Downloads (permissions won't stick).
    temp_place: bool,
    /// Mac: CatGPT may see the screen (Screen & System Audio Recording). Windows: always.
    screen_allowed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatePayload {
    config: Config,
    status: Status,
    stats: Stats,
    /// Templates that ship with Cat Key (the user's own are in `config.templates`).
    templates: Vec<Template>,
    /// Settings came from an older version and just got its new shortcuts (told once).
    upgraded: bool,
}

fn status(state: &AppState) -> Status {
    // Take what we need and let go of the lock right away (callers may hold nothing else).
    let (problems, settings) = {
        let cfg = state.config.lock().unwrap();
        let problems: HashMap<String, String> =
            cfg.rules.iter().filter_map(|r| rule_problem(r).map(|p| (r.id.clone(), p))).collect();
        (problems, cfg.settings.clone())
    };
    Status {
        version: env!("CARGO_PKG_VERSION").to_string(),
        admin: win::is_elevated(),
        powertoys_running: win::process_running(POWERTOYS_ENGINE),
        grab_and_move: win::powertoys_grab_and_move_on(),
        first_run: state.first_run,
        config_reset: state.config_reset,
        config_dir: state.dir.to_string_lossy().to_string(),
        problems,
        engine_running: hook::engine_running(),
        startup: app::startup_status(&settings).to_string(),
        platform: if cfg!(target_os = "macos") { "mac" } else { "windows" },
        watch_fixes: hook::WATCH_FIXES.load(std::sync::atomic::Ordering::Relaxed),
        #[cfg(target_os = "macos")]
        keyboard_allowed: win::keyboard_allowed(),
        #[cfg(not(target_os = "macos"))]
        keyboard_allowed: true,
        #[cfg(target_os = "macos")]
        temp_place: win::running_from_temp_place(),
        #[cfg(not(target_os = "macos"))]
        temp_place: false,
        #[cfg(target_os = "macos")]
        screen_allowed: crate::cg::can_record(),
        #[cfg(not(target_os = "macos"))]
        screen_allowed: true,
    }
}

fn fresh_stats(state: &AppState) -> Stats {
    let mut s = state.stats.lock().unwrap().clone();
    if s.day != win::local_date() {
        s.today = 0;
    }
    s
}

// `async` so these run on a worker thread: status() reads the registry and looks at
// running processes, and the window's main thread should never wait on that.
#[tauri::command]
pub async fn get_state(app: AppHandle) -> StatePayload {
    let state = app.state::<AppState>();
    // Separate statements on purpose: a lock guard created inside one struct literal
    // would stay alive while status() locks the same config again (that froze 1.0.0).
    let config = state.config.lock().unwrap().clone();
    let status = status(&state);
    let stats = fresh_stats(&state);
    let upgraded = state.upgraded.swap(false, std::sync::atomic::Ordering::SeqCst);
    StatePayload { config, status, stats, templates: builtin_templates(), upgraded }
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Status {
    let state = app.state::<AppState>();
    status(&state)
}

#[tauri::command]
pub fn get_stats(state: State<'_, AppState>) -> Stats {
    fresh_stats(&state)
}

#[tauri::command]
pub async fn save_config(app: AppHandle, config: Config) -> Result<Config, String> {
    app::apply_config(&app, config)
}

#[tauri::command]
pub fn set_enabled(app: AppHandle, enabled: bool) {
    app::set_enabled(&app, enabled);
}

#[tauri::command]
pub fn start_recording() {
    hook::send(hook::Cmd::StartRecord);
}

#[tauri::command]
pub fn stop_recording() {
    hook::send(hook::Cmd::StopRecord);
}

#[tauri::command]
pub async fn running_apps(app: AppHandle) -> Vec<serde_json::Value> {
    let apps = win::running_apps();
    let state = app.state::<AppState>();
    apps.into_iter()
        .map(|a| {
            let icon = cached_icon(&state, &a.path);
            serde_json::json!({ "exe": a.exe, "name": a.name, "path": a.path, "title": a.title, "icon": icon })
        })
        .collect()
}

fn cached_icon(state: &AppState, path: &str) -> Option<String> {
    let key = path.to_lowercase();
    if let Some(v) = state.icons.lock().unwrap().get(&key) {
        return v.clone();
    }
    let icon = win::icon_data_url(path);
    state.icons.lock().unwrap().insert(key, icon.clone());
    icon
}

/// Icon for a path, or for an exe name we have seen running.
#[tauri::command]
pub async fn app_icon(app: AppHandle, path: String) -> Option<String> {
    let state = app.state::<AppState>();
    if path.contains('\\') || path.contains('/') || path.contains('%') {
        return cached_icon(&state, &path);
    }
    let exe = path.to_lowercase();
    let key = format!("exe:{exe}");
    if let Some(v) = state.icons.lock().unwrap().get(&key) {
        return v.clone();
    }
    let found = win::running_apps().into_iter().find(|a| a.exe == exe).and_then(|a| cached_icon(&state, &a.path));
    if found.is_some() {
        state.icons.lock().unwrap().insert(key, found.clone());
    }
    found
}

/// Apps installed on this computer: the Start menu on Windows, Applications on a Mac.
#[tauri::command]
pub async fn installed_apps() -> Vec<win::AppInfo> {
    tauri::async_runtime::spawn_blocking(win::installed_apps).await.unwrap_or_default()
}

#[derive(Serialize)]
pub struct Picked {
    path: String,
    exe: String,
    name: String,
    icon: Option<String>,
}

#[tauri::command]
pub async fn pick_program(app: AppHandle) -> Option<Picked> {
    let file = app
        .dialog()
        .file()
        .set_title("Pick an app or file");
    #[cfg(target_os = "macos")]
    let file = file.set_directory("/Applications").add_filter("Apps", &["app"]);
    #[cfg(not(target_os = "macos"))]
    let file = file.add_filter("Apps and shortcuts", &["exe", "lnk", "bat", "cmd", "url"]).add_filter("Any file", &["*"]);
    let file = file.blocking_pick_file()?;
    let path = file.into_path().ok()?.to_string_lossy().to_string();
    let exe = win::file_name_lower(&path);
    let name = win::file_description(&path).unwrap_or_else(|| {
        std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
    });
    let icon = cached_icon(&app.state::<AppState>(), &path);
    Some(Picked { path, exe, name, icon })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    found: usize,
    added: usize,
    /// Templates brought over from a Cat Key export.
    templates: usize,
    config: Config,
}

fn powertoys_file() -> Option<PathBuf> {
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("Microsoft\\PowerToys\\Keyboard Manager");
    let active = std::fs::read_to_string(base.join("settings.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.pointer("/properties/activeConfiguration/value").and_then(|x| x.as_str()).map(String::from))
        .unwrap_or_else(|| "default".into());
    [base.join(format!("{active}.json")), base.join("default.json")].into_iter().find(|p| p.is_file())
}

fn merge_and_apply(app: &AppHandle, incoming: Vec<catkey_core::Rule>) -> Result<ImportResult, String> {
    let found = incoming.len();
    let mut cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let fresh = powertoys::merge_new(&cfg.rules, incoming);
    let added = fresh.len();
    cfg.rules.extend(fresh);
    let config = if added > 0 { app::apply_config(app, cfg)? } else { cfg };
    Ok(ImportResult { found, added, templates: 0, config })
}

#[tauri::command]
pub async fn import_powertoys(app: AppHandle) -> Result<ImportResult, String> {
    let path = powertoys_file().ok_or("PowerToys Keyboard Manager settings weren't found on this PC.")?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Couldn't read PowerToys settings: {e}"))?;
    merge_and_apply(&app, powertoys::import(&text)?)
}

/// Import a Cat Key backup or a PowerToys file. New shortcuts are added; nothing is removed.
#[tauri::command]
pub async fn import_file(app: AppHandle) -> Result<Option<ImportResult>, String> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Import shortcuts")
        .add_filter("Cat Key or PowerToys file", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let (incoming, templates) = match serde_json::from_str::<Config>(&text) {
        Ok(c) if !c.rules.is_empty() || !c.templates.is_empty() => (c.rules, c.templates),
        _ => (powertoys::import(&text)?, Vec::new()),
    };
    if incoming.is_empty() && templates.is_empty() {
        return Err("No shortcuts found in that file.".into());
    }
    let mut new_templates = 0;
    if !templates.is_empty() {
        // Templates from a Cat Key export come along too (skipping names already here).
        let mut cfg = app.state::<AppState>().config.lock().unwrap().clone();
        let before = cfg.templates.len();
        for t in templates.into_iter().filter(|t| !t.builtin && !t.id.starts_with("builtin-")) {
            if !cfg.templates.iter().any(|x| x.name.eq_ignore_ascii_case(&t.name)) {
                cfg.templates.push(Template { id: String::new(), ..t });
            }
        }
        new_templates = cfg.templates.len() - before;
        if new_templates > 0 {
            app::apply_config(&app, cfg)?;
        }
    }
    let mut res = merge_and_apply(&app, incoming)?;
    res.templates = new_templates;
    Ok(Some(res))
}

#[tauri::command]
pub async fn export_file(app: AppHandle) -> Result<bool, String> {
    let cfg = app.state::<AppState>().config.lock().unwrap().clone();
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Export shortcuts")
        .set_file_name("CatKey-shortcuts.json")
        .add_filter("Cat Key file", &["json"])
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn restore_defaults(app: AppHandle) -> Result<Config, String> {
    let mut cfg = app.state::<AppState>().config.lock().unwrap().clone();
    cfg.rules = catkey_core::config::default_rules();
    app::apply_config(&app, cfg)
}

/// "saved" when done here, "relaunching" when an admin copy is starting.
#[tauri::command]
pub async fn set_admin_mode(app: AppHandle, enabled: bool) -> Result<String, String> {
    let mut cfg = app.state::<AppState>().config.lock().unwrap().clone();
    cfg.settings.admin_mode = enabled;
    if !enabled || win::is_elevated() {
        app::apply_config(&app, cfg)?;
        return Ok("saved".into());
    }
    // Save first so the admin copy starts with admin mode on.
    let dir = app.state::<AppState>().dir.clone();
    app::save_config_file(&dir, &cfg)?;
    match win::relaunch_elevated(&format!("--wait-pid {}", std::process::id())) {
        Ok(()) => {
            app.exit(0);
            Ok("relaunching".into())
        }
        Err(e) => {
            cfg.settings.admin_mode = false;
            let _ = app::save_config_file(&dir, &cfg);
            Err(e)
        }
    }
}

/// Leave admin mode right away: start a normal copy and quit this one.
#[tauri::command]
pub fn restart_normal(app: AppHandle) {
    if win::is_elevated() {
        win::relaunch_unelevated_after_exit();
    }
    app.exit(0);
}

#[tauri::command]
pub fn ui_ready(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
pub fn window_minimize(window: WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_toggle_maximize(window: WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

/// Close the window and free the webview; Cat Key keeps running in the tray.
#[tauri::command]
pub async fn window_close(window: WebviewWindow) {
    hook::send(hook::Cmd::StopRecord);
    let _ = window.destroy();
}

/// Open a Windows or PowerToys page so the user can fix something the checkup found.
#[tauri::command]
pub async fn open_system_page(page: String) -> Result<(), String> {
    match page.as_str() {
        #[cfg(target_os = "macos")]
        "startup" => win::launch("x-apple.systempreferences:com.apple.LoginItems-Settings.extension", ""),
        #[cfg(target_os = "macos")]
        "accessibility" => win::launch("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility", ""),
        #[cfg(target_os = "macos")]
        "input-monitoring" => win::launch("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent", ""),
        #[cfg(not(target_os = "macos"))]
        "startup" => win::launch("ms-settings:startupapps", ""),
        "powertoys" => {
            let path = win::find_process_path("PowerToys.exe")
                .or_else(|| {
                    let local = std::env::var("LOCALAPPDATA").ok()?;
                    let p = format!("{local}\\PowerToys\\PowerToys.exe");
                    std::path::Path::new(&p).is_file().then_some(p)
                })
                .or_else(|| {
                    let pf = std::env::var("ProgramFiles").ok()?;
                    let p = format!("{pf}\\PowerToys\\PowerToys.exe");
                    std::path::Path::new(&p).is_file().then_some(p)
                })
                .ok_or("PowerToys wasn't found. Open it from the Start menu.")?;
            win::launch(&path, "")
        }
        _ => Err("unknown page".into()),
    }
}

/// Mac: ask again for a permission ("keyboard" = Accessibility, "screen" = Screen & System
/// Audio Recording). Clears Cat Key's own stale entry first (an update leaves the old
/// version's switch on, which macOS no longer accepts and won't ask past), then shows
/// macOS's prompt and its page in System Settings. The switch itself is the user's.
/// Returns whether it's allowed right now.
#[tauri::command]
pub fn mac_allow(what: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        match what.as_str() {
            "keyboard" => {
                if win::keyboard_allowed() {
                    return Ok(true);
                }
                win::reset_permissions(&["Accessibility", "ListenEvent", "PostEvent"]);
                let _ = win::launch("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility", "");
                Ok(crate::hook::ask_permission())
            }
            "screen" => {
                if crate::cg::can_record() {
                    return Ok(true);
                }
                win::reset_permissions(&["ScreenCapture"]);
                let ok = crate::cg::ask_to_record();
                if !ok {
                    crate::cg::open_record_settings();
                }
                Ok(ok)
            }
            _ => Err("unknown permission".into()),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = what;
        Ok(true)
    }
}

/// Re-apply "start with Windows" (fixes a missing startup entry).
#[tauri::command]
pub async fn repair_startup(app: AppHandle) -> Result<String, String> {
    let settings = app.state::<AppState>().config.lock().unwrap().settings.clone();
    app::apply_startup(&settings)?;
    Ok(app::startup_status(&settings).to_string())
}

/// Start a fresh copy of Cat Key and quit this one.
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).args(["--wait-pid", &std::process::id().to_string()]).spawn();
    }
    app.exit(0);
}

#[derive(Serialize)]
pub struct KeyLog {
    on: bool,
    lines: Vec<String>,
}

/// Turn the key log on or off ("Settings → Try a shortcut").
#[tauri::command]
pub fn key_log(on: bool) {
    hook::diag_set(on);
}

/// Key log lines since the last call.
#[tauri::command]
pub fn key_log_take() -> KeyLog {
    let (on, lines) = hook::diag_take();
    KeyLog { on, lines }
}

/// Save text the window made (the key log) where the user picks.
#[tauri::command]
pub async fn save_text(app: AppHandle, name: String, text: String) -> Result<bool, String> {
    let ext = name.rsplit('.').next().filter(|e| *e == "json").unwrap_or("txt").to_string();
    let label = if ext == "json" { "Cat Key file" } else { "Text" };
    let Some(file) = app.dialog().file().set_title("Save").set_file_name(&name).add_filter(label, &[ext.as_str()]).blocking_save_file() else {
        return Ok(false);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn open_link(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("only https links".into());
    }
    win::open_url(&url)
}

/// Open an AI app on this computer for Cat Builder. Claude takes the whole request in its own
/// link (claude://claude.ai/new?q=…); otherwise the app opens and the request waits on the
/// clipboard. Answers "filled", "paste" or "missing" (not installed).
#[tauri::command]
pub async fn open_ai_app(app: String, link: Option<String>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let names: &[&str] = match app.as_str() {
            "claude" => &["Claude"],
            "chatgpt" => &["ChatGPT"],
            _ => return Err("unknown app".to_string()),
        };
        if let Some(link) = link.filter(|l| app == "claude" && l.starts_with("claude://")) {
            if let Some((exe, args)) = win::protocol_command("claude", &link) {
                if win::run_as_user(&exe, &args).is_ok() {
                    return Ok("filled".to_string());
                }
            }
            // On a Mac `open` passes a link of any length; on Windows only short ones are safe here.
            let fits = cfg!(target_os = "macos") || link.len() <= 250;
            if fits && win::has_protocol("claude") && win::open_url(&link).is_ok() {
                return Ok("filled".to_string());
            }
        }
        match win::start_app_id(names) {
            Some(id) => win::launch_app_id(&id).map(|_| "paste".to_string()),
            None => Ok("missing".to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
pub struct TextFile {
    name: String,
    text: String,
}

/// Pick a file and read it (Cat Key packs, whole-setup profiles, PowerToys settings).
#[tauri::command]
pub async fn open_text_file(app: AppHandle) -> Result<Option<TextFile>, String> {
    let Some(file) = app
        .dialog()
        .file()
        .set_title("Import")
        .add_filter("Cat Key, PowerToys or text file", &["json", "txt"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() > 8 * 1024 * 1024 {
        return Err("That file is too big to be a shortcut file.".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|_| "That file isn't text Cat Key can read.".to_string())?;
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    Ok(Some(TextFile { name, text }))
}

/// Shortcuts from a PowerToys Keyboard Manager settings file's text.
#[tauri::command]
pub fn powertoys_rules(text: String) -> Result<Vec<Rule>, String> {
    powertoys::import(&text)
}

#[tauri::command]
pub fn clipboard_seq() -> u32 {
    win::clipboard_seq()
}

#[tauri::command]
pub fn read_clipboard() -> Option<String> {
    win::clipboard_text()
}

/// Bring Cat Key to the front (Cat Builder caught an answer).
#[tauri::command]
pub fn focus_window(app: AppHandle) {
    app::show_main(&app);
}

#[derive(serde::Deserialize)]
pub struct TrackItem {
    id: String,
    keys: String,
    #[serde(default)]
    apps: Vec<String>,
}

/// Shortcuts to count when used for real (the ones you're training).
#[tauri::command]
pub fn set_tracked(items: Vec<TrackItem>) {
    let list = items
        .into_iter()
        .filter_map(|t| catkey_core::keys::Shortcut::parse(&t.keys).ok().map(|sc| (t.id, sc, t.apps)))
        .collect::<Vec<_>>();
    hook::set_tracked(list);
}

/// How often each tracked shortcut was used since the last call.
#[tauri::command]
pub fn tracked_counts() -> HashMap<String, u32> {
    hook::take_tracked_counts()
}

// ---------------------------------------------------------------- shortcut coach

/// Shortcuts used, things done the long way, and the time they cost or saved.
#[tauri::command]
pub fn coach_stats() -> serde_json::Value {
    crate::coach::stats_json()
}

/// What each app calls its commands, from the library packs: app exe ("*" for any) → (name, keys).
#[tauri::command]
pub fn set_coach_dict(state: State<AppState>, dict: HashMap<String, Vec<(String, String)>>) {
    crate::coach::set_dict(&state.dir, dict);
}

/// Show a sample tip.
#[tauri::command]
pub fn coach_demo() {
    crate::coach::demo();
}

/// The tip on show (the tip window asks when it has loaded).
#[tauri::command]
pub fn coach_last() -> Option<crate::coach::TipPayload> {
    crate::coach::last_tip()
}

/// Forget one "done the long way" entry.
#[tauri::command]
pub fn coach_forget(id: String) {
    crate::coach::forget_tip(&id);
}

// ---------------------------------------------------------------- purring

/// Start or stop the background purr. Returns whether it purrs now.
#[tauri::command]
pub fn purr_set(app: AppHandle, on: bool) -> bool {
    let now = crate::purr::set(on, None);
    let _ = app.emit("ck://purr", now);
    crate::tray::refresh(&app);
    now
}

#[tauri::command]
pub fn purr_state() -> bool {
    crate::purr::is_on()
}

// ---------------------------------------------------------------- your data folder and backups

/// What a backup holds: everything Cat Key keeps about you, all plain JSON.
const DATA_FILES: &[&str] = &["config.json", "progress.json", "coach.json", "stats.json", "coach-dict.json"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFile {
    name: String,
    bytes: u64,
    /// Last saved, seconds since 1970.
    modified: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataInfo {
    folder: String,
    files: Vec<DataFile>,
}

fn flush_everything(app: &AppHandle) {
    app::save_stats(app);
    crate::coach::flush(true);
}

/// The data folder and what's in it (saved first, so the sizes and times are current).
#[tauri::command]
pub fn data_info(app: AppHandle, state: State<AppState>) -> DataInfo {
    flush_everything(&app);
    let mut files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&state.dir) {
        for e in rd.flatten() {
            let Ok(meta) = e.metadata() else { continue };
            if !meta.is_file() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".tmp") || name == "restored.flag" {
                continue;
            }
            let modified = meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            files.push(DataFile { name, bytes: meta.len(), modified });
        }
    }
    files.sort_by_key(|f| DATA_FILES.iter().position(|n| *n == f.name).unwrap_or(99));
    DataInfo { folder: state.dir.to_string_lossy().to_string(), files }
}

/// Open the data folder in File Explorer.
#[tauri::command]
pub fn open_data_folder(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    flush_everything(&app);
    let _ = std::fs::create_dir_all(&state.dir);
    win::launch(&state.dir.to_string_lossy(), "")
}

/// The window's own records (training, typing, friends, rank), kept in the data
/// folder beside the config, so that one folder holds everything.
#[tauri::command]
pub fn save_progress(state: State<AppState>, text: String) -> Result<(), String> {
    if app::NO_SAVE.load(std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }
    if text.len() > 16 * 1024 * 1024 {
        return Err("progress too large".into());
    }
    app::write_atomic(&state.dir.join("progress.json"), &text)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    text: String,
    /// It came back from a backup: the window should take it as it is.
    restored: bool,
}

#[tauri::command]
pub fn load_progress(state: State<AppState>) -> Option<Progress> {
    let flag = state.dir.join("restored.flag");
    let restored = flag.exists();
    if restored {
        let _ = std::fs::remove_file(&flag);
    }
    std::fs::read_to_string(state.dir.join("progress.json")).ok().map(|text| Progress { text, restored })
}

fn copy_data(from: &std::path::Path, to: &std::path::Path) -> Result<usize, String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    let mut n = 0;
    for f in DATA_FILES {
        let src = from.join(f);
        if src.is_file() {
            std::fs::copy(&src, to.join(f)).map_err(|e| format!("{f}: {e}"))?;
            n += 1;
        }
    }
    Ok(n)
}

/// Copy everything into a new dated folder where the user picks. Returns that folder.
#[tauri::command]
pub async fn backup_now(app: AppHandle) -> Result<Option<String>, String> {
    flush_everything(&app);
    let Some(picked) = app.dialog().file().set_title("Where should the backup go?").blocking_pick_folder() else {
        return Ok(None);
    };
    let root = picked.into_path().map_err(|e| e.to_string())?;
    let dest = root.join(format!("Cat Key backup {}", win::local_stamp()));
    let dir = app.state::<AppState>().dir.clone();
    copy_data(&dir, &dest)?;
    let _ = std::fs::write(
        dest.join("README.txt"),
        "This is a Cat Key backup: your shortcuts, templates, packs, settings, your cat, training and typing progress, friends and stats.\r\n\r\nTo bring it back: Cat Key > Settings > Your data and backups > Restore from a backup, then pick this folder.\r\n",
    );
    Ok(Some(dest.to_string_lossy().to_string()))
}

/// Bring a backup back. What's there now is kept in "Before restore" first. The
/// window then restarts Cat Key, which loads the restored files.
#[tauri::command]
pub async fn restore_backup(app: AppHandle) -> Result<bool, String> {
    let Some(picked) = app.dialog().file().set_title("Pick a Cat Key backup folder").blocking_pick_folder() else {
        return Ok(false);
    };
    let src = picked.into_path().map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(src.join("config.json"))
        .map_err(|_| "That folder isn't a Cat Key backup (there's no config.json in it).".to_string())?;
    serde_json::from_str::<Config>(&text).map_err(|e| format!("The backup's config.json can't be read ({e})."))?;
    flush_everything(&app);
    let dir = app.state::<AppState>().dir.clone();
    copy_data(&dir, &dir.join("Before restore").join(win::local_stamp()))?;
    app::NO_SAVE.store(true, std::sync::atomic::Ordering::SeqCst);
    for f in DATA_FILES {
        let from = src.join(f);
        let to = dir.join(f);
        if from.is_file() {
            std::fs::copy(&from, &to).map_err(|e| format!("{f}: {e}"))?;
        } else if *f != "config.json" {
            let _ = std::fs::remove_file(&to); // not in the backup: start that part fresh
        }
    }
    let _ = std::fs::write(dir.join("restored.flag"), "1");
    Ok(true)
}

/// The tip's Dismiss (and its own timer): hide it for now.
#[tauri::command]
pub fn coach_hide() {
    crate::coach::hide();
}

/// The tip's Ignore: never show this one again (kept in the Ignored list, which can give it back).
#[tauri::command]
pub fn coach_ignore(app: AppHandle, id: String) -> Result<Vec<String>, String> {
    crate::coach::hide();
    let state = app.state::<AppState>();
    let mut cfg = state.config.lock().unwrap().clone();
    let id = id.to_lowercase();
    if !cfg.settings.coach_muted.iter().any(|m| m.to_lowercase() == id) {
        cfg.settings.coach_muted.push(id);
    }
    let cfg = app::apply_config(&app, cfg)?;
    let _ = app.emit("ck://coach-muted", &cfg.settings.coach_muted);
    Ok(cfg.settings.coach_muted)
}

static PENDING_ADJUST: std::sync::Mutex<Option<crate::coach::TipPayload>> = std::sync::Mutex::new(None);

/// The tip's Adjust: open Cat Key with a new shortcut for that action, in that app.
#[tauri::command]
pub fn coach_adjust(app: AppHandle, id: String) -> Result<(), String> {
    crate::coach::hide();
    let tip = crate::coach::tip_info(&id).ok_or("That tip isn't known any more.")?;
    if let Ok(mut g) = PENDING_ADJUST.lock() {
        *g = Some(tip.clone());
    }
    app::show_main(&app);
    let _ = app.emit_to("main", "ck://coach-adjust", &tip);
    Ok(())
}

/// An Adjust that arrived while the window was still opening.
#[tauri::command]
pub fn coach_take_adjust() -> Option<crate::coach::TipPayload> {
    PENDING_ADJUST.lock().ok().and_then(|mut g| g.take())
}

/// Training: keep every key for the trainer while a round runs (see hook::set_capture).
#[tauri::command]
pub fn set_capture(on: bool) {
    hook::set_capture(on);
}

// ---------------------------------------------------------------- the MCP connection (AI apps)

/// Packs an AI app sent (through `CatKey --mcp`), taken out of the inbox.
#[tauri::command]
pub fn take_inbox() -> Vec<String> {
    crate::mcp::take_inbox()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpInfo {
    /// Claude Desktop is installed / already starts this Cat Key.
    claude_installed: bool,
    claude_connected: bool,
    /// The setup for any MCP app, and the line for Claude Code.
    entry: String,
    claude_code: String,
    /// Cat Key runs from a place that won't last (a disk image, a download that macOS moved).
    temp_place: bool,
}

#[tauri::command]
pub fn mcp_info() -> McpInfo {
    let (claude_installed, claude_connected) = crate::mcp::claude_status();
    let entry = crate::mcp::server_entry();
    let exe = entry["command"].as_str().unwrap_or("").to_string();
    McpInfo {
        claude_installed,
        claude_connected,
        entry: serde_json::to_string_pretty(&serde_json::json!({ "mcpServers": { "catkey": entry } })).unwrap_or_default(),
        claude_code: format!("claude mcp add catkey -- \"{exe}\" --mcp"),
        temp_place: win::running_from_temp_place(),
    }
}

#[tauri::command]
pub fn mcp_connect() -> Result<(), String> {
    crate::mcp::connect_claude()
}
