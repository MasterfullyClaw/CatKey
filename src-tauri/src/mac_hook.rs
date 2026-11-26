//! The keyboard on a Mac: a Quartz event tap.
//!
//! The tap sees every key before apps do. Mac key codes become the same virtual keys the
//! engine uses on Windows (⌘ is the Win key, ⌥ is Alt, ⌃ is Ctrl), the engine decides,
//! and the keys it sends are posted back marked as ours, so the tap lets them through.
//! On a Mac each key event carries the modifiers held with it, so Cat Key keeps its own
//! picture of what apps should see as held, and stamps every key with it.

use catkey_core::engine::{Effect, Engine, Env, KeyEvent, Out};
use catkey_core::keys::key_name;
use catkey_core::Compiled;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub enum Cmd {
    Config(Arc<Compiled>),
    Enabled(bool),
    StartRecord,
    StopRecord,
    Quit,
}

/// Keys the watchdog fixed (Windows only; always 0 here).
pub static WATCH_FIXES: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------- Quartz

type CGEventRef = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CGEventSourceRef = *mut c_void;
type TapCallback = extern "C" fn(*mut c_void, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(tap: u32, place: u32, options: u32, mask: u64, callback: TapCallback, info: *mut c_void) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventSetType(event: CGEventRef, kind: u32);
    fn CGEventCreateKeyboardEvent(source: CGEventSourceRef, keycode: u16, down: bool) -> CGEventRef;
    fn CGEventKeyboardSetUnicodeString(event: CGEventRef, len: usize, s: *const u16);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGEventSourceCreate(state: i32) -> CGEventSourceRef;
    fn CGEventSourceSetLocalEventsSuppressionInterval(source: CGEventSourceRef, seconds: f64);
    fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    fn CGEventSourceFlagsState(state: i32) -> u64;
    fn CGRequestListenEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(alloc: *const c_void, port: CFMachPortRef, order: isize) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: CFRunLoopRef);
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopCommonModes: *const c_void;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

const HID_TAP: u32 = 0;
const SESSION_TAP: u32 = 1;
const HEAD_INSERT: u32 = 0;
const TAP_DEFAULT: u32 = 0;
const KEY_DOWN: u32 = 10;
const KEY_UP: u32 = 11;
const FLAGS_CHANGED: u32 = 12;
const TAP_DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const TAP_DISABLED_USER: u32 = 0xFFFF_FFFF;
const FIELD_AUTOREPEAT: u32 = 8;
const FIELD_KEYCODE: u32 = 9;
const FIELD_SOURCE_PID: u32 = 41;
const FIELD_USER_DATA: u32 = 42;
const STATE_PRIVATE: i32 = -1;
const STATE_COMBINED: i32 = 0;
const STATE_HID: i32 = 1;

/// Stamped on every key we post, so the tap lets ours through.
const MARK: i64 = 0x0CA7_CA7E;

// Modifier flags (CGEventFlags) and their left/right "device" bits.
const F_CAPS: u64 = 0x0001_0000;
const F_SHIFT: u64 = 0x0002_0000;
const F_CTRL: u64 = 0x0004_0000;
const F_ALT: u64 = 0x0008_0000;
const F_CMD: u64 = 0x0010_0000;
const F_NUMPAD: u64 = 0x0020_0000;
const F_FN: u64 = 0x0080_0000;
const D_LCTRL: u64 = 0x0000_0001;
const D_LSHIFT: u64 = 0x0000_0002;
const D_RSHIFT: u64 = 0x0000_0004;
const D_LCMD: u64 = 0x0000_0008;
const D_RCMD: u64 = 0x0000_0010;
const D_LALT: u64 = 0x0000_0020;
const D_RALT: u64 = 0x0000_0040;
const D_RCTRL: u64 = 0x0000_2000;
const MOD_FLAGS: u64 = F_SHIFT | F_CTRL | F_ALT | F_CMD | D_LCTRL | D_LSHIFT | D_RSHIFT | D_LCMD | D_RCMD | D_LALT | D_RALT | D_RCTRL;

/// The eight modifier keys: (virtual key, Mac key code, family flag, device bit).
const MODS: [(u8, u16, u64, u64); 8] = [
    (0xA2, 0x3B, F_CTRL, D_LCTRL),
    (0xA3, 0x3E, F_CTRL, D_RCTRL),
    (0xA0, 0x38, F_SHIFT, D_LSHIFT),
    (0xA1, 0x3C, F_SHIFT, D_RSHIFT),
    (0xA4, 0x3A, F_ALT, D_LALT),
    (0xA5, 0x3D, F_ALT, D_RALT),
    (0x5B, 0x37, F_CMD, D_LCMD),
    (0x5C, 0x36, F_CMD, D_RCMD),
];

fn mod_index(vk: u8) -> Option<usize> {
    MODS.iter().position(|m| m.0 == vk)
}

/// Mac key code → the engine's virtual key (0 = not a key Cat Key handles).
pub(crate) fn mac_to_vk(code: u16) -> u8 {
    match code {
        0x00 => b'A', 0x0B => b'B', 0x08 => b'C', 0x02 => b'D', 0x0E => b'E', 0x03 => b'F', 0x05 => b'G', 0x04 => b'H',
        0x22 => b'I', 0x26 => b'J', 0x28 => b'K', 0x25 => b'L', 0x2E => b'M', 0x2D => b'N', 0x1F => b'O', 0x23 => b'P',
        0x0C => b'Q', 0x0F => b'R', 0x01 => b'S', 0x11 => b'T', 0x20 => b'U', 0x09 => b'V', 0x0D => b'W', 0x07 => b'X',
        0x10 => b'Y', 0x06 => b'Z',
        0x1D => b'0', 0x12 => b'1', 0x13 => b'2', 0x14 => b'3', 0x15 => b'4', 0x17 => b'5', 0x16 => b'6', 0x1A => b'7',
        0x1C => b'8', 0x19 => b'9',
        0x24 | 0x4C => 0x0D, // Return, keypad Enter
        0x30 => 0x09,        // Tab
        0x31 => 0x20,        // Space
        0x33 => 0x08,        // delete (Backspace)
        0x35 => 0x1B,        // Esc
        0x75 => 0x2E,        // forward delete
        0x72 => 0x2D,        // Help (where Insert sits)
        0x73 => 0x24, 0x77 => 0x23, 0x74 => 0x21, 0x79 => 0x22, // Home End PageUp PageDown
        0x7B => 0x25, 0x7E => 0x26, 0x7C => 0x27, 0x7D => 0x28, // arrows
        0x7A => 0x70, 0x78 => 0x71, 0x63 => 0x72, 0x76 => 0x73, 0x60 => 0x74, 0x61 => 0x75, 0x62 => 0x76, 0x64 => 0x77,
        0x65 => 0x78, 0x6D => 0x79, 0x67 => 0x7A, 0x6F => 0x7B, 0x69 => 0x7C, 0x6B => 0x7D, 0x71 => 0x7E, 0x6A => 0x7F,
        0x40 => 0x80, 0x4F => 0x81, 0x50 => 0x82, 0x5A => 0x83, // F1–F20
        0x29 => 0xBA, 0x18 => 0xBB, 0x2B => 0xBC, 0x1B => 0xBD, 0x2F => 0xBE, 0x2C => 0xBF, 0x32 => 0xC0,
        0x21 => 0xDB, 0x2A => 0xDC, 0x1E => 0xDD, 0x27 => 0xDE, 0x0A => 0xE2,
        0x52 => 0x60, 0x53 => 0x61, 0x54 => 0x62, 0x55 => 0x63, 0x56 => 0x64, 0x57 => 0x65, 0x58 => 0x66, 0x59 => 0x67,
        0x5B => 0x68, 0x5C => 0x69, 0x43 => 0x6A, 0x45 => 0x6B, 0x4E => 0x6D, 0x41 => 0x6E, 0x4B => 0x6F, 0x47 => 0x0C,
        0x48 => 0xAF, 0x49 => 0xAE, 0x4A => 0xAD, // volume up / down / mute
        _ => MODS.iter().find(|m| m.1 == code).map(|m| m.0).unwrap_or(0),
    }
}

/// The engine's virtual key → a Mac key code.
fn vk_to_mac(vk: u8) -> Option<u16> {
    static TABLE: OnceLock<[u16; 256]> = OnceLock::new();
    let t = TABLE.get_or_init(|| {
        let mut t = [u16::MAX; 256];
        // Fill from the other direction; the first Mac key wins (Return before keypad Enter).
        for code in (0u16..0x80).rev() {
            let vk = mac_to_vk(code);
            if vk != 0 {
                t[vk as usize] = code;
            }
        }
        t[0x0D] = 0x24;
        t
    });
    let c = t[vk as usize];
    (c != u16::MAX).then_some(c)
}

/// Extra flags real Mac keys carry (arrows and the like report fn and keypad).
fn key_flags(code: u16) -> u64 {
    match code {
        0x7B..=0x7E => F_FN | F_NUMPAD,
        0x72..=0x79 | 0x60..=0x6F | 0x7A | 0x40 | 0x4F | 0x50 | 0x5A => F_FN,
        0x41..=0x5C if code != 0x48 && code != 0x49 && code != 0x4A => F_NUMPAD,
        _ => 0,
    }
}

// ---------------------------------------------------------------- state

struct Ctx {
    engine: Engine,
    env: MacEnv,
    effects: Sender<Effect>,
    /// Modifiers held on the keyboard (bits in MODS order), from the events we see.
    phys: u8,
    /// Modifiers apps should see as held, which can differ from `phys` while a
    /// shortcut is being replaced (⌥ let go and ⌘ held in its place, say).
    shown: u8,
    track_down: [bool; 256],
    cap_down: [bool; 256],
    source: usize,
    surrogate: u16,
}

static CTX: Mutex<Option<Ctx>> = Mutex::new(None);
static TAP: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static RUN_LOOP: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static RUNNING: AtomicBool = AtomicBool::new(false);
static CAPTURE: AtomicBool = AtomicBool::new(false);
static TEST_KEYS: AtomicBool = AtomicBool::new(false);
static T0: OnceLock<Instant> = OnceLock::new();

fn now_ms() -> u32 {
    T0.get_or_init(Instant::now).elapsed().as_millis() as u32
}

fn bits_to_flags(bits: u8) -> u64 {
    let mut f = 0;
    for (i, m) in MODS.iter().enumerate() {
        if bits & (1 << i) != 0 {
            f |= m.2 | m.3;
        }
    }
    f
}

fn flags_to_bits(flags: u64) -> u8 {
    let device = flags & (D_LCTRL | D_LSHIFT | D_RSHIFT | D_LCMD | D_RCMD | D_LALT | D_RALT | D_RCTRL) != 0;
    let mut bits = 0u8;
    for (i, m) in MODS.iter().enumerate() {
        let on = if device { flags & m.3 != 0 } else { flags & m.2 != 0 && i % 2 == 0 };
        if on {
            bits |= 1 << i;
        }
    }
    bits
}

struct MacEnv {
    bundle: String,
    pid: i32,
    checked: Option<Instant>,
}

impl MacEnv {
    fn refresh(&mut self) {
        if self.checked.map(|t| t.elapsed() < Duration::from_millis(150)).unwrap_or(false) {
            return;
        }
        self.checked = Some(Instant::now());
        if let Some((b, pid)) = crate::win::front_app() {
            self.bundle = b;
            self.pid = pid;
        }
    }
}

impl Env for MacEnv {
    fn foreground_exe(&mut self) -> &str {
        self.refresh();
        &self.bundle
    }
    fn key_down_async(&mut self, vk: u8) -> bool {
        vk_to_mac(vk).map(|c| unsafe { CGEventSourceKeyState(STATE_HID, c) }).unwrap_or(false)
    }
    fn focus(&mut self) -> usize {
        self.refresh();
        self.pid as usize
    }
}

fn own_app_in_front() -> bool {
    crate::win::front_app().map(|(_, pid)| pid == std::process::id() as i32).unwrap_or(false)
}

// ---------------------------------------------------------------- sending

impl Ctx {
    fn post_key(&mut self, code: u16, down: bool, modifier: bool) {
        unsafe {
            let ev = CGEventCreateKeyboardEvent(self.source as CGEventSourceRef, code, down);
            if ev.is_null() {
                return;
            }
            let caps = CGEventSourceFlagsState(STATE_COMBINED) & F_CAPS;
            let flags = bits_to_flags(self.shown) | caps | if modifier { 0 } else { key_flags(code) };
            if modifier {
                CGEventSetType(ev, FLAGS_CHANGED);
            }
            CGEventSetFlags(ev, flags);
            CGEventSetIntegerValueField(ev, FIELD_USER_DATA, MARK);
            CGEventPost(HID_TAP, ev);
            CFRelease(ev as *const c_void);
        }
    }

    fn post_text(&mut self, units: &[u16]) {
        unsafe {
            for down in [true, false] {
                let ev = CGEventCreateKeyboardEvent(self.source as CGEventSourceRef, 0, down);
                if ev.is_null() {
                    return;
                }
                CGEventKeyboardSetUnicodeString(ev, units.len(), units.as_ptr());
                CGEventSetFlags(ev, 0);
                CGEventSetIntegerValueField(ev, FIELD_USER_DATA, MARK);
                CGEventPost(HID_TAP, ev);
                CFRelease(ev as *const c_void);
            }
        }
    }

    /// Send what the engine asked for, in order.
    fn send_out(&mut self) {
        let outs: Vec<Out> = std::mem::take(&mut self.engine.out);
        for o in outs {
            match o {
                Out::Key { vk, down } | Out::Raw { vk, down, .. } => {
                    if let Some(i) = mod_index(vk) {
                        if down {
                            self.shown |= 1 << i;
                        } else {
                            self.shown &= !(1 << i);
                        }
                        self.post_key(MODS[i].1, down, true);
                    } else if let Some(code) = vk_to_mac(vk) {
                        self.post_key(code, down, false);
                    }
                }
                Out::Unicode { unit, down: true } => {
                    if (0xD800..0xDC00).contains(&unit) {
                        self.surrogate = unit;
                    } else if (0xDC00..0xE000).contains(&unit) && self.surrogate != 0 {
                        let pair = [self.surrogate, unit];
                        self.surrogate = 0;
                        self.post_text(&pair);
                    } else {
                        self.post_text(&[unit]);
                    }
                }
                Out::Unicode { .. } => {}
                Out::Dummy => {} // Windows' menu-bar guard; nothing to guard on a Mac
            }
        }
    }

    fn flush_effects(&mut self) {
        for e in self.engine.effects.drain(..) {
            let _ = self.effects.send(e);
        }
    }
}

// ---------------------------------------------------------------- the tap

extern "C" fn tap_callback(_proxy: *mut c_void, kind: u32, event: CGEventRef, _info: *mut c_void) -> CGEventRef {
    if kind == TAP_DISABLED_TIMEOUT || kind == TAP_DISABLED_USER {
        let tap = TAP.load(Ordering::SeqCst);
        if !tap.is_null() {
            unsafe { CGEventTapEnable(tap, true) };
        }
        return event;
    }
    if !(kind == KEY_DOWN || kind == KEY_UP || kind == FLAGS_CHANGED) || event.is_null() {
        return event;
    }
    if unsafe { CGEventGetIntegerValueField(event, FIELD_USER_DATA) } == MARK {
        return event; // a key Cat Key sent
    }
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(kind, event))) {
        Ok(true) => null_mut(),
        Ok(false) => event,
        Err(_) => {
            recover();
            event
        }
    }
}

fn handle(kind: u32, event: CGEventRef) -> bool {
    let code = unsafe { CGEventGetIntegerValueField(event, FIELD_KEYCODE) } as u16;
    let flags = unsafe { CGEventGetFlags(event) };
    let (vk, down) = match kind {
        KEY_DOWN => (mac_to_vk(code), true),
        KEY_UP => (mac_to_vk(code), false),
        _ => match MODS.iter().position(|m| m.1 == code) {
            Some(i) => {
                let m = MODS[i];
                let device = flags & (D_LCTRL | D_LSHIFT | D_RSHIFT | D_LCMD | D_RCMD | D_LALT | D_RALT | D_RCTRL) != 0;
                (m.0, if device { flags & m.3 != 0 } else { flags & m.2 != 0 })
            }
            None => return false, // Caps Lock and fn pass as they are
        },
    };
    if vk == 0 {
        return false;
    }
    let Ok(mut guard) = CTX.lock() else { return false };
    let Some(ctx) = guard.as_mut() else { return false };

    let from_program = unsafe { CGEventGetIntegerValueField(event, FIELD_SOURCE_PID) } != 0;
    let ev = KeyEvent {
        vk,
        scan: code,
        down,
        extended: false,
        time: now_ms(),
        injected: from_program && !TEST_KEYS.load(Ordering::Relaxed),
    };
    let modifier = mod_index(vk);
    if let Some(i) = modifier {
        if !from_program {
            if down {
                ctx.phys |= 1 << i;
            } else {
                ctx.phys &= !(1 << i);
            }
        }
    }
    let logging = diag_on();

    // Training: while a round runs and Cat Key is in front, keys go to the trainer only.
    if !from_program {
        let i = vk as usize;
        if !down && ctx.cap_down[i] {
            ctx.cap_down[i] = false;
            if !ctx.engine.is_recording() {
                return true;
            }
        } else if down && modifier.is_none() && CAPTURE.load(Ordering::Relaxed) && own_app_in_front() {
            ctx.cap_down[i] = true;
            if !ctx.engine.is_recording() {
                if logging {
                    diag_line(format!("{}↓  kept for the trainer", key_name(vk)));
                }
                return true;
            }
        }
    }

    let swallow = ctx.engine.handle(ev, &mut ctx.env);
    if !from_program {
        count_tracked(ctx, &ev);
        if down && modifier.is_none() && (ctx.phys & 0b1111_0011) != 0 {
            let app = ctx.env.foreground_exe().to_string();
            crate::coach::count_combo(&app);
        }
    }
    let sent = fmt_outs(&ctx.engine.out);
    ctx.send_out();

    if !swallow {
        // A modifier that goes through changes what apps see as held.
        if let Some(i) = modifier {
            if down {
                ctx.shown |= 1 << i;
            } else {
                ctx.shown &= !(1 << i);
            }
        }
        // Nothing replaced right now: what's on the keyboard is what apps see.
        if ctx.engine.held_mods() == 0 && ctx.engine.out.is_empty() && modifier.is_none() && sent.is_empty() {
            ctx.shown = ctx.phys;
        }
        let want = bits_to_flags(ctx.shown);
        if flags & MOD_FLAGS != want {
            unsafe { CGEventSetFlags(event, (flags & !MOD_FLAGS) | want) };
        }
    } else if let Some(i) = modifier {
        // A modifier kept from apps: they don't see it held.
        if down {
            ctx.shown &= !(1 << i);
        }
    }
    if unsafe { CGEventGetIntegerValueField(event, FIELD_AUTOREPEAT) } != 0 && swallow && logging {
        diag_line(format!("{} repeat", key_name(vk)));
    }
    if logging {
        let app = ctx.env.foreground_exe().to_string();
        diag_line(format!(
            "{}{}  from {}  → {}{}  [{}]",
            key_name(vk),
            if down { "↓" } else { "↑" },
            if from_program { "another program" } else { "keyboard" },
            if swallow { "swallowed" } else { "passed" },
            if sent.is_empty() { String::new() } else { format!("  then send: {sent}") },
            if app.is_empty() { "?" } else { app.as_str() },
        ));
    }
    ctx.flush_effects();
    swallow
}

/// A bug in the engine must never leave keys stuck: reset and carry on.
fn recover() {
    crate::log("keyboard engine panicked; state reset");
    if let Ok(mut g) = CTX.lock() {
        if let Some(ctx) = g.as_mut() {
            ctx.engine.hard_reset();
            ctx.send_out();
            ctx.shown = ctx.phys;
        }
    }
}

// ---------------------------------------------------------------- starting

/// Ask for the Accessibility permission (shows the system prompt once).
fn ask_permission() -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    unsafe {
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt as _);
        let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
        let ok = AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const c_void);
        let _ = CGRequestListenEventAccess();
        let _ = CGRequestPostEventAccess();
        ok
    }
}

pub fn start(compiled: Arc<Compiled>, enabled: bool, effects: Sender<Effect>) {
    T0.get_or_init(Instant::now);
    let phys = flags_to_bits(unsafe { CGEventSourceFlagsState(STATE_HID) });
    let source = unsafe {
        let s = CGEventSourceCreate(STATE_PRIVATE);
        if !s.is_null() {
            // Never hold back the real keyboard after a key we send.
            CGEventSourceSetLocalEventsSuppressionInterval(s, 0.0);
        }
        s
    } as usize;
    if let Ok(mut g) = CTX.lock() {
        *g = Some(Ctx {
            engine: Engine::new(compiled, enabled),
            env: MacEnv { bundle: String::new(), pid: 0, checked: None },
            effects,
            phys,
            shown: phys,
            track_down: [false; 256],
            cap_down: [false; 256],
            source,
            surrogate: 0,
        });
    }
    std::thread::Builder::new()
        .name("catkey-keyboard".into())
        .spawn(run)
        .expect("start keyboard thread");
}

fn run() {
    let mut asked = false;
    loop {
        if crate::win::keyboard_allowed() {
            break;
        }
        if !asked {
            asked = true;
            ask_permission();
            crate::log("waiting for the Accessibility permission");
        }
        std::thread::sleep(Duration::from_millis(1500));
    }
    let mask: u64 = (1 << KEY_DOWN) | (1 << KEY_UP) | (1 << FLAGS_CHANGED);
    let tap = loop {
        let tap = unsafe { CGEventTapCreate(SESSION_TAP, HEAD_INSERT, TAP_DEFAULT, mask, tap_callback, null_mut()) };
        if !tap.is_null() {
            break tap;
        }
        crate::log("could not start the keyboard tap; trying again");
        std::thread::sleep(Duration::from_secs(3));
    };
    unsafe {
        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        let rl = CFRunLoopGetCurrent();
        CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        TAP.store(tap, Ordering::SeqCst);
        RUN_LOOP.store(rl, Ordering::SeqCst);
        RUNNING.store(true, Ordering::SeqCst);
        crate::log("keyboard tap running");
        CFRunLoopRun();
        RUNNING.store(false, Ordering::SeqCst);
        CGEventTapEnable(tap, false);
        CFRelease(source as *const c_void);
        CFRelease(tap as *const c_void);
        TAP.store(null_mut(), Ordering::SeqCst);
    }
}

pub fn engine_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

/// Send a command to the keyboard engine (safe from any thread).
pub fn send(cmd: Cmd) {
    let mut quit = false;
    if let Ok(mut g) = CTX.lock() {
        if let Some(ctx) = g.as_mut() {
            match cmd {
                Cmd::Config(c) => ctx.engine.set_compiled(c),
                Cmd::Enabled(on) => {
                    ctx.engine.set_enabled(on);
                    ctx.send_out();
                }
                Cmd::StartRecord => ctx.engine.start_recording(now_ms()),
                Cmd::StopRecord => ctx.engine.stop_recording(),
                Cmd::Quit => {
                    ctx.engine.set_enabled(false);
                    ctx.send_out();
                    quit = true;
                }
            }
            ctx.flush_effects();
        }
    }
    if quit {
        let rl = RUN_LOOP.load(Ordering::SeqCst);
        if !rl.is_null() {
            unsafe { CFRunLoopStop(rl) };
        }
    }
}

pub fn set_capture(on: bool) {
    CAPTURE.store(on, Ordering::SeqCst);
}

pub fn set_test_keys(on: bool) {
    TEST_KEYS.store(on, Ordering::SeqCst);
}

// ---------------------------------------------------------------- shortcuts being learned

pub struct Tracked {
    ids: Vec<String>,
    by_vk: Vec<Vec<(catkey_core::keys::ModSpec, Vec<String>, u32)>>,
}
static TRACKED: Mutex<Option<Arc<Tracked>>> = Mutex::new(None);
static TRACK_COUNTS: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

pub fn set_tracked(list: Vec<(String, catkey_core::keys::Shortcut, Vec<String>)>) {
    let mut t = Tracked { ids: Vec::with_capacity(list.len()), by_vk: vec![Vec::new(); 256] };
    for (i, (id, sc, apps)) in list.into_iter().enumerate() {
        t.ids.push(id);
        let apps = apps.into_iter().map(|a| a.to_ascii_lowercase()).collect();
        t.by_vk[sc.key as usize].push((sc.mods, apps, i as u32));
    }
    if let Ok(mut g) = TRACKED.lock() {
        *g = if t.ids.is_empty() { None } else { Some(Arc::new(t)) };
    }
    if let Ok(mut c) = TRACK_COUNTS.lock() {
        c.clear();
    }
}

pub fn take_tracked_counts() -> std::collections::HashMap<String, u32> {
    let counts: Vec<(u32, u32)> = TRACK_COUNTS.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default();
    let Some(t) = TRACKED.lock().ok().and_then(|g| g.clone()) else { return Default::default() };
    let mut out = std::collections::HashMap::new();
    for (i, n) in counts {
        if let Some(id) = t.ids.get(i as usize) {
            *out.entry(id.clone()).or_insert(0) += n;
        }
    }
    out
}

fn count_tracked(ctx: &mut Ctx, ev: &KeyEvent) {
    let vk = ev.vk as usize;
    let fresh = ev.down && !ctx.track_down[vk];
    ctx.track_down[vk] = ev.down;
    if !fresh || ev.injected || catkey_core::keys::is_modifier(ev.vk) || ctx.engine.is_recording() {
        return;
    }
    let Some(t) = TRACKED.try_lock().ok().and_then(|g| g.clone()) else { return };
    let list = &t.by_vk[vk];
    if list.is_empty() {
        return;
    }
    let mods = ctx.engine.held_mods();
    let mut exe: Option<String> = None;
    for (spec, apps, idx) in list {
        if !spec.matches(mods) {
            continue;
        }
        if !apps.is_empty() {
            let e = exe.get_or_insert_with(|| ctx.env.foreground_exe().to_string());
            if !apps.iter().any(|a| catkey_core::config::app_matches(a, e)) {
                continue;
            }
        }
        if let Ok(mut c) = TRACK_COUNTS.try_lock() {
            match c.iter_mut().find(|(i, _)| i == idx) {
                Some((_, n)) => *n += 1,
                None => c.push((*idx, 1)),
            }
        }
    }
}

// ---------------------------------------------------------------- key log

static DIAG_ON: AtomicBool = AtomicBool::new(false);
static DIAG_T0: AtomicU32 = AtomicU32::new(0);
static DIAG: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
const DIAG_MAX_LINES: usize = 3000;
const DIAG_MAX_MS: u32 = 10 * 60 * 1000;

pub fn diag_set(on: bool) {
    if on {
        if let Ok(mut q) = DIAG.lock() {
            q.clear();
        }
        DIAG_T0.store(now_ms(), Ordering::SeqCst);
        diag_line(format!(
            "key log started · Cat Key {} · keyboard tap {} · Accessibility {}",
            env!("CARGO_PKG_VERSION"),
            if engine_running() { "on" } else { "OFF" },
            if crate::win::keyboard_allowed() { "allowed" } else { "NOT allowed" },
        ));
    } else if DIAG_ON.load(Ordering::SeqCst) {
        diag_line("key log stopped".into());
    }
    DIAG_ON.store(on, Ordering::SeqCst);
}

pub fn diag_take() -> (bool, Vec<String>) {
    let lines = DIAG.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default();
    (DIAG_ON.load(Ordering::SeqCst), lines)
}

fn diag_on() -> bool {
    if !DIAG_ON.load(Ordering::Relaxed) {
        return false;
    }
    if now_ms().wrapping_sub(DIAG_T0.load(Ordering::Relaxed)) > DIAG_MAX_MS {
        diag_set(false);
        return false;
    }
    true
}

fn diag_line(text: String) {
    let t = now_ms().wrapping_sub(DIAG_T0.load(Ordering::Relaxed));
    if let Ok(mut q) = DIAG.lock() {
        if q.len() >= DIAG_MAX_LINES {
            q.pop_front();
        }
        q.push_back(format!("{:>7.3}s  {}", t as f64 / 1000.0, text));
    }
}

fn fmt_outs(outs: &[Out]) -> String {
    let arrow = |d: bool| if d { "↓" } else { "↑" };
    outs.iter()
        .map(|o| match *o {
            Out::Key { vk, down } => format!("{}{}", key_name(vk), arrow(down)),
            Out::Raw { vk, down, .. } => format!("{}{}(again)", key_name(vk), arrow(down)),
            Out::Unicode { down: true, .. } => "text".to_string(),
            Out::Unicode { .. } | Out::Dummy => String::new(),
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_codes_round_trip() {
        for vk in [b'A', b'Z', b'0', b'9', 0x0D, 0x09, 0x20, 0x08, 0x1B, 0x25, 0x28, 0x70, 0x7B, 0xBA, 0xDE, 0xA2, 0x5B, 0xA5] {
            let code = vk_to_mac(vk).expect("has a Mac key");
            assert_eq!(mac_to_vk(code), vk, "vk {vk:#x}");
        }
        assert_eq!(vk_to_mac(0x0D), Some(0x24)); // Return, not keypad Enter
    }
}
