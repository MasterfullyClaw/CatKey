//! The app switcher on a Mac: where the pointer is, the screen under it, the apps with
//! windows (front to back, like the Mac's own ⌘Tab) and bringing one to the front.
//! Everything here is in points, with the origin at the top left of the main screen.

use crate::switcher::{Screen, SwitchItem};
use catkey_core::switcher::Rect;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSApplicationActivationOptions, NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use std::collections::HashMap;
use std::ffi::c_void;
use std::process::{Command, Stdio};
use std::ptr::null;
use std::time::{Duration, Instant};

type CFTypeRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreate(source: CFTypeRef) -> CFTypeRef;
    fn CGEventGetLocation(event: CFTypeRef) -> CGPoint;
    fn CGGetDisplaysWithPoint(point: CGPoint, max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> CGRect;
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFTypeRef;
    fn CGRectMakeWithDictionaryRepresentation(dict: CFTypeRef, rect: *mut CGRect) -> bool;
    static kCGWindowLayer: CFTypeRef;
    static kCGWindowOwnerPID: CFTypeRef;
    static kCGWindowName: CFTypeRef;
    static kCGWindowBounds: CFTypeRef;
    static kCGWindowAlpha: CFTypeRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(array: CFTypeRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFTypeRef, index: isize) -> CFTypeRef;
    fn CFDictionaryGetValue(dict: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
    fn CFNumberGetValue(number: CFTypeRef, kind: isize, out: *mut c_void) -> bool;
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
    fn AXUIElementCopyAttributeValue(element: CFTypeRef, attribute: CFTypeRef, value: *mut CFTypeRef) -> i32;
    fn AXUIElementSetMessagingTimeout(element: CFTypeRef, seconds: f32) -> i32;
}

const ON_SCREEN_ONLY: u32 = 1 << 0;
const EXCLUDE_DESKTOP: u32 = 1 << 4;
const CF_SINT64: isize = 4;
const CF_FLOAT64: isize = 6;

/// Where a mouse event happened (for the mouse tap).
pub(crate) fn event_location(event: *mut c_void) -> (f64, f64) {
    if event.is_null() {
        return (0.0, 0.0);
    }
    let p = unsafe { CGEventGetLocation(event as CFTypeRef) };
    (p.x, p.y)
}

/// Where the pointer is.
pub fn pointer() -> Option<(f64, f64)> {
    unsafe {
        let ev = CGEventCreate(null());
        if ev.is_null() {
            return None;
        }
        let p = CGEventGetLocation(ev);
        CFRelease(ev);
        Some((p.x, p.y))
    }
}

/// The screen under a point. The part free of the menu bar and the Dock comes from the
/// window side when the panel opens; here it's the whole screen.
pub fn screen_at(x: f64, y: f64) -> Option<Screen> {
    unsafe {
        let mut id = 0u32;
        let mut n = 0u32;
        if CGGetDisplaysWithPoint(CGPoint { x, y }, 1, &mut id, &mut n) != 0 || n == 0 {
            return None;
        }
        let b = CGDisplayBounds(id);
        let full = Rect::new(b.origin.x, b.origin.y, b.size.width, b.size.height);
        Some(Screen { full, work: full, scale: 1.0 })
    }
}

unsafe fn number(dict: CFTypeRef, key: CFTypeRef) -> Option<i64> {
    let v = CFDictionaryGetValue(dict, key);
    if v.is_null() || CFGetTypeID(v) != CFNumberGetTypeID() {
        return None;
    }
    let mut out: i64 = 0;
    CFNumberGetValue(v, CF_SINT64, &mut out as *mut i64 as *mut c_void).then_some(out)
}

unsafe fn float(dict: CFTypeRef, key: CFTypeRef) -> Option<f64> {
    let v = CFDictionaryGetValue(dict, key);
    if v.is_null() || CFGetTypeID(v) != CFNumberGetTypeID() {
        return None;
    }
    let mut out: f64 = 0.0;
    CFNumberGetValue(v, CF_FLOAT64, &mut out as *mut f64 as *mut c_void).then_some(out)
}

unsafe fn string(dict: CFTypeRef, key: CFTypeRef) -> Option<String> {
    let v = CFDictionaryGetValue(dict, key);
    if v.is_null() || CFGetTypeID(v) != CFStringGetTypeID() {
        return None;
    }
    Some(CFString::wrap_under_get_rule(v as _).to_string())
}

/// The title of an app's front window, asked through Accessibility (only when Cat Key
/// has it, which the keyboard needs anyway). Quick: an app that's busy gets 0.15 s.
fn front_window_title(pid: i32) -> Option<String> {
    unsafe {
        if !AXIsProcessTrusted() {
            return None;
        }
        let app = AXUIElementCreateApplication(pid);
        if app.is_null() {
            return None;
        }
        AXUIElementSetMessagingTimeout(app, 0.15);
        let focused = CFString::from_static_string("AXFocusedWindow");
        let title_key = CFString::from_static_string("AXTitle");
        let mut out = None;
        let mut win: CFTypeRef = null();
        if AXUIElementCopyAttributeValue(app, focused.as_concrete_TypeRef() as CFTypeRef, &mut win) == 0 && !win.is_null() {
            let mut t: CFTypeRef = null();
            if AXUIElementCopyAttributeValue(win, title_key.as_concrete_TypeRef() as CFTypeRef, &mut t) == 0 && !t.is_null() {
                if CFGetTypeID(t) == CFStringGetTypeID() {
                    out = Some(CFString::wrap_under_create_rule(t as _).to_string());
                } else {
                    CFRelease(t);
                }
            }
            CFRelease(win);
        }
        CFRelease(app);
        out.filter(|s| !s.trim().is_empty())
    }
}

/// The apps you can switch to: the ones with windows on screen first, front to back,
/// then the ones that are hidden or have no window open.
pub fn switch_windows() -> Vec<SwitchItem> {
    let own = std::process::id() as i32;
    let mut order: Vec<i32> = Vec::new();
    let mut names: HashMap<i32, String> = HashMap::new();
    unsafe {
        let list = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, 0);
        if !list.is_null() {
            for i in 0..CFArrayGetCount(list) {
                let d = CFArrayGetValueAtIndex(list, i);
                if d.is_null() || number(d, kCGWindowLayer).unwrap_or(1) != 0 {
                    continue;
                }
                let Some(pid) = number(d, kCGWindowOwnerPID).map(|p| p as i32) else { continue };
                if pid == own || float(d, kCGWindowAlpha).unwrap_or(1.0) <= 0.01 {
                    continue;
                }
                let mut r = CGRect::default();
                let b = CFDictionaryGetValue(d, kCGWindowBounds);
                if b.is_null() || !CGRectMakeWithDictionaryRepresentation(b, &mut r) || r.size.width < 40.0 || r.size.height < 40.0 {
                    continue;
                }
                if !order.contains(&pid) {
                    order.push(pid);
                    // Window names only come with Screen Recording; often there's none.
                    if let Some(t) = string(d, kCGWindowName).filter(|t| !t.trim().is_empty()) {
                        names.insert(pid, t);
                    }
                }
            }
            CFRelease(list);
        }
    }
    let started = Instant::now();
    autoreleasepool(|_| {
        let ws = NSWorkspace::sharedWorkspace();
        let front = ws.frontmostApplication().map(|a| a.processIdentifier());
        let mut apps: Vec<(usize, SwitchItem)> = Vec::new();
        for app in ws.runningApplications().iter() {
            if app.activationPolicy() != NSApplicationActivationPolicy::Regular {
                continue;
            }
            let pid = app.processIdentifier();
            if pid == own || app.isTerminated() {
                continue;
            }
            let bundle = app.bundleIdentifier().map(|s| s.to_string()).unwrap_or_default();
            let name = app.localizedName().map(|s| s.to_string()).unwrap_or_else(|| bundle.clone());
            let path = app.bundleURL().and_then(|u| u.path()).map(|p| p.to_string()).unwrap_or_default();
            let rank = order.iter().position(|p| *p == pid);
            let mut title = names.get(&pid).cloned().unwrap_or_default();
            if title.is_empty() && rank.is_some() && started.elapsed() < Duration::from_millis(250) {
                title = front_window_title(pid).unwrap_or_default();
            }
            apps.push((
                rank.unwrap_or(usize::MAX),
                SwitchItem {
                    id: pid.to_string(),
                    title,
                    app: name,
                    exe: bundle.to_lowercase(),
                    path,
                    front: Some(pid) == front,
                    min: rank.is_none() || app.isHidden(),
                    can_close: true,
                },
            ));
        }
        apps.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.app.to_lowercase().cmp(&b.1.app.to_lowercase())));
        apps.into_iter().map(|(_, it)| it).collect()
    })
}

/// The app whose window is at a point (front to back), for the coach. `None` when it's
/// the menu bar or the desktop.
pub fn pid_at(x: f64, y: f64) -> Option<i32> {
    unsafe {
        let list = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, 0);
        if list.is_null() {
            return None;
        }
        let mut found = None;
        for i in 0..CFArrayGetCount(list) {
            let d = CFArrayGetValueAtIndex(list, i);
            if d.is_null() || float(d, kCGWindowAlpha).unwrap_or(1.0) <= 0.01 {
                continue;
            }
            // The menu bar and the Dock (layers 20 and up, below pop-up menus) aren't an app's.
            let layer = number(d, kCGWindowLayer).unwrap_or(0);
            if (20..100).contains(&layer) {
                continue;
            }
            let mut r = CGRect::default();
            let b = CFDictionaryGetValue(d, kCGWindowBounds);
            if b.is_null() || !CGRectMakeWithDictionaryRepresentation(b, &mut r) {
                continue;
            }
            if x >= r.origin.x && x < r.origin.x + r.size.width && y >= r.origin.y && y < r.origin.y + r.size.height {
                found = number(d, kCGWindowOwnerPID).map(|p| p as i32);
                break;
            }
        }
        CFRelease(list);
        found
    }
}

fn open_app(path: &str) -> bool {
    !path.is_empty()
        && Command::new("/usr/bin/open")
            .arg("-a")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

/// Bring an app to the front. `reopen`: it has no window on screen, so ask it the way the
/// Dock does (which brings back a minimised window, or opens a new one).
pub fn switch_to(id: &str, reopen: bool) -> bool {
    let Ok(pid) = id.parse::<i32>() else { return false };
    let path = autoreleasepool(|_| {
        let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
        let path = app.bundleURL().and_then(|u| u.path()).map(|p| p.to_string()).unwrap_or_default();
        if !reopen {
            if app.isHidden() {
                app.unhide();
            }
            #[allow(deprecated)]
            app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows | NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
        Some(path)
    });
    let Some(path) = path else { return false };
    if reopen {
        return open_app(&path);
    }
    // Newer macOS can turn an app's request down; then the Dock's way always works.
    std::thread::sleep(Duration::from_millis(150));
    if crate::win::front_app().map(|(_, p)| p) != Some(pid) {
        return open_app(&path);
    }
    true
}

/// On a Mac the switcher lists apps, so closing one means quitting it (it can still ask
/// to save first).
pub fn switch_close(id: &str) -> bool {
    let Ok(pid) = id.parse::<i32>() else { return false };
    autoreleasepool(|_| NSRunningApplication::runningApplicationWithProcessIdentifier(pid).map(|a| a.terminate()).unwrap_or(false))
}

/// Apps always have their bundle's icon on a Mac.
pub fn window_icon(_id: &str) -> Option<String> {
    None
}
