//! Commands the UI calls through `invoke`.

use crate::app::{self, AppState, Stats, POWERTOYS_ENGINE};
use crate::{hook, win};
use catkey_core::config::rule_problem;
use catkey_core::{builtin_templates, powertoys, Config, Template};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    version: String,
    admin: bool,
    powertoys_running: bool,
    first_run: bool,
    config_reset: bool,
    config_dir: String,
    /// Rule id -> why it can't run (shown on the card).
    problems: HashMap<String, String>,
    /// The keyboard hook is installed and its thread is alive.
    engine_running: bool,
    /// "on" | "off" | "blocked" (Windows turned it off in Startup apps) | "missing"
    startup: String,
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
        first_run: state.first_run,
        config_reset: state.config_reset,
        config_dir: state.dir.to_string_lossy().to_string(),
        problems,
        engine_running: hook::engine_running(),
        startup: app::startup_status(&settings).to_string(),
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
        .set_title("Pick an app or file")
        .add_filter("Apps and shortcuts", &["exe", "lnk", "bat", "cmd", "url"])
        .add_filter("Any file", &["*"])
        .blocking_pick_file()?;
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

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn open_link(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("only https links".into());
    }
    win::launch(&url, "")
}
