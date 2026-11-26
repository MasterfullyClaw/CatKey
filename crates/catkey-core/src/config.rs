//! The saved configuration (`config.json`) and its compiled, lookup-friendly form.

use crate::keys::{is_modifier, ModSpec, Shortcut};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// 2 = 1.0.2: Alt+T (new tab) joined the starter set; user templates.
/// 3 = 1.2.0: cat sounds are on by default.
pub const CONFIG_VERSION: u32 = 3;

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
fn default_purr_volume() -> u32 {
    45
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
    /// Templates the user saved. The built-in ones live in code (`builtin_templates`).
    #[serde(default)]
    pub templates: Vec<Template>,
    /// Library packs the user saved (imported, made with an AI, shared with them),
    /// kept as their catkey-pack JSON. The built-in packs ship with the app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<serde_json::Value>,
}

/// A named set of shortcuts that can be added to (or removed from) the list in one go.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Ships with Cat Key: always there, can't be renamed, changed or deleted.
    #[serde(default)]
    pub builtin: bool,
    #[serde(default)]
    pub rules: Vec<Rule>,
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
    #[serde(default = "yes")]
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
    /// Shortcut coach: a short tip when something with a shortcut was done the long way.
    #[serde(default = "yes")]
    pub coach: bool,
    /// Also coach in Chrome-based apps (asking them what's under the pointer costs them some work).
    #[serde(default)]
    pub coach_chromium: bool,
    /// Tips the user turned off ("app.exe|name", or "app.exe|*" for a whole app).
    #[serde(default)]
    pub coach_muted: Vec<String>,
    /// Background purr volume, 0–100.
    #[serde(default = "default_purr_volume")]
    pub purr_volume: u32,
    /// The cat's look, kept for the window only: custom cats, outfits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cat_look: Option<serde_json::Value>,
    /// Quick setting: Alt works as Ctrl and Ctrl as Alt, everywhere (off unless chosen).
    #[serde(default)]
    pub swap_alt_ctrl: bool,
    /// Where the coach's tips appear on screen.
    #[serde(default)]
    pub coach_pos: CoachPos,
    /// Sounds by part ("ui", "cat", "training", "tips", "tools"): false = that part is quiet.
    /// Missing parts are on. The master switch is `sounds`.
    #[serde(default)]
    pub sound_parts: std::collections::BTreeMap<String, bool>,
    /// The Cat Dock: the cat ball on the screen edge and what it keeps.
    #[serde(default)]
    pub dock: DockSettings,
    /// Paw prints where you click, and the cat-paw pointer.
    #[serde(default)]
    pub paws: PawSettings,
    /// The app switcher: rest the pointer in a corner and your open windows show up.
    #[serde(default)]
    pub switcher: SwitcherSettings,
}

/// The app switcher (off unless chosen).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherSettings {
    #[serde(default)]
    pub on: bool,
    /// Where the pointer rests to open it: "top-left", "top", "top-right", "left", "right",
    /// "bottom-left" or "bottom-right".
    #[serde(default = "default_spot")]
    pub spot: String,
    /// How long it rests there first, in milliseconds.
    #[serde(default = "default_switch_delay")]
    pub delay_ms: u32,
}

fn default_spot() -> String {
    "top-left".into()
}
fn default_switch_delay() -> u32 {
    150
}

impl Default for SwitcherSettings {
    fn default() -> Self {
        SwitcherSettings { on: false, spot: default_spot(), delay_ms: default_switch_delay() }
    }
}

/// Paw prints and the paw pointer (both off unless chosen).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PawSettings {
    /// Little paw prints where you click, scroll and drag, fading at once.
    #[serde(default)]
    pub on: bool,
    /// The pointer becomes a cat paw (Windows).
    #[serde(default)]
    pub cursor: bool,
    /// The paws' colours, from the cat you picked: fur, outline, toe beans (CSS colours).
    #[serde(default)]
    pub colors: Vec<String>,
}

/// The Cat Dock's settings.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockSettings {
    /// Show the cat ball on the screen edge.
    #[serde(default)]
    pub on: bool,
    /// "right" or "left".
    #[serde(default = "right_edge")]
    pub edge: String,
    /// Where on that edge, from the top (0) to the bottom (1).
    #[serde(default = "dock_y")]
    pub y: f64,
    /// Keep a history of what you copy.
    #[serde(default = "yes")]
    pub clipboard: bool,
    /// Remember that history after Cat Key restarts.
    #[serde(default)]
    pub keep_clips: bool,
    /// Show how hard the computer works (CPU, memory, graphics, heat).
    #[serde(default = "yes")]
    pub vitals: bool,
}

fn right_edge() -> String {
    "right".into()
}
fn dock_y() -> f64 {
    0.62
}

impl Default for DockSettings {
    fn default() -> Self {
        DockSettings { on: false, edge: right_edge(), y: dock_y(), clipboard: true, keep_clips: false, vitals: true }
    }
}

/// Where a coach tip shows up: a spot on the screen, or anywhere the user dragged it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoachPos {
    /// "bottom-right", "bottom-center", "bottom-left", "top-center" (the island),
    /// "top-left", "top-right", "left", "right", "center", or "custom".
    #[serde(default = "default_anchor")]
    pub anchor: String,
    /// For "custom": the tip's centre as a share of the screen's width and height (0–1).
    #[serde(default = "half")]
    pub x: f64,
    #[serde(default = "half")]
    pub y: f64,
}

fn default_anchor() -> String {
    "bottom-right".into()
}
fn half() -> f64 {
    0.5
}

impl Default for CoachPos {
    fn default() -> Self {
        CoachPos { anchor: default_anchor(), x: 0.5, y: 0.5 }
    }
}

impl CoachPos {
    pub const ANCHORS: [&'static str; 10] =
        ["bottom-right", "bottom-center", "bottom-left", "top-center", "top-left", "top-right", "left", "right", "center", "custom"];

    /// Top-left corner for a tip of `w`×`h` inside the work area (`left`, `top`, `width`, `height`),
    /// `margin` away from the edges, never off screen.
    pub fn place(&self, area: (i32, i32, i32, i32), w: i32, h: i32, margin: i32) -> (i32, i32) {
        let (l, t, aw, ah) = area;
        let (r, b) = (l + aw, t + ah);
        let cx = l + (aw - w) / 2;
        let cy = t + (ah - h) / 2;
        let (x, y) = match self.anchor.as_str() {
            "bottom-center" => (cx, b - h - margin),
            "bottom-left" => (l + margin, b - h - margin),
            "top-center" => (cx, t + margin),
            "top-left" => (l + margin, t + margin),
            "top-right" => (r - w - margin, t + margin),
            "left" => (l + margin, cy),
            "right" => (r - w - margin, cy),
            "center" => (cx, cy),
            "custom" => {
                let fx = if self.x.is_finite() { self.x.clamp(0.0, 1.0) } else { 0.5 };
                let fy = if self.y.is_finite() { self.y.clamp(0.0, 1.0) } else { 0.5 };
                (l + (fx * aw as f64) as i32 - w / 2, t + (fy * ah as f64) as i32 - h / 2)
            }
            _ => (r - w - margin, b - h - margin),
        };
        (x.clamp(l, (r - w).max(l)), y.clamp(t, (b - h).max(t)))
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            launch_at_startup: true,
            admin_mode: false,
            theme: default_theme(),
            skin: default_skin(),
            cat_name: default_cat_name(),
            sounds: true,
            pause_hotkey: None,
            excluded_apps: Vec::new(),
            hide_powertoys_tip: false,
            coach: true,
            coach_chromium: false,
            coach_muted: Vec::new(),
            purr_volume: default_purr_volume(),
            cat_look: None,
            swap_alt_ctrl: false,
            coach_pos: CoachPos::default(),
            sound_parts: Default::default(),
            dock: DockSettings::default(),
            paws: PawSettings::default(),
            switcher: SwitcherSettings::default(),
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
    /// The library pack (or template) this came from, e.g. "mac-on-windows".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
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
/// An app in a rule matches the app in front: exactly, or by its start when it ends in
/// `*` (Mac apps whose id carries a version, like "com.adobe.premierepro.25").
pub fn app_matches(pattern: &str, exe: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(start) => !start.is_empty() && exe.starts_with(start),
        None => pattern == exe,
    }
}

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
        source: String::new(),
    }
}

/// The starter pack: the user's Mac-style Alt shortcuts. A Mac already has ⌘, so it
/// starts with none (the Mac starter template is there for people coming from Windows).
pub fn default_rules() -> Vec<Rule> {
    if cfg!(target_os = "macos") {
        return Vec::new();
    }
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
        new_tab(),
    ]
}

fn new_tab() -> Rule {
    rule("LAlt+T", "LCtrl+T", "New tab")
}

pub const STARTER_TEMPLATE_ID: &str = "builtin-starter";

/// Terminals keep Ctrl as it is (Ctrl+C stops a program there).
const MAC_TERMINALS: [&str; 7] = [
    "com.apple.terminal",
    "com.googlecode.iterm2",
    "dev.warp.warp-stable",
    "com.github.wez.wezterm",
    "net.kovidgoyal.kitty",
    "com.mitchellh.ghostty",
    "org.alacritty",
];

/// The Mac's templates: Windows habits (Ctrl) doing what ⌘ does, outside terminals.
fn mac_templates() -> Vec<(&'static str, &'static str, &'static str, Vec<Rule>)> {
    let ctrl = |key: &str, cmd: &str, name: &str| {
        let mut r = rule(&format!("LCtrl+{key}"), cmd, name);
        r.scope = Scope { mode: ScopeMode::Except, apps: MAC_TERMINALS.iter().map(|s| s.to_string()).collect() };
        r
    };
    vec![
        (
            STARTER_TEMPLATE_ID,
            "Windows keys on a Mac",
            "Ctrl does what ⌘ does, like on Windows: copy, paste, undo, save, find, tabs. ⌘ keeps working, and terminals keep their Ctrl.",
            vec![
                ctrl("C", "LWin+C", "Copy"),
                ctrl("V", "LWin+V", "Paste"),
                ctrl("X", "LWin+X", "Cut"),
                ctrl("Z", "LWin+Z", "Undo"),
                ctrl("Y", "LWin+LShift+Z", "Redo"),
                ctrl("A", "LWin+A", "Select all"),
                ctrl("S", "LWin+S", "Save"),
                ctrl("F", "LWin+F", "Find"),
                ctrl("N", "LWin+N", "New"),
                ctrl("T", "LWin+T", "New tab"),
                ctrl("W", "LWin+W", "Close tab"),
                ctrl("R", "LWin+R", "Reload"),
            ],
        ),
        (
            "builtin-mac-more",
            "More Windows habits",
            "Print, open, bold, italic, zoom and the address bar, with Ctrl like on Windows.",
            vec![
                ctrl("P", "LWin+P", "Print"),
                ctrl("O", "LWin+O", "Open"),
                ctrl("B", "LWin+B", "Bold"),
                ctrl("I", "LWin+I", "Italic"),
                ctrl("L", "LWin+L", "Address bar"),
                ctrl("Equals", "LWin+Equals", "Zoom in"),
                ctrl("Minus", "LWin+Minus", "Zoom out"),
                ctrl("0", "LWin+0", "Reset zoom"),
                ctrl("Backspace", "LAlt+Backspace", "Delete word"),
            ],
        ),
    ]
}

/// Templates that ship with Cat Key. Fixed ids, so the UI can tell them apart.
pub fn builtin_templates() -> Vec<Template> {
    fn tpl(id: &str, name: &str, description: &str, rules: Vec<Rule>) -> Template {
        let rules = rules
            .into_iter()
            .enumerate()
            .map(|(i, mut r)| {
                r.id = format!("{id}-{}", i + 1);
                r
            })
            .collect();
        Template { id: id.into(), name: name.into(), description: description.into(), builtin: true, rules }
    }
    if cfg!(target_os = "macos") {
        return mac_templates().into_iter().map(|(id, name, d, rules)| tpl(id, name, d, rules)).collect();
    }
    vec![
        tpl(
            STARTER_TEMPLATE_ID,
            "Your starter set",
            "The shortcuts Cat Key came with: your PowerToys set plus Alt + T for a new tab. Alt works like ⌘ on a Mac.",
            default_rules(),
        ),
        tpl(
            "builtin-mac-more",
            "More Mac-style",
            "More of the ⌘ habits: cut, redo, reopen a closed tab, zoom, bold, switching tabs and more.",
            vec![
                rule("LAlt+X", "LCtrl+X", "Cut"),
                rule("LAlt+LShift+Z", "LCtrl+Y", "Redo"),
                rule("LAlt+LShift+T", "LCtrl+LShift+T", "Reopen closed tab"),
                rule("LAlt+L", "LCtrl+L", "Address bar"),
                rule("LAlt+P", "LCtrl+P", "Print"),
                rule("LAlt+O", "LCtrl+O", "Open"),
                rule("LAlt+K", "LCtrl+K", "Quick search"),
                rule("LAlt+B", "LCtrl+B", "Bold"),
                rule("LAlt+I", "LCtrl+I", "Italic"),
                rule("LAlt+Equals", "LCtrl+Equals", "Zoom in"),
                rule("LAlt+Minus", "LCtrl+Minus", "Zoom out"),
                rule("LAlt+0", "LCtrl+0", "Reset zoom"),
                rule("LAlt+LShift+RBracket", "LCtrl+Tab", "Next tab"),
                rule("LAlt+LShift+LBracket", "LCtrl+LShift+Tab", "Previous tab"),
                rule("LAlt+LBracket", "LAlt+Left", "Back"),
                rule("LAlt+RBracket", "LAlt+Right", "Forward"),
                rule("LAlt+Backspace", "LCtrl+Backspace", "Delete word"),
            ],
        ),
        tpl(
            "builtin-caps-esc",
            "Caps Lock as Escape",
            "Caps Lock becomes Escape: close pop-ups and menus without reaching. Loved by Vim users.",
            vec![rule("CapsLock", "Esc", "Escape")],
        ),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            enabled: true,
            rules: default_rules(),
            settings: Settings::default(),
            templates: Vec::new(),
            packs: Vec::new(),
        }
    }
}

/// Trim names, tidy app lists and clamp text of one rule.
fn tidy_rule(r: &mut Rule) {
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

fn unique_ids(rules: &mut [Rule]) {
    let mut seen = std::collections::HashSet::new();
    for r in rules {
        if r.id.trim().is_empty() || !seen.insert(r.id.clone()) {
            r.id = new_id();
            seen.insert(r.id.clone());
        }
    }
}

impl Config {
    /// Bring a config saved by an older Cat Key up to date. Call before `normalize`.
    /// Returns true when something was added, so the caller can save it once.
    pub fn migrate(&mut self) -> bool {
        let mut changed = false;
        if self.version < 2 {
            // 1.0.2: Alt+T opens a new tab, like the rest of the starter set,
            // unless the user already gave Alt+T a job of their own.
            let t = new_tab();
            if !self.rules.iter().any(|r| r.trigger.overlaps(&t.trigger)) {
                self.rules.push(t);
                changed = true;
            }
        }
        if self.version < 3 {
            // 1.2.0: Cat Key is a cat; its sounds are on unless you turn them off.
            self.settings.sounds = true;
            changed = true;
        }
        changed
    }

    /// Fill ids, tidy app names, clamp odd values. Safe to call repeatedly.
    pub fn normalize(&mut self) {
        self.version = CONFIG_VERSION;
        unique_ids(&mut self.rules);
        for r in &mut self.rules {
            tidy_rule(r);
        }
        // Built-in templates are never stored; user templates get tidy ids and names.
        self.templates.retain(|t| !t.builtin && !t.id.starts_with("builtin-"));
        let mut seen = std::collections::HashSet::new();
        for t in &mut self.templates {
            if t.id.trim().is_empty() || !seen.insert(t.id.clone()) {
                t.id = format!("t{}", &new_id()[1..]);
                seen.insert(t.id.clone());
            }
            t.name = t.name.trim().chars().take(40).collect();
            if t.name.is_empty() {
                t.name = "My template".into();
            }
            t.description = t.description.trim().chars().take(200).collect();
            unique_ids(&mut t.rules);
            for r in &mut t.rules {
                tidy_rule(r);
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
            CScope::Only(apps) => apps.iter().any(|a| app_matches(a, exe)),
            CScope::Except(apps) => !apps.iter().any(|a| app_matches(a, exe)),
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
    if cfg.settings.swap_alt_ctrl {
        // Alt ⇄ Ctrl on both sides, ahead of any other remap of those keys. Rules then
        // see the swapped keys, like a keyboard whose two keys traded places.
        // On a Mac the pair is ⌘ (the Win key here) and Control.
        let pairs: [(u8, u8); 4] = if cfg!(target_os = "macos") {
            [(0x5B, 0xA2), (0xA2, 0x5B), (0x5C, 0xA3), (0xA3, 0x5C)]
        } else {
            [(0xA4, 0xA2), (0xA2, 0xA4), (0xA5, 0xA3), (0xA3, 0xA5)]
        };
        for (from, to) in pairs {
            c.remaps[from as usize].insert(0, CKeyRemap { index: SYSTEM_RULE, scope: CScope::All, action: KAction::Key(to) });
        }
    }
    c
}

/// Index of a remap that comes from a setting, not from a rule (nothing to count).
pub const SYSTEM_RULE: u32 = u32::MAX;

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
            source: String::new(),
        });
        cfg.normalize();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("\"trigger\": \"LAlt+LShift+N\""));
        assert!(json.contains("\"chrome.exe\""));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
        let c = compile(&back);
        assert_eq!(c.rule_count(), 13);
        // The starter Alt+T (new tab) and this per-app Alt+T launcher.
        assert_eq!(c.shortcuts[b'T' as usize].len(), 2);
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
    fn old_config_gets_new_tab_once() {
        let mut cfg: Config = serde_json::from_str(
            r#"{"version":1,"rules":[{"trigger":"LAlt+W","action":{"type":"keys","keys":"LCtrl+W"}}]}"#,
        )
        .unwrap();
        assert!(cfg.migrate());
        cfg.normalize();
        assert_eq!(cfg.version, CONFIG_VERSION);
        let t: Vec<_> = cfg.rules.iter().filter(|r| r.trigger == Shortcut::parse("LAlt+T").unwrap()).collect();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].action, Action::Keys { keys: Shortcut::parse("LCtrl+T").unwrap() });
        // Saved and loaded again: nothing more is added.
        let mut again: Config = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert!(!again.migrate());
        assert_eq!(again.rules.len(), 2);
    }

    #[test]
    fn migration_respects_own_alt_t() {
        let mut cfg: Config = serde_json::from_str(
            r#"{"version":1,"rules":[{"trigger":"Alt+T","action":{"type":"launch","target":"wt.exe"}}]}"#,
        )
        .unwrap();
        cfg.migrate();
        assert_eq!(cfg.rules.len(), 1, "the user's own Alt+T stays, no second Alt+T");
    }

    #[test]
    fn cat_sounds_come_on_once_in_1_2() {
        let mut cfg: Config = serde_json::from_str(r#"{"version":2,"settings":{"sounds":false}}"#).unwrap();
        assert!(cfg.migrate());
        assert!(cfg.settings.sounds && cfg.settings.coach);
        cfg.normalize();
        cfg.settings.sounds = false; // turned off again later: stays off
        assert!(!cfg.migrate());
        assert!(!cfg.settings.sounds);
    }

    #[test]
    fn tips_land_where_asked_and_stay_on_screen() {
        let area = (0, 0, 1920, 1040);
        let p = |anchor: &str| CoachPos { anchor: anchor.into(), x: 0.5, y: 0.5 };
        assert_eq!(p("bottom-right").place(area, 400, 120, 14), (1506, 906));
        assert_eq!(p("top-center").place(area, 400, 120, 14), (760, 14));
        assert_eq!(p("left").place(area, 400, 120, 14), (14, 460));
        assert_eq!(p("center").place(area, 400, 120, 14), (760, 460));
        // A dragged spot near the corner is pulled back inside the screen.
        let c = CoachPos { anchor: "custom".into(), x: 0.99, y: 0.01 };
        assert_eq!(c.place(area, 400, 120, 14), (1520, 0));
        // A second screen to the left of the first.
        assert_eq!(p("bottom-left").place((-1280, 0, 1280, 984), 400, 120, 14), (-1266, 850));
        // Unknown spots fall back to the bottom right.
        assert_eq!(p("somewhere").place(area, 400, 120, 14), (1506, 906));
    }

    #[test]
    fn app_patterns_match_by_start() {
        assert!(app_matches("com.adobe.premierepro*", "com.adobe.premierepro.25"));
        assert!(app_matches("chrome.exe", "chrome.exe"));
        assert!(!app_matches("chrome.exe", "chrome2.exe"));
        assert!(!app_matches("*", "anything"));
        assert_eq!(normalize_app("com.adobe.PremierePro*"), "com.adobe.premierepro*");
    }

    #[test]
    fn mac_templates_are_sound() {
        let all = mac_templates();
        assert_eq!(all[0].0, STARTER_TEMPLATE_ID);
        for (_, _, _, rules) in &all {
            assert!(!rules.is_empty());
            for r in rules {
                assert_eq!(r.scope.mode, ScopeMode::Except);
                assert!(r.scope.apps.iter().any(|a| a == "com.apple.terminal"));
            }
        }
    }

    #[test]
    fn builtin_templates_are_sound() {
        let all = builtin_templates();
        assert_eq!(all[0].id, STARTER_TEMPLATE_ID);
        assert_eq!(all[0].rules.len(), default_rules().len());
        let mut ids = std::collections::HashSet::new();
        for t in &all {
            assert!(t.builtin && !t.name.is_empty() && !t.rules.is_empty());
            assert!(ids.insert(t.id.clone()));
            for r in &t.rules {
                assert!(ids.insert(r.id.clone()), "duplicate id {}", r.id);
                assert_eq!(rule_problem(r), None, "{}", r.name);
            }
            // No template fights with itself.
            for (i, a) in t.rules.iter().enumerate() {
                for b in &t.rules[i + 1..] {
                    assert!(!a.trigger.overlaps(&b.trigger), "{} vs {}", a.name, b.name);
                }
            }
        }
        // The extras never clash with the starter set.
        for a in &all[0].rules {
            for b in &all[1].rules {
                assert!(!a.trigger.overlaps(&b.trigger), "{} vs {}", a.name, b.name);
            }
        }
    }

    #[test]
    fn user_templates_are_tidied() {
        let mut cfg = Config::default();
        cfg.templates.push(Template { id: String::new(), name: "  ".into(), description: String::new(), builtin: false, rules: default_rules() });
        cfg.templates.push(Template { id: "builtin-starter".into(), name: "x".into(), description: String::new(), builtin: true, rules: vec![] });
        cfg.normalize();
        assert_eq!(cfg.templates.len(), 1);
        assert_eq!(cfg.templates[0].name, "My template");
        assert!(cfg.templates[0].id.starts_with('t'));
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
        // Templates never become live shortcuts.
        assert_eq!(compile(&cfg).rule_count(), default_rules().len());
    }

    #[test]
    fn overlap_rules() {
        let p = |s: &str| Shortcut::parse(s).unwrap();
        assert!(p("Alt+T").overlaps(&p("LAlt+T")));
        assert!(!p("LAlt+T").overlaps(&p("RAlt+T")));
        assert!(!p("LAlt+T").overlaps(&p("LAlt+LShift+T")));
        assert!(!p("LAlt+T").overlaps(&p("LCtrl+T")));
        assert!(p("CapsLock").overlaps(&p("CapsLock")));
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
            source: String::new(),
        });
        assert_eq!(compile(&cfg).rule_count(), 0);
    }
}
