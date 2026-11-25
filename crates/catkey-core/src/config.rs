//! The saved configuration (`config.json`) and its compiled, lookup-friendly form.

use crate::keys::{is_modifier, ModSpec, Shortcut};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub const CONFIG_VERSION: u32 = 1;

fn yes() -> bool {
    true
}
fn default_version() -> u32 {
    CONFIG_VERSION
}
fn default_theme() -> String {
    "auto".into()
}
fn default_skin() -> String {
    "mochi".into()
}
fn default_cat_name() -> String {
    "Mochi".into()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    /// Master switch: false = Cat Key is napping, every key passes through untouched.
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "yes")]
    pub launch_at_startup: bool,
    /// Run elevated so shortcuts also work inside apps running as administrator.
    #[serde(default)]
    pub admin_mode: bool,
    /// "auto" | "light" | "dark"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// "mochi" | "kuro" | "tora" | "sora"
    #[serde(default = "default_skin")]
    pub skin: String,
    #[serde(default = "default_cat_name")]
    pub cat_name: String,
    #[serde(default)]
    pub sounds: bool,
    /// Optional global shortcut that pauses/resumes every remap.
    #[serde(default)]
    pub pause_hotkey: Option<Shortcut>,
    /// Apps where Cat Key does nothing at all (games…), as lowercase exe names.
    #[serde(default)]
    pub excluded_apps: Vec<String>,
    /// The PowerToys warning was dismissed.
    #[serde(default)]
    pub hide_powertoys_tip: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            launch_at_startup: true,
            admin_mode: false,
            theme: default_theme(),
            skin: default_skin(),
            cat_name: default_cat_name(),
            sounds: false,
            pause_hotkey: None,
            excluded_apps: Vec::new(),
            hide_powertoys_tip: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(default)]
    pub id: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Optional friendly label ("Copy", "Open terminal"…).
    #[serde(default)]
    pub name: String,
    pub trigger: Shortcut,
    pub action: Action,
    #[serde(default)]
    pub scope: Scope,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    /// Press another key or shortcut.
    Keys { keys: Shortcut },
    /// Open a program, file, folder or URL.
    Launch {
        target: String,
        #[serde(default)]
        args: String,
    },
    /// Type a piece of text.
    Text { text: String },
    /// Swallow the shortcut so nothing happens.
    Block,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ScopeMode {
    #[default]
    All,
    Only,
    Except,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    #[serde(default)]
    pub mode: ScopeMode,
    /// Lowercase exe names, e.g. "chrome.exe".
    #[serde(default)]
    pub apps: Vec<String>,
}

impl Scope {
    pub fn all() -> Scope {
        Scope::default()
    }
    pub fn only(apps: &[&str]) -> Scope {
        Scope { mode: ScopeMode::Only, apps: apps.iter().map(|a| normalize_app(a)).collect() }
    }
}

/// "Chrome" / "C:\\…\\chrome.exe" / "chrome.EXE" -> "chrome.exe"
pub fn normalize_app(app: &str) -> String {
    let a = app.trim().trim_matches('"');
    let file = a.rsplit(['\\', '/']).next().unwrap_or(a).trim().to_lowercase();
    if file.is_empty() {
        return file;
    }
    if file.contains('.') {
        file
    } else {
        format!("{file}.exe")
    }
}

static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

pub fn new_id() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    format!("r{:x}{:x}", t, ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn rule(trigger: &str, keys: &str, name: &str) -> Rule {
    Rule {
        id: new_id(),
        enabled: true,
        name: name.into(),
        trigger: Shortcut::parse(trigger).expect("default trigger"),
        action: Action::Keys { keys: Shortcut::parse(keys).expect("default keys") },
        scope: Scope::all(),
    }
}

/// The starter pack: the user's Mac-style Alt shortcuts.
pub fn default_rules() -> Vec<Rule> {
    vec![
        rule("LAlt+W", "LCtrl+W", "Close tab"),
        rule("LAlt+C", "LCtrl+C", "Copy"),
        rule("LAlt+V", "LCtrl+V", "Paste"),
        rule("LAlt+N", "Ctrl+N", "New"),
        rule("LAlt+Q", "LAlt+F4", "Quit app"),
        rule("LAlt+F", "LCtrl+F", "Find"),
        rule("LAlt+A", "LCtrl+A", "Select all"),
        rule("LAlt+LShift+N", "LCtrl+LShift+N", "New window (private)"),
        rule("LAlt+S", "LCtrl+S", "Save"),
        rule("LAlt+Z", "LCtrl+Z", "Undo"),
        rule("LAlt+R", "LCtrl+R", "Reload"),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Config { version: CONFIG_VERSION, enabled: true, rules: default_rules(), settings: Settings::default() }
    }
}

impl Config {
    /// Fill ids, tidy app names, clamp odd values. Safe to call repeatedly.
    pub fn normalize(&mut self) {
        self.version = CONFIG_VERSION;
        let mut seen = std::collections::HashSet::new();
        for r in &mut self.rules {
            if r.id.trim().is_empty() || !seen.insert(r.id.clone()) {
                r.id = new_id();
                seen.insert(r.id.clone());
            }
            r.name = r.name.trim().chars().take(60).collect();
            r.scope.apps = tidy_apps(&r.scope.apps);
            if r.scope.apps.is_empty() {
                r.scope.mode = ScopeMode::All;
            }
            if let Action::Launch { target, args } = &mut r.action {
                *target = target.trim().to_string();
                *args = args.trim().to_string();
            }
            if let Action::Text { text } = &mut r.action {
                if text.chars().count() > 4000 {
                    *text = text.chars().take(4000).collect();
                }
            }
        }
        self.settings.excluded_apps = tidy_apps(&self.settings.excluded_apps);
        if !matches!(self.settings.theme.as_str(), "auto" | "light" | "dark") {
            self.settings.theme = default_theme();
        }
        if !matches!(self.settings.skin.as_str(), "mochi" | "kuro" | "tora" | "sora") {
            self.settings.skin = default_skin();
        }
        self.settings.cat_name = self.settings.cat_name.trim().chars().take(24).collect();
        if self.settings.cat_name.is_empty() {
            self.settings.cat_name = default_cat_name();
        }
        if let Some(p) = self.settings.pause_hotkey {
            if is_modifier(p.key) {
                self.settings.pause_hotkey = None;
            }
        }
    }
}

fn tidy_apps(apps: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in apps {
        let n = normalize_app(a);
        if !n.is_empty() && !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

/// Why a rule cannot run (shown in the UI); `None` = fine.
pub fn rule_problem(r: &Rule) -> Option<String> {
    let t = r.trigger;
    let single = t.mods.is_empty();
    match &r.action {
        Action::Keys { keys } => {
            if single && keys.mods.is_empty() {
                return None; // plain key remap, modifiers allowed on both sides
            }
            if is_modifier(t.key) {
                return Some("A shortcut needs a regular key (not just Ctrl/Alt/Shift/Win).".into());
            }
            if is_modifier(keys.key) {
                return Some("The result needs a regular key, e.g. Ctrl + C.".into());
            }
            None
        }
        Action::Block => None,
        Action::Launch { target, .. } if target.trim().is_empty() => Some("Pick something to open.".into()),
        Action::Text { text } if text.is_empty() => Some("Type the text to send.".into()),
        _ => {
            if is_modifier(t.key) {
                Some("Pick a regular key for this shortcut (not just Ctrl/Alt/Shift/Win).".into())
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Compiled form, used by the engine on every key press.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum CScope {
    All,
    Only(Arc<[String]>),
    Except(Arc<[String]>),
}

impl CScope {
    pub fn needs_app(&self) -> bool {
        !matches!(self, CScope::All)
    }
    pub fn matches(&self, exe: &str) -> bool {
        match self {
            CScope::All => true,
            CScope::Only(apps) => apps.iter().any(|a| a == exe),
            CScope::Except(apps) => !apps.iter().any(|a| a == exe),
        }
    }
    fn rank(&self) -> u8 {
        match self {
            CScope::Only(_) => 0,
            CScope::Except(_) => 1,
            CScope::All => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CAction {
    Keys { mods: ModSpec, key: u8 },
    Launch { target: Arc<str>, args: Arc<str> },
    Text { text: Arc<str> },
    Block,
}

/// A shortcut rule (exact modifier match).
#[derive(Clone, Debug)]
pub struct CRule {
    pub index: u32,
    pub mods: ModSpec,
    pub scope: CScope,
    pub action: CAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KAction {
    /// Becomes another key (may be a modifier, e.g. CapsLock -> Ctrl).
    Key(u8),
    /// The key does nothing.
    Block,
}

/// A single-key remap: works whatever modifiers are held.
#[derive(Clone, Debug)]
pub struct CKeyRemap {
    pub index: u32,
    pub scope: CScope,
    pub action: KAction,
}

#[derive(Clone, Debug)]
pub struct Compiled {
    /// Shortcut rules by trigger key, most specific scope first.
    pub shortcuts: Vec<Vec<CRule>>,
    /// Single-key remaps by key, most specific scope first.
    pub remaps: Vec<Vec<CKeyRemap>>,
    pub excluded: Arc<[String]>,
    pub pause_hotkey: Option<Shortcut>,
    /// Rule index -> rule id (for stats and UI feedback).
    pub rule_ids: Vec<String>,
}

impl Default for Compiled {
    fn default() -> Self {
        Compiled {
            shortcuts: vec![Vec::new(); 256],
            remaps: vec![Vec::new(); 256],
            excluded: Arc::from(Vec::new()),
            pause_hotkey: None,
            rule_ids: Vec::new(),
        }
    }
}

impl Compiled {
    pub fn rule_count(&self) -> usize {
        self.shortcuts.iter().map(|v| v.len()).sum::<usize>() + self.remaps.iter().map(|v| v.len()).sum::<usize>()
    }
}

fn compile_scope(s: &Scope) -> CScope {
    if s.apps.is_empty() {
        return CScope::All;
    }
    let apps: Arc<[String]> = s.apps.iter().map(|a| normalize_app(a)).collect::<Vec<_>>().into();
    match s.mode {
        ScopeMode::All => CScope::All,
        ScopeMode::Only => CScope::Only(apps),
        ScopeMode::Except => CScope::Except(apps),
    }
}

/// The hook reports left/right keys, so a generic "Alt" trigger means both.
fn trigger_keys(vk: u8) -> Vec<u8> {
    use crate::keys::*;
    match vk {
        VK_SHIFT => vec![VK_LSHIFT, VK_RSHIFT],
        VK_CONTROL => vec![VK_LCONTROL, VK_RCONTROL],
        VK_MENU => vec![VK_LMENU, VK_RMENU],
        other => vec![other],
    }
}

pub fn compile(cfg: &Config) -> Compiled {
    let mut c = Compiled::default();
    c.rule_ids = cfg.rules.iter().map(|r| r.id.clone()).collect();
    c.excluded = cfg.settings.excluded_apps.iter().map(|a| normalize_app(a)).collect::<Vec<_>>().into();
    c.pause_hotkey = cfg.settings.pause_hotkey.filter(|p| !is_modifier(p.key) && p.key != 0);

    for (i, r) in cfg.rules.iter().enumerate() {
        if !r.enabled || r.trigger.key == 0 || rule_problem(r).is_some() {
            continue;
        }
        let index = i as u32;
        let scope = compile_scope(&r.scope);
        let t = r.trigger;
        let single = t.mods.is_empty();
        match &r.action {
            Action::Keys { keys } if single && keys.mods.is_empty() => {
                for k in trigger_keys(t.key) {
                    c.remaps[k as usize].push(CKeyRemap { index, scope: scope.clone(), action: KAction::Key(keys.key) });
                }
            }
            Action::Block if single => {
                for k in trigger_keys(t.key) {
                    c.remaps[k as usize].push(CKeyRemap { index, scope: scope.clone(), action: KAction::Block });
                }
            }
            action => {
                let action = match action {
                    Action::Keys { keys } => CAction::Keys { mods: keys.mods, key: keys.key },
                    Action::Launch { target, args } => {
                        CAction::Launch { target: Arc::from(target.as_str()), args: Arc::from(args.as_str()) }
                    }
                    Action::Text { text } => CAction::Text { text: Arc::from(text.as_str()) },
                    Action::Block => CAction::Block,
                };
                c.shortcuts[t.key as usize].push(CRule { index, mods: t.mods, scope, action });
            }
        }
    }
    for list in &mut c.shortcuts {
        list.sort_by_key(|r| r.scope.rank());
    }
    for list in &mut c.remaps {
        list.sort_by_key(|r| r.scope.rank());
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_json() {
        let mut cfg = Config::default();
        cfg.rules.push(Rule {
            id: String::new(),
            enabled: true,
            name: "Terminal".into(),
            trigger: Shortcut::parse("LAlt+T").unwrap(),
            action: Action::Launch { target: "wt.exe".into(), args: String::new() },
            scope: Scope::only(&["Chrome"]),
        });
        cfg.normalize();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("\"trigger\": \"LAlt+LShift+N\""));
        assert!(json.contains("\"chrome.exe\""));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
        let c = compile(&back);
        assert_eq!(c.rule_count(), 12);
        assert_eq!(c.shortcuts[b'T' as usize].len(), 1);
    }

    #[test]
    fn minimal_json_gets_defaults() {
        let cfg: Config = serde_json::from_str(r#"{"rules":[{"trigger":"CapsLock","action":{"type":"keys","keys":"Esc"}}]}"#).unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.settings.theme, "auto");
        let c = compile(&cfg);
        assert_eq!(c.remaps[0x14].len(), 1);
    }

    #[test]
    fn app_specific_sorted_first() {
        let mut cfg = Config { rules: vec![], ..Config::default() };
        cfg.rules.push(rule("LAlt+F", "LCtrl+F", "Find"));
        let mut r = rule("LAlt+F", "LCtrl+L", "Address bar");
        r.scope = Scope::only(&["chrome.exe"]);
        cfg.rules.push(r);
        let c = compile(&cfg);
        let list = &c.shortcuts[b'F' as usize];
        assert!(matches!(list[0].scope, CScope::Only(_)));
        assert!(matches!(list[1].scope, CScope::All));
    }

    #[test]
    fn invalid_rules_are_skipped() {
        let mut cfg = Config { rules: vec![], ..Config::default() };
        cfg.rules.push(rule("LAlt+LShift", "LCtrl+C", "bad"));
        cfg.rules.push(Rule {
            id: "x".into(),
            enabled: true,
            name: String::new(),
            trigger: Shortcut::parse("LAlt+O").unwrap(),
            action: Action::Launch { target: " ".into(), args: String::new() },
            scope: Scope::all(),
        });
        assert_eq!(compile(&cfg).rule_count(), 0);
    }
}
