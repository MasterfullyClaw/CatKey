//! App state, config files, the effects dispatcher and window management.

use crate::{hook, tray, win};
use catkey_core::engine::Effect;
use catkey_core::{compile, Compiled, Config, Settings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// True while the window (and its webview) exists.
pub static UI_OPEN: AtomicBool = AtomicBool::new(false);

pub const POWERTOYS_ENGINE: &str = "PowerToys.KeyboardManagerEngine.exe";

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    #[serde(default)]
    pub day: String,
    #[serde(default)]
    pub today: u64,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub per_rule: HashMap<String, u64>,
    #[serde(skip)]
    pub dirty: bool,
}

pub struct AppState {
    pub dir: PathBuf,
    pub config: Mutex<Config>,
    pub compiled: Mutex<Arc<Compiled>>,
    pub stats: Mutex<Stats>,
    pub first_run: bool,
    pub config_reset: bool,
    /// config.json came from an older Cat Key and was brought up to date (the UI says so once).
    pub upgraded: AtomicBool,
    pub icons: Mutex<HashMap<String, Option<String>>>,
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("CatKey")
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn save_config_file(dir: &Path, cfg: &Config) -> Result<(), String> {
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    write_atomic(&dir.join("config.json"), &text)
}

/// (config, first run?, was the file broken and replaced?)
/// (config, first run, file was damaged, upgraded from an older version)
fn load_config(dir: &Path) -> (Config, bool, bool, bool) {
    let path = dir.join("config.json");
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Config>(&text) {
            Ok(mut c) => {
                let upgraded = c.migrate();
                c.normalize();
                (c, false, false, upgraded)
            }
            Err(e) => {
                crate::log(&format!("config.json could not be read ({e}); keeping a copy and starting fresh"));
                let _ = std::fs::copy(&path, dir.join("config.broken.json"));
                (Config::default(), false, true, false)
            }
        },
        Err(_) => (Config::default(), true, false, false),
    }
}

fn load_stats(dir: &Path) -> Stats {
    std::fs::read_to_string(dir.join("stats.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_stats(app: &AppHandle) {
    let state = app.state::<AppState>();
    let snapshot = {
        let mut s = state.stats.lock().unwrap();
        if !s.dirty {
            return;
        }
        s.dirty = false;
        s.clone()
    };
    if let Ok(text) = serde_json::to_string(&snapshot) {
        let _ = write_atomic(&state.dir.join("stats.json"), &text);
    }
}

fn bump_stats(app: &AppHandle, rule_id: Option<&str>) {
    let state = app.state::<AppState>();
    let mut s = state.stats.lock().unwrap();
    let today = win::local_date();
    if s.day != today {
        s.day = today;
        s.today = 0;
    }
    s.today += 1;
    s.total += 1;
    if let Some(id) = rule_id {
        *s.per_rule.entry(id.to_string()).or_insert(0) += 1;
    }
    s.dirty = true;
}

/// Apply a config from the UI: normalize, hand to the keyboard thread, save.
pub fn apply_config(app: &AppHandle, cfg: Config) -> Result<Config, String> {
    let state = app.state::<AppState>();
    let mut cfg = cfg;
    cfg.normalize();
    let compiled = Arc::new(compile(&cfg));
    hook::send(hook::Cmd::Config(compiled.clone()));
    hook::send(hook::Cmd::Enabled(cfg.enabled));
    *state.compiled.lock().unwrap() = compiled;
    let old = {
        let mut g = state.config.lock().unwrap();
        std::mem::replace(&mut *g, cfg.clone())
    };
    save_config_file(&state.dir, &cfg)?;
    if old.settings.launch_at_startup != cfg.settings.launch_at_startup
        || old.settings.admin_mode != cfg.settings.admin_mode
    {
        let s = cfg.settings.clone();
        std::thread::spawn(move || {
            if let Err(e) = apply_startup(&s) {
                crate::log(&format!("startup setting: {e}"));
            }
        });
    }
    tray::refresh(app);
    Ok(cfg)
}

pub fn set_enabled(app: &AppHandle, enabled: bool) {
    let mut cfg = app.state::<AppState>().config.lock().unwrap().clone();
    if cfg.enabled == enabled {
        return;
    }
    cfg.enabled = enabled;
    if apply_config(app, cfg).is_ok() {
        let _ = app.emit("ck://enabled", enabled);
    }
}

pub fn toggle_enabled(app: &AppHandle) {
    let on = app.state::<AppState>().config.lock().unwrap().enabled;
    set_enabled(app, !on);
}

/// Make "start with Windows" match the settings (Run key, or a logon task in admin mode).
pub fn apply_startup(settings: &Settings) -> Result<(), String> {
    let elevated = win::is_elevated();
    if !settings.launch_at_startup {
        win::set_run_entry(None);
        if elevated {
            win::delete_admin_task()?;
        }
        return Ok(());
    }
    if settings.admin_mode && elevated {
        if !win::admin_task_exists() {
            win::create_admin_task()?;
        }
        win::set_run_entry(None);
    } else {
        let cmd = win::autostart_command();
        if win::get_run_entry().as_deref() != Some(cmd.as_str()) && !win::set_run_entry(Some(&cmd)) {
            return Err("couldn't write the startup entry".into());
        }
        if elevated && !settings.admin_mode {
            win::delete_admin_task()?;
        }
    }
    Ok(())
}

/// "on" | "off" | "blocked" (Windows switched it off in Startup apps) | "missing"
pub fn startup_status(settings: &Settings) -> &'static str {
    if !settings.launch_at_startup {
        return "off";
    }
    if settings.admin_mode && win::is_elevated() {
        return if win::admin_task_exists() { "on" } else { "missing" };
    }
    match win::get_run_entry() {
        Some(cmd) if cmd == win::autostart_command() => {
            if win::startup_approved() == Some(false) {
                "blocked"
            } else {
                "on"
            }
        }
        _ => "missing",
    }
}

static CREATING: AtomicBool = AtomicBool::new(false);

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    if CREATING.swap(true, Ordering::SeqCst) {
        return; // already being created
    }
    // On Windows, creating a webview from the main thread inside an event handler
    // (tray click, menu, second-instance callback) deadlocks. Build it from a thread.
    let app = app.clone();
    std::thread::spawn(move || {
        build_main(&app);
        CREATING.store(false, Ordering::SeqCst);
    });
}

fn build_main(app: &AppHandle) {
    let built = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Cat Key")
        .inner_size(1040.0, 720.0)
        .min_inner_size(820.0, 600.0)
        .decorations(false)
        .shadow(true)
        .resizable(true)
        .center()
        .visible(false)
        .background_color(tauri::window::Color(22, 18, 29, 255))
        .build();
    match built {
        Ok(w) => {
            UI_OPEN.store(true, Ordering::SeqCst);
            w.on_window_event(|e| {
                if let WindowEvent::Destroyed = e {
                    UI_OPEN.store(false, Ordering::SeqCst);
                    hook::send(hook::Cmd::StopRecord);
                }
            });
            // If the page is slow to say it's ready, show the window anyway.
            std::thread::sleep(Duration::from_millis(1800));
            if !w.is_visible().unwrap_or(false) {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        Err(e) => crate::log(&format!("could not open the window: {e}")),
    }
}

#[derive(Serialize, Clone)]
struct RecordPayload {
    phase: &'static str,
    shortcut: Option<String>,
}

fn start_dispatcher(app: AppHandle, rx: Receiver<Effect>) {
    std::thread::Builder::new()
        .name("catkey-effects".into())
        .spawn(move || {
            unsafe {
                use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
                CoInitializeEx(std::ptr::null(), (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32);
            }
            while let Ok(e) = rx.recv() {
                match e {
                    Effect::Fired { rule } => {
                        let id = {
                            let state = app.state::<AppState>();
                            let c = state.compiled.lock().unwrap();
                            c.rule_ids.get(rule as usize).cloned()
                        };
                        bump_stats(&app, id.as_deref());
                        if UI_OPEN.load(Ordering::Relaxed) {
                            let _ = app.emit("ck://fired", id);
                        }
                    }
                    Effect::Launch { target, args } => {
                        if let Err(err) = win::launch(&target, &args) {
                            crate::log(&format!("open {target}: {err}"));
                            let _ = app.emit("ck://error", format!("Couldn't open {target}: {err}"));
                        }
                    }
                    Effect::Toggled { enabled } => {
                        let a = app.clone();
                        // Called from the keyboard thread's pause hotkey: config + tray + UI.
                        let mut cfg = a.state::<AppState>().config.lock().unwrap().clone();
                        cfg.enabled = enabled;
                        let _ = apply_config(&a, cfg);
                        let _ = a.emit("ck://enabled", enabled);
                    }
                    Effect::RecordUpdate { shortcut } => {
                        let _ = app.emit("ck://record", RecordPayload { phase: "update", shortcut: Some(shortcut) });
                    }
                    Effect::RecordDone { shortcut } => {
                        let _ = app.emit("ck://record", RecordPayload { phase: "done", shortcut });
                    }
                    Effect::RecordCancelled => {
                        let _ = app.emit("ck://record", RecordPayload { phase: "cancel", shortcut: None });
                    }
                }
            }
        })
        .expect("start effects thread");
}

pub fn run(args: Vec<String>) {
    let autostart = args.iter().any(|a| a == "--autostart");
    win::opt_out_of_power_throttling_for_process();
    let dir = config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let (cfg, first_run, config_reset, upgraded) = load_config(&dir);

    if autostart && !cfg.settings.launch_at_startup {
        // Started by a leftover startup entry after the user turned it off.
        let _ = apply_startup(&cfg.settings);
        return;
    }
    // A second copy only wakes the first one up (single-instance plugin) and exits.
    let second = win::another_instance_running();
    if second {
        // We were started by the user, so we may hand the foreground to the first copy.
        win::allow_any_foreground();
    }
    if (first_run || upgraded) && !second {
        // Save the starter set (or an upgrade) once, so it isn't offered again after the user changes it.
        if let Err(e) = save_config_file(&dir, &cfg) {
            crate::log(&format!("couldn't save config.json: {e}"));
        }
    }

    // The keyboard works from here on, before any window exists.
    let compiled = Arc::new(compile(&cfg));
    let (tx, rx) = mpsc::channel::<Effect>();
    if !second {
        hook::start(compiled.clone(), cfg.enabled, tx);
    }

    let settings = cfg.settings.clone();
    let state = AppState {
        stats: Mutex::new(load_stats(&dir)),
        dir,
        config: Mutex::new(cfg),
        compiled: Mutex::new(compiled),
        first_run,
        config_reset,
        upgraded: AtomicBool::new(upgraded),
        icons: Mutex::new(HashMap::new()),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if !argv.iter().any(|a| a == "--autostart") {
                show_main(app);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            crate::commands::get_state,
            crate::commands::get_status,
            crate::commands::get_stats,
            crate::commands::save_config,
            crate::commands::set_enabled,
            crate::commands::start_recording,
            crate::commands::stop_recording,
            crate::commands::running_apps,
            crate::commands::app_icon,
            crate::commands::pick_program,
            crate::commands::import_powertoys,
            crate::commands::import_file,
            crate::commands::export_file,
            crate::commands::restore_defaults,
            crate::commands::set_admin_mode,
            crate::commands::restart_normal,
            crate::commands::ui_ready,
            crate::commands::window_minimize,
            crate::commands::window_toggle_maximize,
            crate::commands::window_close,
            crate::commands::quit_app,
            crate::commands::open_link,
            crate::commands::open_system_page,
            crate::commands::repair_startup,
            crate::commands::restart_app,
            crate::commands::key_log,
            crate::commands::key_log_take,
            crate::commands::save_text,
        ])
        .setup(move |app| {
            if second {
                std::process::exit(0);
            }
            let handle = app.handle().clone();
            tray::create(&handle)?;
            start_dispatcher(handle.clone(), rx);
            let saver = handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(120));
                save_stats(&saver);
            });
            std::thread::spawn(move || {
                if let Err(e) = apply_startup(&settings) {
                    crate::log(&format!("startup setting: {e}"));
                }
            });
            if !autostart {
                show_main(&handle);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Cat Key failed to start");

    app.run(|app, event| match event {
        RunEvent::ExitRequested { code, api, .. } => {
            // Closing the window just hides Cat Key in the tray; only Quit exits.
            if code.is_none() {
                api.prevent_exit();
            }
        }
        RunEvent::Exit => {
            hook::send(hook::Cmd::Quit);
            save_stats(app);
            std::thread::sleep(Duration::from_millis(60));
        }
        _ => {}
    });
}
