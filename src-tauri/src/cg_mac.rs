//! CatGPT on a Mac: screenshots and very light screen recordings with macOS's own tools
//! (screencapture, then sips or avconvert to make them small), the AI apps, the keys that
//! paste into them, and the screen-recording permission CatGPT needs.

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

type CGEventRef = *mut c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGGetDisplaysWithPoint(point: CGPoint, max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
    fn CGEventCreateKeyboardEvent(source: *mut c_void, keycode: u16, down: bool) -> CGEventRef;
    fn CGEventSetFlags(ev: CGEventRef, flags: u64);
    fn CGEventSetIntegerValueField(ev: CGEventRef, field: u32, value: i64);
    fn CGEventPost(tap: u32, ev: CGEventRef);
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

/// Cat Key's own key events carry this mark, so its keyboard engine leaves them alone.
const MARK: i64 = 0x0CA7_CA7E;
const FIELD_USER_DATA: u32 = 42;

/// screencapture's number for the screen under the pointer (1 = the main screen).
fn display_number() -> u32 {
    unsafe {
        let Some((x, y)) = crate::mac_switch::pointer() else { return 1 };
        let mut id = 0u32;
        let mut n = 0u32;
        if CGGetDisplaysWithPoint(CGPoint { x, y }, 1, &mut id, &mut n) != 0 || n == 0 {
            return 1;
        }
        let mut list = [0u32; 16];
        let mut count = 0u32;
        if CGGetActiveDisplayList(16, list.as_mut_ptr(), &mut count) != 0 {
            return 1;
        }
        list[..count as usize].iter().position(|d| *d == id).map(|i| i as u32 + 1).unwrap_or(1)
    }
}

/// Whether macOS lets Cat Key see other apps' windows (Screen & System Audio Recording).
pub fn can_record() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

/// Ask macOS for the screen-recording permission (it shows its own prompt once).
pub fn ask_to_record() -> bool {
    unsafe { CGRequestScreenCaptureAccess() }
}

/// Open System Settings at Screen & System Audio Recording.
pub fn open_record_settings() {
    let _ = crate::win::open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture");
}

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd).args(args).stdin(Stdio::null()).output().map_err(|e| format!("{cmd}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() { format!("{cmd} stopped ({})", out.status) } else { err });
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Shrink a picture to `max_w` wide as a JPEG of `quality` (macOS's sips). Returns (w, h).
fn shrink(src: &Path, out: &Path, max_w: u32, quality: u8) -> Result<(u32, u32), String> {
    let (w, _) = size_of_image(src).unwrap_or((max_w + 1, 0));
    let s = src.to_string_lossy().to_string();
    let o = out.to_string_lossy().to_string();
    let q = quality.to_string();
    let m = max_w.to_string();
    let mut args: Vec<&str> = vec!["-s", "format", "jpeg", "-s", "formatOptions", &q];
    if w > max_w {
        args.extend(["--resampleWidth", &m]);
    }
    args.extend([s.as_str(), "--out", o.as_str()]);
    run("/usr/bin/sips", &args)?;
    size_of_image(out).ok_or_else(|| "The picture couldn't be read back.".to_string())
}

fn size_of_image(p: &Path) -> Option<(u32, u32)> {
    let out = run("/usr/bin/sips", &["-g", "pixelWidth", "-g", "pixelHeight", &p.to_string_lossy()]).ok()?;
    let num = |k: &str| out.lines().find(|l| l.trim_start().starts_with(k)).and_then(|l| l.split(':').nth(1)).and_then(|v| v.trim().parse().ok());
    Some((num("pixelWidth")?, num("pixelHeight")?))
}

/// A picture of the screen under the pointer, as a small JPEG. Returns (width, height).
pub fn screenshot(out: &Path, max_w: u32, quality: u8) -> Result<(u32, u32), String> {
    if !can_record() {
        return Err(NEEDS_PERMISSION.into());
    }
    let tmp = out.with_extension("full.jpg");
    let d = display_number().to_string();
    let t = tmp.to_string_lossy().to_string();
    run("/usr/sbin/screencapture", &["-x", "-C", "-t", "jpg", "-D", &d, &t])?;
    let r = shrink(&tmp, out, max_w, quality);
    let _ = std::fs::remove_file(&tmp);
    r
}

pub const NEEDS_PERMISSION: &str = "macOS needs your OK first: allow Cat Key in System Settings → Privacy & Security → Screen & System Audio Recording, then open Cat Key again.";

// ------------------------------------------------------------------ light screen recording

pub struct Recording {
    child: Child,
    raw: PathBuf,
    out: PathBuf,
    started: Instant,
    stop_stills: Arc<AtomicBool>,
    stills: std::thread::JoinHandle<Vec<String>>,
}

pub struct RecDone {
    pub w: u32,
    pub h: u32,
    pub secs: f64,
    pub frames: Vec<String>,
}

/// Start recording the screen under the pointer. macOS's screencapture records it; when it
/// stops, avconvert makes it small. Stills for the AI are taken every 3 seconds.
pub fn record_start(out: &Path, frames_dir: &Path, max_secs: u64) -> Result<Recording, String> {
    if !can_record() {
        return Err(NEEDS_PERMISSION.into());
    }
    let raw = out.with_extension("raw.mov");
    let d = display_number().to_string();
    let secs = max_secs.to_string();
    let child = Command::new("/usr/sbin/screencapture")
        .args(["-v", "-x", "-k", "-V", &secs, "-D", &d, &raw.to_string_lossy()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("screencapture: {e}"))?;
    let stop_stills = Arc::new(AtomicBool::new(false));
    let (stop2, dir, num) = (stop_stills.clone(), frames_dir.to_path_buf(), d.clone());
    let stills = std::thread::spawn(move || {
        let mut out = Vec::new();
        let start = Instant::now();
        let mut next = Duration::from_millis(600);
        while !stop2.load(Ordering::SeqCst) && out.len() < 40 {
            if start.elapsed() >= next {
                let full = dir.join(format!("still{:02}.jpg", out.len() + 1));
                let p = dir.join(format!("frame{:02}.jpg", out.len() + 1));
                let f = full.to_string_lossy().to_string();
                if run("/usr/sbin/screencapture", &["-x", "-C", "-t", "jpg", "-D", &num, &f]).is_ok() && shrink(&full, &p, 1600, 72).is_ok() {
                    out.push(p.to_string_lossy().to_string());
                }
                let _ = std::fs::remove_file(&full);
                next += Duration::from_secs(3);
            }
            std::thread::sleep(Duration::from_millis(150));
        }
        out
    });
    Ok(Recording { child, raw, out: out.to_path_buf(), started: Instant::now(), stop_stills, stills })
}

impl Recording {
    pub fn stop(mut self) -> Result<RecDone, String> {
        let secs = self.started.elapsed().as_secs_f64();
        self.stop_stills.store(true, Ordering::SeqCst);
        unsafe {
            libc::kill(self.child.id() as i32, libc::SIGINT);
        }
        let until = Instant::now() + Duration::from_secs(20);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(100)),
                _ => {
                    let _ = self.child.kill();
                    break;
                }
            }
        }
        let frames = self.stills.join().unwrap_or_default();
        if !self.raw.exists() {
            return Err("macOS didn't save the recording (is Screen Recording allowed for Cat Key?).".into());
        }
        // Make it light: 1280×720 H.264 is small and still readable. If that fails, keep the original.
        let small = run(
            "/usr/bin/avconvert",
            &["--preset", "Preset1280x720", "--source", &self.raw.to_string_lossy(), "--output", &self.out.to_string_lossy(), "--replace"],
        );
        if small.is_ok() && self.out.exists() {
            let _ = std::fs::remove_file(&self.raw);
        } else {
            std::fs::rename(&self.raw, &self.out).map_err(|e| e.to_string())?;
        }
        Ok(RecDone { w: 1280, h: 720, secs, frames })
    }
}

// ------------------------------------------------------------------ the AI apps

/// The app in front (bundle id, lower case).
pub fn front_bundle() -> String {
    crate::win::front_app().map(|(b, _)| b).unwrap_or_default()
}

/// Bring an app to the front by its bundle id (opens it if it isn't running).
pub fn activate(bundle: &str) -> bool {
    crate::win::launch_app_id(bundle).is_ok()
}

fn keycode(k: &str) -> Option<u16> {
    Some(match k {
        "v" => 9,
        "n" => 45,
        "o" => 31,
        "k" => 40,
        "g" => 5,
        "enter" => 36,
        "tab" => 48,
        "1" => 18,
        "2" => 19,
        "3" => 20,
        "4" => 21,
        "5" => 23,
        "6" => 22,
        "7" => 26,
        "8" => 28,
        "9" => 25,
        _ => return None,
    })
}

/// Press a shortcut in whatever is in front: modifiers ("cmd", "alt", "ctrl", "shift") and a key.
pub fn press(mods: &[&str], k: &str) {
    let Some(code) = keycode(k) else { return };
    let mut flags = 0u64;
    for m in mods {
        flags |= match *m {
            "cmd" => 0x0010_0000,
            "alt" => 0x0008_0000,
            "ctrl" => 0x0004_0000,
            "shift" => 0x0002_0000,
            _ => 0,
        };
    }
    unsafe {
        for down in [true, false] {
            let ev = CGEventCreateKeyboardEvent(std::ptr::null_mut(), code, down);
            if ev.is_null() {
                return;
            }
            CGEventSetFlags(ev, flags);
            CGEventSetIntegerValueField(ev, FIELD_USER_DATA, MARK);
            CGEventPost(0, ev);
            CFRelease(ev as *const c_void);
            std::thread::sleep(Duration::from_millis(12));
        }
    }
}

pub fn paste() {
    press(&["cmd"], "v");
}

/// Float a window above everything, the menu bar included (the drawing canvas), and on
/// every Space and over full-screen apps.
pub fn float_above_all(app: &tauri::AppHandle, w: &tauri::WebviewWindow, level: isize) {
    let Ok(ns) = w.ns_window() else { return };
    let ns = ns as usize;
    let _ = app.run_on_main_thread(move || unsafe {
        let win = ns as *mut objc2::runtime::AnyObject;
        if win.is_null() {
            return;
        }
        let _: () = objc2::msg_send![&*win, setLevel: level];
        // can join all Spaces (1) + full-screen auxiliary (256)
        let _: () = objc2::msg_send![&*win, setCollectionBehavior: 257usize];
    });
}

/// The app in front now, as the app switcher names it (its process id), to go back to later.
pub fn front_id() -> Option<String> {
    crate::win::front_app().map(|(_, pid)| pid.to_string()).filter(|p| p != &std::process::id().to_string())
}

// ------------------------------------------------------------------ Claude's recent chats

type CFRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateApplication(pid: i32) -> CFRef;
    fn AXUIElementCopyAttributeValue(element: CFRef, attribute: CFRef, value: *mut CFRef) -> i32;
    fn AXUIElementSetAttributeValue(element: CFRef, attribute: CFRef, value: CFRef) -> i32;
    fn AXUIElementSetMessagingTimeout(element: CFRef, seconds: f32) -> i32;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFBooleanTrue: CFRef;
    fn CFRetain(cf: CFRef) -> CFRef;
    fn CFGetTypeID(cf: CFRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFURLGetTypeID() -> usize;
    fn CFArrayGetCount(arr: CFRef) -> isize;
    fn CFArrayGetValueAtIndex(arr: CFRef, i: isize) -> CFRef;
    fn CFURLGetString(url: CFRef) -> CFRef;
}

/// An Accessibility object we own (released when dropped).
struct Ax(CFRef);
impl Drop for Ax {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) };
        }
    }
}

fn cfs(s: &str) -> core_foundation::string::CFString {
    core_foundation::string::CFString::new(s)
}

fn cf_to_string(v: CFRef) -> Option<String> {
    use core_foundation::base::TCFType;
    unsafe {
        if v.is_null() || CFGetTypeID(v) != CFStringGetTypeID() {
            return None;
        }
        Some(core_foundation::string::CFString::wrap_under_get_rule(v as _).to_string())
    }
}

impl Ax {
    fn copy(&self, attr: &str) -> Option<Ax> {
        use core_foundation::base::TCFType;
        let name = cfs(attr);
        let mut v: CFRef = std::ptr::null();
        let ok = unsafe { AXUIElementCopyAttributeValue(self.0, name.as_concrete_TypeRef() as CFRef, &mut v) } == 0;
        (ok && !v.is_null()).then_some(Ax(v))
    }
    fn text(&self, attr: &str) -> Option<String> {
        self.copy(attr).and_then(|v| cf_to_string(v.0)).filter(|s| !s.trim().is_empty())
    }
    fn url(&self) -> Option<String> {
        let v = self.copy("AXURL")?;
        unsafe {
            if CFGetTypeID(v.0) == CFURLGetTypeID() {
                cf_to_string(CFURLGetString(v.0))
            } else {
                cf_to_string(v.0)
            }
        }
    }
    fn children(&self) -> Vec<Ax> {
        let Some(arr) = self.copy("AXChildren") else { return Vec::new() };
        unsafe {
            if CFGetTypeID(arr.0) != CFArrayGetTypeID() {
                return Vec::new();
            }
            (0..CFArrayGetCount(arr.0)).map(|i| Ax(CFRetain(CFArrayGetValueAtIndex(arr.0, i)))).collect()
        }
    }
    /// A link's words: its title, or the text inside it.
    fn label(&self) -> Option<String> {
        if let Some(t) = self.text("AXTitle").or_else(|| self.text("AXDescription")) {
            return Some(t);
        }
        let mut stack = self.children();
        let mut n = 0;
        while let Some(c) = stack.pop() {
            n += 1;
            if n > 40 {
                break;
            }
            if let Some(v) = c.text("AXValue").or_else(|| c.text("AXTitle")) {
                return Some(v);
            }
            stack.extend(c.children());
        }
        None
    }
}

fn claude_pid() -> Option<i32> {
    use objc2::rc::autoreleasepool;
    autoreleasepool(|_| {
        let apps = objc2_app_kit::NSRunningApplication::runningApplicationsWithBundleIdentifier(&objc2_foundation::NSString::from_str(
            "com.anthropic.claudefordesktop",
        ));
        apps.iter().next().map(|a| a.processIdentifier())
    })
}

/// The chats in the Claude app's sidebar, most recent first: (title, chat id). Read the way a
/// screen reader reads them (Cat Key already has the Accessibility permission).
pub fn claude_recent() -> Vec<(String, String)> {
    use core_foundation::base::TCFType;
    let Some(pid) = claude_pid() else { return Vec::new() };
    if !unsafe { AXIsProcessTrusted() } {
        return Vec::new();
    }
    let app = Ax(unsafe { AXUIElementCreateApplication(pid) });
    if app.0.is_null() {
        return Vec::new();
    }
    unsafe {
        AXUIElementSetMessagingTimeout(app.0, 0.4);
        // Electron apps build their accessibility tree only when it's asked for.
        let first = AXUIElementSetAttributeValue(app.0, cfs("AXManualAccessibility").as_concrete_TypeRef() as CFRef, kCFBooleanTrue) == 0;
        if first {
            std::thread::sleep(Duration::from_millis(250));
        }
    }
    let mut out: Vec<(String, String)> = Vec::new();
    let mut queue: std::collections::VecDeque<(Ax, u32)> = app.children().into_iter().map(|c| (c, 0)).collect();
    let start = Instant::now();
    let mut seen = 0;
    while let Some((el, depth)) = queue.pop_front() {
        seen += 1;
        if seen > 9000 || start.elapsed() > Duration::from_secs(4) || out.len() >= 15 {
            break;
        }
        if el.text("AXRole").as_deref() == Some("AXLink") {
            if let Some(url) = el.url() {
                if let Some(id) = url.split("/chat/").nth(1).map(|s| s.chars().take_while(|c| c.is_ascii_hexdigit() || *c == '-').collect::<String>()) {
                    if id.len() >= 32 && !out.iter().any(|(_, i)| *i == id) {
                        let title = el.label().unwrap_or_else(|| "A chat".into());
                        out.push((title.chars().take(80).collect(), id));
                    }
                }
            }
            continue;
        }
        if depth < 60 {
            queue.extend(el.children().into_iter().map(|c| (c, depth + 1)));
        }
    }
    out
}
