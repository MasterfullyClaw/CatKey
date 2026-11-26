//! Cat Key as an MCP server, for Claude Desktop and other AI apps: `CatKey --mcp`.
//!
//! The AI can hand Cat Key a shortcut pack it made (no copying and pasting), read the
//! pack format first, and see your shortcuts so it doesn't clash with them. It speaks MCP
//! over its own stdin and stdout and never touches the keyboard: a pack goes into Cat
//! Key's inbox, and the app opens it so you can tick what you want (or close it).
//!
//! `connect_claude` adds this server to Claude Desktop's settings when you ask it to.

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const GUIDE: &str = include_str!("../../library/README.md");

/// Run the server until the AI app closes the pipe.
pub fn serve() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim_start_matches('\u{feff}').trim();
        if line.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                send(&mut out, &error(Value::Null, -32700, "That wasn't JSON."));
                continue;
            }
        };
        if let Some(batch) = msg.as_array() {
            let replies: Vec<Value> = batch.iter().filter_map(handle).collect();
            if !replies.is_empty() {
                send(&mut out, &Value::Array(replies));
            }
        } else if let Some(reply) = handle(&msg) {
            send(&mut out, &reply);
        }
    }
}

fn send(out: &mut impl Write, v: &Value) {
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// One message in, at most one reply out (notifications get none).
pub fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = match method {
        "initialize" => Ok(initialize(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => Ok(call(&params)),
        _ => Err((-32601, format!("Cat Key doesn't know {method}"))),
    };
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, m)) => error(id, code, &m),
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(|v| v.as_str()).unwrap_or(VERSIONS[0]);
    let version = VERSIONS.iter().find(|v| **v == asked).copied().unwrap_or(VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "catkey", "title": "Cat Key", "version": env!("CARGO_PKG_VERSION") },
        "instructions": "Cat Key is the user's keyboard-shortcut manager (Windows and Mac). To give them shortcuts for an app: call get_pack_format, look up the app's real default shortcuts in its official documentation, write a catkey-pack, and send it with add_shortcut_pack. Cat Key opens it on their screen so they choose what to add. Call list_shortcuts first if you're suggesting new keys, so they don't clash with the user's own."
    })
}

fn tools() -> Value {
    json!([
        {
            "name": "get_pack_format",
            "title": "How a Cat Key pack is written",
            "description": "The catkey-pack format: kinds (cheat sheet or remap), key names, Mod/Opt modifiers for packs that work on Windows and Mac, and the optional fields. Read it before writing a pack.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "add_shortcut_pack",
            "title": "Send a pack to Cat Key",
            "description": "Hand Cat Key one shortcut pack (`pack`) or several (`packs`, one per app). Cat Key opens on the user's screen with it, so they can tick what they want and add it; nothing changes until they do. Use the app's real default shortcuts, with a source link when you have one.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pack": { "type": "object", "description": "A catkey-pack (see get_pack_format)." },
                    "packs": { "type": "array", "items": { "type": "object" }, "description": "Several catkey-packs, one per app." }
                },
                "additionalProperties": false
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": "list_shortcuts",
            "title": "The user's own shortcuts",
            "description": "The shortcuts the user has set up in Cat Key: what they press, what it does, and in which apps. Read-only. Use it so new suggestions don't clash.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
            "annotations": { "readOnlyHint": true }
        }
    ])
}

fn text(t: impl Into<String>, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": t.into() }], "isError": is_error })
}

fn call(params: &Value) -> Value {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    match name {
        "get_pack_format" => text(
            format!(
                "{GUIDE}\n\nMore:\n* One pack per app. Check every shortcut against the app's own documentation; never invent one.\n* `platform`: \"windows\", \"mac\" or \"both\". For \"both\", use Mod/Opt/Shift/Ctrl and add `mac` where the Mac key is different.\n* For an app Cat Key doesn't know, `apps` names its program: Windows exe in lower case (\"reaper.exe\"), Mac bundle id (\"com.cockos.reaper\")."
            ),
            false,
        ),
        "add_shortcut_pack" => add_pack(&args),
        "list_shortcuts" => list_shortcuts(),
        _ => text(format!("Cat Key has no tool called {name}."), true),
    }
}

fn looks_like_pack(v: &Value) -> bool {
    v.is_object() && v.get("sections").and_then(|s| s.as_array()).is_some_and(|s| !s.is_empty())
}

fn add_pack(args: &Value) -> Value {
    let body = if let Some(list) = args.get("packs").and_then(|p| p.as_array()) {
        if list.is_empty() || !list.iter().all(looks_like_pack) {
            return text("Each pack needs a name and at least one section with items (see get_pack_format).", true);
        }
        json!({ "format": "catkey-packs", "version": 1, "packs": list })
    } else if let Some(p) = args.get("pack").filter(|p| looks_like_pack(p)) {
        p.clone()
    } else {
        return text("Send `pack` (one catkey-pack) or `packs` (a list). A pack needs at least one section with items (see get_pack_format).", true);
    };
    let n = body.get("packs").and_then(|p| p.as_array()).map(|p| p.len()).unwrap_or(1);
    let id = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let file = inbox_dir().join(format!("{id}.json"));
    if let Err(e) = crate::app::write_atomic(&file, &body.to_string()) {
        return text(format!("Couldn't hand it to Cat Key: {e}"), true);
    }
    wake_app();
    text(
        if n == 1 {
            "Sent. Cat Key is opening it on the user's screen: they tick the shortcuts they want and press Add."
        } else {
            "Sent. Cat Key is adding them to the user's Library (under Yours), where they can open each one and add what they want."
        },
        false,
    )
}

fn list_shortcuts() -> Value {
    let cfg: Value = std::fs::read_to_string(crate::app::config_dir().join("config.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    let rules = cfg.get("rules").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    if rules.is_empty() {
        return text("The user has no shortcuts of their own in Cat Key yet.", false);
    }
    let mut lines = Vec::new();
    for r in rules {
        let s = |v: Option<&Value>| v.and_then(|x| x.as_str()).unwrap_or("").to_string();
        let a = r.get("action").cloned().unwrap_or_default();
        let does = match a.get("type").and_then(|t| t.as_str()).unwrap_or("") {
            "keys" => format!("presses {}", s(a.get("keys"))),
            "launch" => format!("opens {}", s(a.get("target"))),
            "text" => "types a piece of text".to_string(),
            "block" => "does nothing (blocked)".to_string(),
            other => other.to_string(),
        };
        let scope = r.get("scope").cloned().unwrap_or_default();
        let apps: Vec<String> = scope.get("apps").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let where_ = match scope.get("mode").and_then(|m| m.as_str()).unwrap_or("all") {
            "only" if !apps.is_empty() => format!(" (only in {})", apps.join(", ")),
            "except" if !apps.is_empty() => format!(" (not in {})", apps.join(", ")),
            _ => String::new(),
        };
        let on = if r.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true) { "" } else { " [off]" };
        let name = s(r.get("name"));
        let name = if name.is_empty() { String::new() } else { format!("{name}: ") };
        lines.push(format!("- {name}{} → {does}{where_}{on}", s(r.get("trigger"))));
    }
    text(format!("Key names: Win is the Windows key (⌘ on a Mac), Alt is ⌥ Option on a Mac, L/R mean the left or right key.\n{}", lines.join("\n")), false)
}

pub fn inbox_dir() -> PathBuf {
    crate::app::config_dir().join("inbox")
}

/// Bring Cat Key up so it picks up the inbox (if it's already running, this only wakes it).
fn wake_app() {
    let Ok(exe) = std::env::current_exe() else { return };
    #[cfg(target_os = "macos")]
    {
        if let Some(app) = exe.ancestors().find(|p| p.extension().is_some_and(|x| x == "app")) {
            let _ = Command::new("/usr/bin/open").arg("-a").arg(app).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
            return;
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        let _ = Command::new(&exe).creation_flags(DETACHED_PROCESS).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        return;
    }
    #[allow(unreachable_code)]
    {
        let _ = Command::new(&exe).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    }
}

/// Packs waiting in the inbox (oldest first), taken out of it.
pub fn take_inbox() -> Vec<String> {
    let dir = inbox_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let mut out = Vec::new();
    for f in files.into_iter().take(20) {
        if let Ok(t) = std::fs::read_to_string(&f) {
            if t.len() < 2_000_000 {
                out.push(t);
            }
        }
        let _ = std::fs::remove_file(&f);
    }
    out
}

/// Whether anything is waiting (cheap: one folder listing).
pub fn inbox_waiting() -> bool {
    std::fs::read_dir(inbox_dir()).map(|rd| rd.flatten().any(|e| e.path().extension().is_some_and(|x| x == "json"))).unwrap_or(false)
}

// ------------------------------------------------------------------ connecting Claude Desktop

fn claude_config() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/Claude/claude_desktop_config.json"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Claude").join("claude_desktop_config.json"))
    }
}

/// What Claude Desktop (or any MCP app) needs to start this server.
pub fn server_entry() -> Value {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    json!({ "command": exe, "args": ["--mcp"] })
}

/// Whether Claude Desktop is installed, and whether it already starts this Cat Key.
pub fn claude_status() -> (bool, bool) {
    let Some(path) = claude_config() else { return (false, false) };
    let installed = path.parent().is_some_and(Path::exists);
    let cfg: Value = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| json!({}));
    let ours = server_entry();
    let connected = cfg.pointer("/mcpServers/catkey/command") == ours.get("command");
    (installed, connected)
}

/// Add (or update) Cat Key in Claude Desktop's settings, keeping everything else there.
pub fn connect_claude() -> Result<(), String> {
    let path = claude_config().ok_or("Couldn't find where Claude Desktop keeps its settings.")?;
    let dir = path.parent().ok_or("Couldn't find Claude Desktop's settings folder.")?;
    if !dir.exists() {
        return Err("Claude Desktop isn't installed on this computer (it has no settings folder yet). Install it and open it once, then try again.".into());
    }
    let old = std::fs::read_to_string(&path).ok();
    let mut cfg: Value = match &old {
        Some(t) if !t.trim().is_empty() => serde_json::from_str(t).map_err(|_| "Claude Desktop's settings file isn't valid JSON, so Cat Key left it alone. Fix it (or add Cat Key by hand with “Copy the setup”).".to_string())?,
        _ => json!({}),
    };
    if !cfg.is_object() {
        return Err("Claude Desktop's settings file looks unusual, so Cat Key left it alone.".into());
    }
    if let Some(t) = &old {
        let backup = path.with_extension("json.catkey-backup");
        if !backup.exists() {
            let _ = std::fs::write(&backup, t);
        }
    }
    let servers = cfg.as_object_mut().unwrap().entry("mcpServers").or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers.as_object_mut().unwrap().insert("catkey".into(), server_entry());
    let text = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    crate::app::write_atomic(&path, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_answers_the_handshake_and_lists_its_tools() {
        let r = handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"t","version":"1"}}})).unwrap();
        assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(r["result"]["serverInfo"]["name"], "catkey");
        let r = handle(&json!({"jsonrpc":"2.0","id":"x","method":"initialize","params":{"protocolVersion":"1999-01-01"}})).unwrap();
        assert_eq!(r["result"]["protocolVersion"], VERSIONS[0]);
        assert!(handle(&json!({"jsonrpc":"2.0","method":"notifications/initialized"})).is_none());
        let r = handle(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"})).unwrap();
        let names: Vec<&str> = r["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["get_pack_format", "add_shortcut_pack", "list_shortcuts"]);
        let r = handle(&json!({"jsonrpc":"2.0","id":3,"method":"nope"})).unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }

    #[test]
    fn a_pack_without_shortcuts_is_turned_back() {
        let r = call(&json!({"name":"add_shortcut_pack","arguments":{"pack":{"name":"x","sections":[]}}}));
        assert_eq!(r["isError"], true);
        let r = call(&json!({"name":"get_pack_format","arguments":{}}));
        assert!(r["content"][0]["text"].as_str().unwrap().contains("catkey-pack"));
    }
}
