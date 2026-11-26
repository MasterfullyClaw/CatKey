//! The shortcut coach on a Mac. The tips (reading what you clicked) are Windows-only for
//! now; this side keeps the counts the Cat Warrior page shows: shortcuts used per app per
//! day, saved in coach.json just like on Windows.

use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;

const LOST_MENU_MS: u64 = 2_900;
const LOST_BUTTON_MS: u64 = 1_600;
pub const SAVED_PER_SHORTCUT_MS: u64 = LOST_BUTTON_MS;

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
}

static DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static STATS: Mutex<Option<serde_json::Value>> = Mutex::new(None);
static COMBOS: Mutex<Vec<(String, u32)>> = Mutex::new(Vec::new());

fn load(dir: &Path) -> serde_json::Value {
    std::fs::read_to_string(dir.join("coach.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .filter(|v: &serde_json::Value| v.is_object())
        .unwrap_or_else(|| serde_json::json!({ "days": {}, "tips": {} }))
}

pub fn start(_app: AppHandle, dir: PathBuf, _s: &catkey_core::Settings) {
    let stats = load(&dir);
    if let Ok(mut g) = STATS.lock() {
        *g = Some(stats);
    }
    if let Ok(mut g) = DIR.lock() {
        *g = Some(dir);
    }
}

pub fn set_settings(_s: &catkey_core::Settings) {}

/// The app's own names for its commands, kept for later (the Windows coach uses them).
pub fn set_dict(dir: &Path, dict: HashMap<String, Vec<(String, String)>>) {
    let map: BTreeMap<String, BTreeMap<String, String>> = dict
        .into_iter()
        .map(|(app, items)| (app.to_lowercase(), items.into_iter().collect()))
        .collect();
    if let Ok(text) = serde_json::to_string(&map) {
        let _ = crate::app::write_atomic(&dir.join("coach-dict.json"), &text);
    }
}

/// A shortcut (⌘, ⌥ or ⌃ with a key) used in this app.
pub fn count_combo(app: &str) {
    if app.is_empty() || app.starts_with("com.masterfully.catkey") {
        return;
    }
    if let Ok(mut c) = COMBOS.try_lock() {
        match c.iter_mut().find(|(a, _)| a == app) {
            Some((_, n)) => *n += 1,
            None => c.push((app.to_string(), 1)),
        }
    }
}

pub fn flush(force: bool) {
    let combos: Vec<(String, u32)> = COMBOS.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default();
    let today = crate::win::local_date();
    let snapshot = {
        let Ok(mut g) = STATS.lock() else { return };
        let Some(stats) = g.as_mut() else { return };
        if combos.is_empty() && !force {
            return;
        }
        for (app, n) in combos {
            let day = &mut stats["days"][&today][&app];
            let used = day.get("used").and_then(|v| v.as_u64()).unwrap_or(0) + n as u64;
            day["used"] = used.into();
            if day.get("caught").is_none() {
                day["caught"] = 0.into();
                day["lostMs"] = 0.into();
            }
        }
        stats.clone()
    };
    if crate::app::NO_SAVE.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let dir = DIR.lock().ok().and_then(|g| g.clone());
    if let (Some(dir), Ok(text)) = (dir, serde_json::to_string(&snapshot)) {
        let _ = crate::app::write_atomic(&dir.join("coach.json"), &text);
    }
}

pub fn stats_json() -> serde_json::Value {
    flush(false);
    let mut v = STATS.lock().ok().and_then(|g| g.clone()).unwrap_or_else(|| serde_json::json!({ "days": {}, "tips": {} }));
    v["savedPerShortcutMs"] = SAVED_PER_SHORTCUT_MS.into();
    v["lostMenuMs"] = LOST_MENU_MS.into();
    v["lostButtonMs"] = LOST_BUTTON_MS.into();
    v["available"] = false.into();
    v
}

pub fn forget_tip(id: &str) {
    if let Ok(mut g) = STATS.lock() {
        if let Some(tips) = g.as_mut().and_then(|s| s.get_mut("tips")).and_then(|t| t.as_object_mut()) {
            tips.remove(id);
        }
    }
}

pub fn last_tip() -> Option<TipPayload> {
    None
}

pub fn demo() {}
