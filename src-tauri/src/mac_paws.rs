//! Mac: the mouse for paw prints and the shortcut coach, from a listen-only event tap on
//! its own thread. It only listens (it can't hold anything up), and it only starts once
//! paw prints or the coach are on.

use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

type CGEventRef = *mut c_void;
type CFMachPortRef = *mut c_void;
type TapCallback = extern "C" fn(*mut c_void, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(tap: u32, place: u32, options: u32, mask: u64, callback: TapCallback, info: *mut c_void) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
}

fn at(event: CGEventRef) -> (f64, f64) {
    crate::mac_switch::event_location(event)
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(alloc: *const c_void, port: CFMachPortRef, order: isize) -> *mut c_void;
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopRun();
    static kCFRunLoopCommonModes: *const c_void;
}

const SESSION_TAP: u32 = 1;
const TAIL_APPEND: u32 = 1;
const LISTEN_ONLY: u32 = 1;
const LEFT_DOWN: u32 = 1;
const LEFT_UP: u32 = 2;
const RIGHT_DOWN: u32 = 3;
const LEFT_DRAGGED: u32 = 6;
const SCROLL: u32 = 22;
const DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const DISABLED_USER: u32 = 0xFFFF_FFFF;
const FIELD_SCROLL_Y: u32 = 11;

static STARTED: AtomicBool = AtomicBool::new(false);
static TAP: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

extern "C" fn callback(_proxy: *mut c_void, kind: u32, event: CGEventRef, _info: *mut c_void) -> CGEventRef {
    use crate::paws::{input, Input};
    match kind {
        DISABLED_TIMEOUT | DISABLED_USER => {
            let tap = TAP.load(Ordering::SeqCst);
            if !tap.is_null() {
                unsafe { CGEventTapEnable(tap, true) };
            }
        }
        LEFT_DOWN => {
            input(Input::Down);
            let (x, y) = at(event);
            crate::coach::click(x, y, true);
        }
        LEFT_UP => {
            input(Input::Up);
            let (x, y) = at(event);
            crate::coach::click(x, y, false);
        }
        RIGHT_DOWN => input(Input::Right),
        LEFT_DRAGGED => input(Input::Move),
        SCROLL if !event.is_null() => {
            let dy = unsafe { CGEventGetIntegerValueField(event, FIELD_SCROLL_Y) };
            if dy != 0 {
                input(Input::Scroll(dy as i32));
            }
        }
        _ => {}
    }
    event
}

/// Start listening (once; it stays, and does nothing while paw prints are off).
pub fn start() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("catkey-paws-mouse".into())
        .spawn(|| unsafe {
            let mask: u64 = (1 << LEFT_DOWN) | (1 << LEFT_UP) | (1 << RIGHT_DOWN) | (1 << LEFT_DRAGGED) | (1 << SCROLL);
            let tap = CGEventTapCreate(SESSION_TAP, TAIL_APPEND, LISTEN_ONLY, mask, callback, null_mut());
            if tap.is_null() {
                crate::log("paws: couldn't listen to the mouse (Accessibility permission?)");
                STARTED.store(false, Ordering::SeqCst);
                return;
            }
            TAP.store(tap, Ordering::SeqCst);
            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);
            crate::log("paws: listening to the mouse");
            CFRunLoopRun();
        })
        .ok();
}
