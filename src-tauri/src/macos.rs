//! The Mac side of what `win.rs` does on Windows: the front app, running apps, opening
//! things, the login item, the clipboard and the date. Same function names as `win.rs`,
//! so the rest of the app doesn't care which system it runs on (this module is `win` on a Mac).

use objc2::rc::{autoreleasepool, Retained};
use objc2_app_kit::{NSApplicationActivationPolicy, NSPasteboard, NSPasteboardTypeString, NSWorkspace};
use objc2_foundation::{NSBundle, NSString, NSURL};
use serde::Serialize;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// The app's bundle id in lower case ("com.apple.safari"): what rules match on a Mac.
    pub exe: String,
    pub name: String,
    pub path: String,
    pub title: String,
}

fn ns(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

// ---------------------------------------------------------------- apps

/// Bundle id (lower case) and process id of the app in front.
pub fn front_app() -> Option<(String, i32)> {
    autoreleasepool(|_| {
        let ws = NSWorkspace::sharedWorkspace();
        let app = ws.frontmostApplication()?;
        let id = app.bundleIdentifier().map(|s| s.to_string().to_lowercase()).unwrap_or_default();
        Some((id, app.processIdentifier()))
    })
}

/// Apps with a Dock icon right now (for the app picker).
pub fn running_apps() -> Vec<AppInfo> {
    autoreleasepool(|_| {
        let ws = NSWorkspace::sharedWorkspace();
        let mut out: Vec<AppInfo> = Vec::new();
        let own = std::process::id() as i32;
        for app in ws.runningApplications().iter() {
            if app.activationPolicy() != NSApplicationActivationPolicy::Regular || app.processIdentifier() == own {
                continue;
            }
            let Some(id) = app.bundleIdentifier().map(|s| s.to_string()) else { continue };
            let name = app.localizedName().map(|s| s.to_string()).unwrap_or_else(|| id.clone());
            let path = app.bundleURL().and_then(|u| u.path()).map(|p| p.to_string()).unwrap_or_default();
            let exe = id.to_lowercase();
            if !out.iter().any(|a| a.exe == exe) {
                out.push(AppInfo { exe, name, path, title: String::new() });
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        out
    })
}

fn bundle_at(path: &str) -> Option<Retained<NSBundle>> {
    if !path.ends_with(".app") && !path.ends_with(".app/") {
        return None;
    }
    NSBundle::bundleWithPath(&ns(path))
}

/// Apps in Applications (yours, everyone's and the Mac's own): name, bundle id and where they live.
pub fn installed_apps() -> Vec<AppInfo> {
    let home = std::env::var("HOME").unwrap_or_default();
    let roots = [
        "/Applications".to_string(),
        "/Applications/Utilities".to_string(),
        format!("{home}/Applications"),
        "/System/Applications".to_string(),
        "/System/Applications/Utilities".to_string(),
    ];
    let mut apps: Vec<std::path::PathBuf> = Vec::new();
    for root in &roots {
        let Ok(rd) = std::fs::read_dir(root) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".app") {
                apps.push(p);
            } else if p.is_dir() && !name.starts_with('.') {
                // Apps kept in a folder of their own (Adobe, Microsoft…).
                if let Ok(sub) = std::fs::read_dir(&p) {
                    apps.extend(sub.flatten().map(|x| x.path()).filter(|x| x.extension().is_some_and(|e| e == "app")));
                }
            }
        }
    }
    let mut out: Vec<AppInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    autoreleasepool(|_| {
        for p in apps {
            let path = p.to_string_lossy().to_string();
            let name = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            if name.to_lowercase().contains("uninstall") {
                continue;
            }
            let Some(id) = bundle_at(&path).and_then(|b| b.bundleIdentifier()) else { continue };
            let exe = id.to_string().to_lowercase();
            if seen.insert(exe.clone()) {
                out.push(AppInfo { exe, name, path, title: String::new() });
            }
        }
    });
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/// For a picked file: an app's bundle id (lower case), otherwise the file name.
pub fn file_name_lower(path: &str) -> String {
    autoreleasepool(|_| {
        if let Some(id) = bundle_at(path).and_then(|b| b.bundleIdentifier()) {
            return id.to_string().to_lowercase();
        }
        path.trim_end_matches('/').rsplit('/').next().unwrap_or(path).to_lowercase()
    })
}

/// An app's display name ("Final Cut Pro").
pub fn file_description(path: &str) -> Option<String> {
    autoreleasepool(|_| {
        let b = bundle_at(path)?;
        for key in ["CFBundleDisplayName", "CFBundleName"] {
            if let Some(v) = b.objectForInfoDictionaryKey(&ns(key)) {
                if let Ok(s) = v.downcast::<NSString>() {
                    let s = s.to_string();
                    if !s.trim().is_empty() {
                        return Some(s);
                    }
                }
            }
        }
        Path::new(path.trim_end_matches('/')).file_stem().map(|s| s.to_string_lossy().to_string())
    })
}

/// App icons come later on the Mac; the window draws a letter instead.
pub fn icon_data_url(_path: &str) -> Option<String> {
    None
}

// ---------------------------------------------------------------- opening things

fn open_cmd(args: &[&str]) -> Result<(), String> {
    let ok = Command::new("/usr/bin/open")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok {
        Ok(())
    } else {
        Err("macOS couldn't open it".into())
    }
}

fn split_args(args: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in args.chars() {
        match c {
            '"' => quoted = !quoted,
            ' ' | '\t' if !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn looks_like_bundle_id(s: &str) -> bool {
    !s.contains('/') && !s.contains(' ') && s.split('.').count() >= 3 && !Path::new(s).exists()
}

/// Open a program, file, folder or link.
pub fn launch(target: &str, args: &str) -> Result<(), String> {
    let target = target.trim().trim_matches('"');
    if target.is_empty() {
        return Err("nothing to open".into());
    }
    let target = if let Some(rest) = target.strip_prefix("~/") {
        std::env::var("HOME").map(|h| format!("{h}/{rest}")).unwrap_or_else(|_| target.to_string())
    } else {
        target.to_string()
    };
    let extra = split_args(args);
    let extra: Vec<&str> = extra.iter().map(String::as_str).collect();
    if target.contains("://") || target.starts_with("mailto:") || target.starts_with("x-apple.") {
        return open_cmd(&[&target]);
    }
    if target.trim_end_matches('/').ends_with(".app") {
        let mut a = vec!["-a", target.as_str()];
        if !extra.is_empty() {
            a.push("--args");
            a.extend(extra);
        }
        return open_cmd(&a);
    }
    if looks_like_bundle_id(&target) {
        let mut a = vec!["-b", target.as_str()];
        if !extra.is_empty() {
            a.push("--args");
            a.extend(extra);
        }
        return open_cmd(&a);
    }
    let p = Path::new(&target);
    if p.is_file() && is_executable(p) && !target.ends_with(".command") {
        return Command::new(p)
            .args(&extra)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    open_cmd(&[&target])
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// Open a link of any length (macOS passes it whole).
pub fn open_url(url: &str) -> Result<(), String> {
    open_cmd(&[url])
}

/// True when some app opens this kind of link (claude://, …).
pub fn has_protocol(scheme: &str) -> bool {
    autoreleasepool(|_| {
        let Some(url) = NSURL::URLWithString(&ns(&format!("{scheme}://check"))) else { return false };
        NSWorkspace::sharedWorkspace().URLForApplicationToOpenURL(&url).is_some()
    })
}

/// On a Mac, links go to `open` whole; nothing to look up.
pub fn protocol_command(_scheme: &str, _url: &str) -> Option<(String, String)> {
    None
}

pub fn run_as_user(exe: &str, args: &str) -> Result<(), String> {
    launch(exe, args)
}

/// Known bundle ids for apps Cat Builder can open.
fn known_bundle(name: &str) -> &'static [&'static str] {
    match name.to_lowercase().as_str() {
        "claude" => &["com.anthropic.claudefordesktop"],
        "chatgpt" => &["com.openai.chat"],
        _ => &[],
    }
}

/// An installed app by name ("ChatGPT", "Claude"): the path of its .app.
pub fn start_app_id(names: &[&str]) -> Option<String> {
    autoreleasepool(|_| {
        let ws = NSWorkspace::sharedWorkspace();
        for name in names {
            for id in known_bundle(name) {
                if let Some(p) = ws.URLForApplicationWithBundleIdentifier(&ns(id)).and_then(|u| u.path()) {
                    return Some(p.to_string());
                }
            }
        }
        let home = std::env::var("HOME").unwrap_or_default();
        for name in names {
            for dir in ["/Applications".to_string(), format!("{home}/Applications"), "/System/Applications".to_string()] {
                let p = format!("{dir}/{name}.app");
                if Path::new(&p).exists() {
                    return Some(p);
                }
            }
        }
        None
    })
}

pub fn launch_app_id(id: &str) -> Result<(), String> {
    if id.ends_with(".app") || id.starts_with('/') {
        open_cmd(&["-a", id])
    } else {
        open_cmd(&["-b", id])
    }
}

// ---------------------------------------------------------------- open at login (a LaunchAgent)

const AGENT_LABEL: &str = "com.masterfully.catkey";

fn agent_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Library/LaunchAgents").join(format!("{AGENT_LABEL}.plist"))
}

pub fn autostart_command() -> String {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    format!("\"{}\" --autostart", exe)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The command our login item runs, written the same way as `autostart_command`.
pub fn get_run_entry() -> Option<String> {
    let text = std::fs::read_to_string(agent_path()).ok()?;
    let args: Vec<String> = text
        .split("<string>")
        .skip(1)
        .filter_map(|part| part.split("</string>").next())
        .map(|s| s.replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&"))
        .collect();
    // [label, exe, --autostart]
    let exe = args.get(1)?;
    let rest: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    Some(format!("\"{}\"{}{}", exe, if rest.is_empty() { "" } else { " " }, rest.join(" ")))
}

pub fn set_run_entry(command: Option<&str>) -> bool {
    let path = agent_path();
    match command {
        None => !path.exists() || std::fs::remove_file(&path).is_ok(),
        Some(c) => {
            let exe = c.trim_start_matches('"').split('"').next().unwrap_or_default().to_string();
            let plist = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>{}</string>\n  <key>ProgramArguments</key>\n  <array>\n    <string>{}</string>\n    <string>--autostart</string>\n  </array>\n  <key>RunAtLoad</key>\n  <true/>\n  <key>ProcessType</key>\n  <string>Interactive</string>\n</dict>\n</plist>\n",
                AGENT_LABEL,
                xml_escape(&exe)
            );
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            std::fs::write(&path, plist).is_ok()
        }
    }
}

/// macOS has no separate switch for our login item that we can read.
pub fn startup_approved() -> Option<bool> {
    None
}

// ---------------------------------------------------------------- the clipboard

pub fn clipboard_seq() -> u32 {
    autoreleasepool(|_| NSPasteboard::generalPasteboard().changeCount() as u32)
}

/// What's on the clipboard for the Cat Dock's history: text, or copied files.
pub struct ClipData {
    pub text: String,
    pub files: Vec<String>,
}

/// The clipboard for the history: `None` when it's empty, or when a password manager marked
/// it as private (the nspasteboard.org markers).
pub fn clipboard_read() -> Option<ClipData> {
    autoreleasepool(|_| {
        let pb = NSPasteboard::generalPasteboard();
        let types = pb.types()?;
        let mut has_files = false;
        for t in types.iter() {
            let t = t.to_string();
            if matches!(t.as_str(), "org.nspasteboard.ConcealedType" | "org.nspasteboard.TransientType" | "org.nspasteboard.AutoGeneratedType") {
                return None;
            }
            has_files |= t == "public.file-url";
        }
        if has_files {
            let mut files = Vec::new();
            for it in pb.pasteboardItems().map(|a| a.to_vec()).unwrap_or_default() {
                let Some(u) = it.stringForType(&ns("public.file-url")) else { continue };
                let Some(url) = NSURL::URLWithString(&u) else { continue };
                let url = url.filePathURL().unwrap_or(url);
                if let Some(p) = url.path() {
                    files.push(p.to_string());
                }
            }
            if !files.is_empty() {
                return Some(ClipData { text: String::new(), files });
            }
        }
        let text = unsafe { pb.stringForType(NSPasteboardTypeString) }?.to_string();
        (!text.is_empty()).then_some(ClipData { text, files: Vec::new() })
    })
}

pub fn clipboard_write_text(text: &str) -> bool {
    autoreleasepool(|_| {
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        unsafe { pb.setString_forType(&ns(text), NSPasteboardTypeString) }
    })
}

/// Copied files, the way Finder copies them (a Mac has no "cut" for files).
pub fn clipboard_write_files(paths: &[String], _cut: bool) -> bool {
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::{NSPasteboardItem, NSPasteboardWriting};
    use objc2_foundation::NSArray;
    autoreleasepool(|_| {
        let items: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = paths
            .iter()
            .filter_map(|p| {
                let url = NSURL::fileURLWithPath(&ns(p)).absoluteString()?;
                let item = NSPasteboardItem::new();
                item.setString_forType(&url, &ns("public.file-url"));
                Some(ProtocolObject::from_retained(item))
            })
            .collect();
        if items.is_empty() {
            return false;
        }
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        pb.writeObjects(&NSArray::from_retained_slice(&items))
    })
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
}

/// Whether the (main) mouse button is down right now.
pub fn mouse_down() -> bool {
    unsafe { CGEventSourceButtonState(0, 0) }
}

/// Open a file with its usual app.
pub fn open_path(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("That file isn't there any more.".into());
    }
    open_cmd(&[path])
}

/// Show a file in Finder, selected.
pub fn reveal_path(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("That file isn't there any more.".into());
    }
    open_cmd(&["-R", path])
}

pub fn clipboard_text() -> Option<String> {
    autoreleasepool(|_| {
        let pb = NSPasteboard::generalPasteboard();
        unsafe { pb.stringForType(NSPasteboardTypeString) }.map(|s| s.to_string())
    })
}

// ---------------------------------------------------------------- the process

/// Keep macOS from napping Cat Key (App Nap would delay keys).
pub fn opt_out_of_power_throttling_for_process() {
    use objc2_foundation::{NSActivityOptions, NSProcessInfo};
    autoreleasepool(|_| {
        let info = NSProcessInfo::processInfo();
        let token = info.beginActivityWithOptions_reason(
            NSActivityOptions::UserInitiatedAllowingIdleSystemSleep | NSActivityOptions::LatencyCritical,
            &ns("Cat Key handles keyboard shortcuts"),
        );
        std::mem::forget(token);
    });
}


static INSTANCE_LOCK: std::sync::OnceLock<Option<std::fs::File>> = std::sync::OnceLock::new();

/// Another Cat Key already runs (a lock file only one copy can hold).
pub fn another_instance_running() -> bool {
    use std::os::fd::AsRawFd;
    let held = INSTANCE_LOCK.get_or_init(|| {
        let path = crate::app::config_dir().join(".running.lock");
        let _ = std::fs::create_dir_all(crate::app::config_dir());
        let f = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(path).ok()?;
        let r = unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if r == 0 {
            Some(f)
        } else {
            None
        }
    });
    held.is_none()
}

pub fn allow_any_foreground() {}

pub fn wait_for_pid(pid: u32, timeout_ms: u32) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms as u64) {
        if unsafe { libc::kill(pid as i32, 0) } != 0 {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn local_tm() -> libc::tm {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        tm
    }
}

pub fn local_date() -> String {
    let tm = local_tm();
    format!("{:04}-{:02}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday)
}

/// "2026-11-26 0352": for backup folder names.
pub fn local_stamp() -> String {
    let tm = local_tm();
    format!("{:04}-{:02}-{:02} {:02}{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday, tm.tm_hour, tm.tm_min)
}

// ---------------------------------------------------------------- Windows-only things

pub fn is_elevated() -> bool {
    false
}
pub fn admin_task_exists() -> bool {
    false
}
pub fn create_admin_task() -> Result<(), String> {
    Err("Not needed on a Mac".into())
}
pub fn delete_admin_task() -> Result<(), String> {
    Ok(())
}
pub fn relaunch_elevated(_args: &str) -> Result<(), String> {
    Err("Not needed on a Mac".into())
}
pub fn relaunch_unelevated_after_exit() {}
pub fn process_running(_name: &str) -> bool {
    false
}
pub fn find_process_path(_name: &str) -> Option<String> {
    None
}
pub fn powertoys_grab_and_move_on() -> bool {
    false
}

// ---------------------------------------------------------------- permissions

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

/// Accessibility permission: what lets Cat Key see and change keys on a Mac.
pub fn keyboard_allowed() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Where the app runs from: a disk image or a quarantine copy can't keep permissions.
pub fn running_from_temp_place() -> bool {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    exe.starts_with("/Volumes/") || exe.contains("/AppTranslocation/")
}
