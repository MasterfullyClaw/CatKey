//! Import remaps from PowerToys Keyboard Manager's `default.json`.
//!
//! PowerToys stores keys as virtual-key codes joined by ';' ("164;87" = Left Alt + W).
//! The file layout has changed across versions, so instead of trusting one schema
//! we walk the whole JSON tree and pick up every object with an `originalKeys` field.

use crate::config::{normalize_app, new_id, Action, Rule, Scope, ScopeMode};
use crate::keys::{is_modifier, mod_bit, sided, Family, ModSpec, Shortcut, Side};
use serde_json::Value;

/// PowerToys' code for "Win, either side".
const PT_WIN_EITHER: u32 = 0x104;
/// PowerToys' code for "disabled" (the key does nothing).
const PT_DISABLED: u32 = 0x100;

#[derive(Debug, PartialEq)]
enum Keys {
    Shortcut(Shortcut),
    Disabled,
}

fn parse_codes(s: &str) -> Option<Keys> {
    let codes: Vec<u32> = s.split(';').filter_map(|p| p.trim().parse::<u32>().ok()).collect();
    if codes.is_empty() {
        return None;
    }
    if codes.iter().any(|c| *c == PT_DISABLED) {
        return Some(Keys::Disabled);
    }
    let mut mods = ModSpec::NONE;
    let mut key: Option<u8> = None;
    let mut mod_count = 0;
    let mut lone_mod = 0u8;
    for c in &codes {
        let c = *c;
        if c == PT_WIN_EITHER {
            mods = mods.with(Family::Win, Side::Any);
            mod_count += 1;
            lone_mod = keys_lwin();
            continue;
        }
        if c == 0 || c > 0xFF {
            continue;
        }
        let vk = c as u8;
        if is_modifier(vk) {
            let fam = match mod_bit(vk) {
                b if b & 0b11 != 0 => Family::Ctrl,
                b if b & 0b1100 != 0 => Family::Shift,
                b if b & 0b110000 != 0 => Family::Alt,
                _ => Family::Win,
            };
            let side = match vk {
                0x10 | 0x11 | 0x12 => Side::Any,
                0xA0 | 0xA2 | 0xA4 | 0x5B => Side::Left,
                _ => Side::Right,
            };
            mods = mods.with(fam, side);
            mod_count += 1;
            lone_mod = sided(vk);
        } else {
            key = Some(vk);
        }
    }
    match key {
        Some(k) => Some(Keys::Shortcut(Shortcut::new(mods, k))),
        // A single modifier on its own is a key (e.g. remap Right Alt).
        None if mod_count == 1 => Some(Keys::Shortcut(Shortcut::new(ModSpec::NONE, lone_mod))),
        None => None,
    }
}

fn keys_lwin() -> u8 {
    crate::keys::VK_LWIN
}

fn get_str<'a>(o: &'a serde_json::Map<String, Value>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|n| o.get(*n).and_then(|v| v.as_str())).map(|s| s.trim()).filter(|s| !s.is_empty())
}

fn entry_to_rule(o: &serde_json::Map<String, Value>) -> Option<Rule> {
    let trigger = match parse_codes(get_str(o, &["originalKeys"])?)? {
        Keys::Shortcut(s) => s,
        Keys::Disabled => return None,
    };
    let op = o.get("operationType").and_then(|v| v.as_u64()).unwrap_or(0);
    let text = get_str(o, &["unicodeText", "newRemapString", "newText"]);
    let program = get_str(o, &["runProgramFilePath", "programPath"]);
    let uri = get_str(o, &["openUri", "uriToOpen"]);
    let action = if let Some(t) = text {
        Action::Text { text: t.to_string() }
    } else if let (Some(p), true) = (program, op == 1 || uri.is_none()) {
        Action::Launch { target: p.to_string(), args: get_str(o, &["runProgramArgs", "programArgs"]).unwrap_or("").to_string() }
    } else if let Some(u) = uri {
        Action::Launch { target: u.to_string(), args: String::new() }
    } else {
        match parse_codes(get_str(o, &["newRemapKeys", "newRemapShortcut", "targetKeys"])?)? {
            Keys::Shortcut(s) => Action::Keys { keys: s },
            Keys::Disabled => Action::Block,
        }
    };
    let enabled = ["isActive", "isEnabled", "enabled", "active"]
        .iter()
        .find_map(|n| o.get(*n).and_then(|v| v.as_bool()))
        .unwrap_or(true);
    let scope = match get_str(o, &["targetApp", "targetApplication", "app"]) {
        Some(app) => Scope { mode: ScopeMode::Only, apps: vec![normalize_app(app)] },
        None => Scope::all(),
    };
    Some(Rule { id: new_id(), enabled, name: String::new(), trigger, action, scope })
}

fn walk(v: &Value, out: &mut Vec<Rule>) {
    match v {
        Value::Object(o) => {
            if o.contains_key("originalKeys") {
                if let Some(r) = entry_to_rule(o) {
                    out.push(r);
                }
                return;
            }
            for child in o.values() {
                walk(child, out);
            }
        }
        Value::Array(a) => {
            for child in a {
                walk(child, out);
            }
        }
        _ => {}
    }
}

/// Parse a PowerToys Keyboard Manager config into Cat Key rules.
pub fn import(json: &str) -> Result<Vec<Rule>, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("That file isn't valid JSON: {e}"))?;
    let mut rules = Vec::new();
    walk(&v, &mut rules);
    // Drop exact duplicates (same trigger, action and scope).
    let mut unique: Vec<Rule> = Vec::new();
    for r in rules {
        if !unique.iter().any(|u| u.trigger == r.trigger && u.action == r.action && u.scope == r.scope) {
            unique.push(r);
        }
    }
    Ok(unique)
}

/// Rules from `incoming` that the existing list doesn't already have (same trigger + scope).
pub fn merge_new(existing: &[Rule], incoming: Vec<Rule>) -> Vec<Rule> {
    incoming
        .into_iter()
        .filter(|r| !existing.iter().any(|e| e.trigger == r.trigger && e.scope == r.scope))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "remapKeys": {"inProcess": [{"originalKeys": "20", "newRemapKeys": "27"}]},
      "remapKeysToText": {"inProcess": []},
      "remapShortcuts": {
        "global": [
          {"originalKeys": "164;87", "exactMatch": false, "newRemapKeys": "162;87"},
          {"originalKeys": "164;78", "exactMatch": false, "newRemapKeys": "17;78"},
          {"originalKeys": "164;81", "exactMatch": false, "newRemapKeys": "164;115"},
          {"originalKeys": "164;160;78", "exactMatch": false, "newRemapKeys": "162;160;78"},
          {"originalKeys": "260;84", "operationType": 1, "runProgramFilePath": "C:\\Windows\\System32\\wt.exe", "runProgramArgs": ""},
          {"originalKeys": "164;77", "unicodeText": "hello"},
          {"originalKeys": "164;87", "exactMatch": false, "newRemapKeys": "162;87"}
        ],
        "appSpecific": [
          {"originalKeys": "164;76", "exactMatch": false, "newRemapKeys": "162;76", "targetApp": "chrome"},
          {"originalKeys": "164;73", "newRemapKeys": "256", "targetApp": "msedge.exe", "isActive": false}
        ]
      }
    }"#;

    #[test]
    fn imports_every_shape() {
        let rules = import(SAMPLE).unwrap();
        let show: Vec<String> = rules
            .iter()
            .map(|r| {
                let a = match &r.action {
                    Action::Keys { keys } => keys.to_string(),
                    Action::Launch { target, .. } => format!("open {target}"),
                    Action::Text { text } => format!("type {text}"),
                    Action::Block => "block".into(),
                };
                let scope = if r.scope.apps.is_empty() { String::new() } else { format!(" @{}", r.scope.apps.join(",")) };
                format!("{} -> {}{}{}", r.trigger, a, scope, if r.enabled { "" } else { " (off)" })
            })
            .collect();
        assert_eq!(
            show,
            vec![
                "CapsLock -> Esc",
                "LAlt+W -> LCtrl+W",
                "LAlt+N -> Ctrl+N",
                "LAlt+Q -> LAlt+F4",
                "LAlt+LShift+N -> LCtrl+LShift+N",
                "Win+T -> open C:\\Windows\\System32\\wt.exe",
                "LAlt+M -> type hello",
                "LAlt+L -> LCtrl+L @chrome.exe",
                "LAlt+I -> block @msedge.exe (off)",
            ]
        );
    }

    #[test]
    fn merge_skips_existing() {
        let existing = crate::config::default_rules();
        let incoming = import(SAMPLE).unwrap();
        let fresh = merge_new(&existing, incoming);
        assert!(fresh.iter().all(|r| r.trigger.to_string() != "LAlt+W"));
        assert!(fresh.iter().any(|r| r.trigger.to_string() == "CapsLock"));
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(import("not json").is_err());
        assert_eq!(import("{}").unwrap().len(), 0);
    }
}
