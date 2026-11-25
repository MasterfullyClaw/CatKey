//! The low-level keyboard hook.
//!
//! One dedicated, high-priority thread owns the hook and the [`Engine`]. The hook
//! callback only decides "swallow or pass" and returns at once: it never blocks and
//! never waits on the UI. The keys to send instead are queued and sent by the same
//! thread right after the callback returns.
//!
//! Why not send inside the callback? `SendInput` runs every other program's keyboard
//! hook (PowerToys, overlays, macro tools) before it returns. If one of them is slow,
//! Windows gives up waiting for *our* callback, lets the original key through (Alt+W
//! reaches the app: the "ding") and may even drop the hook. Sending afterwards keeps
//! the callback to microseconds.

use catkey_core::engine::{Effect, Engine, Env, KeyEvent, Out};
use catkey_core::keys::DUMMY_VK;
use catkey_core::Compiled;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
};
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, MAPVK_VK_TO_VSC_EX,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId, PeekMessageW,
    PostQuitMessage, PostThreadMessageW, SetTimer, SetWindowsHookExW, UnhookWindowsHookEx, EVENT_SYSTEM_DESKTOPSWITCH,
    HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WINEVENT_OUTOFCONTEXT, WM_APP,
    WM_KEYDOWN, WM_SYSKEYDOWN, WM_TIMER, WM_USER,
};

/// `dwExtraInfo` on every event we inject, so the hook lets them through.
pub const MARK: usize = 0x0CA7_CA7E;

const WM_CATKEY: u32 = WM_APP + 0x0CA;
/// Posted to ourselves: send the queued keys now that the callback has returned.
const WM_CATKEY_SEND: u32 = WM_APP + 0x0CB;
/// Re-install the hook every so often: keeps us first in line and recovers if
/// Windows ever drops a hook (for example after the PC was very busy).
const REFRESH_MS: u32 = 20_000;
/// A send that takes longer than this is written to the log (another program's
/// keyboard hook is slowing every key down).
const SLOW_SEND_MS: u32 = 150;

pub enum Cmd {
    Config(Arc<Compiled>),
    Enabled(bool),
    StartRecord,
    StopRecord,
    Quit,
}

static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static HOOK_OK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
// ---------------------------------------------------------------- key log
// "Settings → Try a shortcut → Key log": every key Cat Key sees, what it decided and
// what it sent. Off unless the window turns it on; stops by itself after 10 minutes.

static DIAG_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static DIAG_T0: AtomicU32 = AtomicU32::new(0);
static DIAG: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
const DIAG_MAX_LINES: usize = 3000;
const DIAG_MAX_MS: u32 = 10 * 60 * 1000;

pub fn diag_set(on: bool) {
    if on {
        if let Ok(mut q) = DIAG.lock() {
            q.clear();
        }
        DIAG_T0.store(unsafe { GetTickCount() }, Ordering::SeqCst);
        diag_line(format!("key log started · Cat Key {} · hook {}", env!("CARGO_PKG_VERSION"), if engine_running() { "on" } else { "OFF" }));
    } else if DIAG_ON.load(Ordering::SeqCst) {
        diag_line("key log stopped".into());
    }
    DIAG_ON.store(on, Ordering::SeqCst);
}

/// Lines logged since the last call.
pub fn diag_take() -> (bool, Vec<String>) {
    let lines = DIAG.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default();
    (DIAG_ON.load(Ordering::SeqCst), lines)
}

fn diag_on() -> bool {
    if !DIAG_ON.load(Ordering::Relaxed) {
        return false;
    }
    let age = unsafe { GetTickCount() }.wrapping_sub(DIAG_T0.load(Ordering::Relaxed));
    if age > DIAG_MAX_MS {
        diag_set(false);
        return false;
    }
    true
}

fn diag_line(text: String) {
    let t = unsafe { GetTickCount() }.wrapping_sub(DIAG_T0.load(Ordering::Relaxed));
    if let Ok(mut q) = DIAG.lock() {
        if q.len() >= DIAG_MAX_LINES {
            q.pop_front();
        }
        q.push_back(format!("{:>7.3}s  {}", t as f64 / 1000.0, text));
    }
}

fn fmt_outs(outs: &[Out]) -> String {
    use catkey_core::keys::key_name;
    let arrow = |d: bool| if d { "↓" } else { "↑" };
    outs.iter()
        .map(|o| match *o {
            Out::Key { vk, down } => format!("{}{}", key_name(vk), arrow(down)),
            Out::Raw { vk, down, .. } => format!("{}{}(again)", key_name(vk), arrow(down)),
            Out::Unicode { down: true, .. } => "text".to_string(),
            Out::Unicode { .. } => String::new(),
            Out::Dummy => "mask".to_string(),
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The modifiers Windows reports as held right now.
fn held_now() -> String {
    const MODS: [(u8, &str); 8] = [
        (0xA2, "LCtrl"), (0xA3, "RCtrl"), (0xA0, "LShift"), (0xA1, "RShift"),
        (0xA4, "LAlt"), (0xA5, "RAlt"), (0x5B, "LWin"), (0x5C, "RWin"),
    ];
    let held: Vec<&str> = MODS
        .iter()
        .filter(|(vk, _)| unsafe { (GetAsyncKeyState(*vk as i32) as u16) & 0x8000 != 0 })
        .map(|(_, n)| *n)
        .collect();
    if held.is_empty() { "-".into() } else { held.join("+") }
}

/// A WM_CATKEY_SEND is already on its way.
static SEND_POSTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `SendInput` is running right now (our callback can run again meanwhile).
static SENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LAST_SLOW_LOG: AtomicU32 = AtomicU32::new(0);

/// True while the keyboard hook is installed and its thread is running.
pub fn engine_running() -> bool {
    HOOK_OK.load(Ordering::SeqCst) && THREAD_ID.load(Ordering::SeqCst) != 0
}
static PENDING: Mutex<Vec<Cmd>> = Mutex::new(Vec::new());

struct Ctx {
    engine: Engine,
    env: WinEnv,
    hook: HHOOK,
    effects: Sender<Effect>,
    /// Keys to send once the hook callback has returned (in order).
    pending: Vec<Out>,
    /// (time, vk, flags, scan) of the last event, to ignore an exact duplicate.
    last: (u32, u32, u32, u32),
}

impl Ctx {
    /// Queue what the engine just asked for and make sure a send is on its way.
    fn queue_out(&mut self) {
        if self.engine.out.is_empty() {
            return;
        }
        self.pending.extend_from_slice(&self.engine.out);
        self.engine.out.clear();
        self.post_send();
    }

    fn post_send(&self) {
        if !SEND_POSTED.swap(true, Ordering::SeqCst) {
            unsafe {
                PostThreadMessageW(GetCurrentThreadId(), WM_CATKEY_SEND, 0, 0);
            }
        }
    }

    /// Keys we queued haven't all gone out yet.
    fn sending(&self) -> bool {
        !self.pending.is_empty() || SENDING.load(Ordering::SeqCst)
    }
}

thread_local! {
    static CTX: RefCell<Option<Ctx>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct WinEnv {
    hwnd: usize,
    pid: u32,
    exe: String,
}

impl Env for WinEnv {
    fn foreground_exe(&mut self) -> &str {
        unsafe {
            let hwnd = GetForegroundWindow();
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if hwnd as usize != self.hwnd || pid != self.pid {
                self.hwnd = hwnd as usize;
                self.pid = pid;
                self.exe = crate::win::window_exe(hwnd, pid).unwrap_or_default();
            }
        }
        &self.exe
    }

    fn key_down_async(&mut self, vk: u8) -> bool {
        unsafe { (GetAsyncKeyState(vk as i32) as u16) & 0x8000 != 0 }
    }
}

/// Send a command to the hook thread (safe from any thread).
pub fn send(cmd: Cmd) {
    if let Ok(mut q) = PENDING.lock() {
        q.push(cmd);
    }
    let tid = THREAD_ID.load(Ordering::SeqCst);
    if tid != 0 {
        unsafe {
            PostThreadMessageW(tid, WM_CATKEY, 0, 0);
        }
    }
}

pub fn start(compiled: Arc<Compiled>, enabled: bool, effects: Sender<Effect>) {
    std::thread::Builder::new()
        .name("catkey-keyboard".into())
        .spawn(move || unsafe { run(compiled, enabled, effects) })
        .expect("start keyboard thread");
}

unsafe fn install() -> HHOOK {
    SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), GetModuleHandleW(null_mut()), 0)
}

unsafe fn run(compiled: Arc<Compiled>, enabled: bool, effects: Sender<Effect>) {
    let mut msg: MSG = zeroed();
    // Make sure this thread has a message queue before anyone posts to it.
    PeekMessageW(&mut msg, null_mut(), WM_USER, WM_USER, PM_NOREMOVE);
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
    crate::win::opt_out_of_power_throttling_for_thread();

    let hook = install();
    if hook.is_null() {
        crate::log("could not install the keyboard hook");
    }
    HOOK_OK.store(!hook.is_null(), Ordering::SeqCst);
    CTX.with(|c| {
        *c.borrow_mut() = Some(Ctx {
            engine: Engine::new(compiled, enabled),
            env: WinEnv::default(),
            hook,
            effects,
            pending: Vec::with_capacity(32),
            last: (0, 0, 0, 0),
        })
    });
    THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);
    SetWinEventHook(
        EVENT_SYSTEM_DESKTOPSWITCH,
        EVENT_SYSTEM_DESKTOPSWITCH,
        null_mut(),
        Some(on_desktop_switch),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
    );
    SetTimer(null_mut(), 0, REFRESH_MS, None);
    drain_commands();

    loop {
        let r = GetMessageW(&mut msg, null_mut(), 0, 0);
        if r == 0 || r == -1 {
            break;
        }
        match msg.message {
            WM_CATKEY_SEND => flush(),
            WM_CATKEY => drain_commands(),
            WM_TIMER => refresh_hook(),
            _ => {
                DispatchMessageW(&msg);
            }
        }
    }
    flush();
    CTX.with(|c| {
        if let Some(ctx) = c.borrow_mut().as_mut() {
            if !ctx.hook.is_null() {
                UnhookWindowsHookEx(ctx.hook);
                ctx.hook = null_mut();
            }
        }
    });
    HOOK_OK.store(false, Ordering::SeqCst);
    THREAD_ID.store(0, Ordering::SeqCst);
}

unsafe fn refresh_hook() {
    CTX.with(|c| {
        if let Ok(mut g) = c.try_borrow_mut() {
            if let Some(ctx) = g.as_mut() {
                let fresh = install();
                if !fresh.is_null() {
                    if !ctx.hook.is_null() {
                        UnhookWindowsHookEx(ctx.hook);
                    }
                    ctx.hook = fresh;
                    HOOK_OK.store(true, Ordering::SeqCst);
                } else if diag_on() {
                    diag_line(format!("could not re-install the keyboard hook (error {})", GetLastError()));
                }
            }
        }
    });
}

/// Send the queued keys. Never called inside the hook callback, and never while
/// `CTX` is borrowed: `SendInput` can run our own callback again for new key presses.
unsafe fn flush() {
    let outs = CTX.with(|c| match c.try_borrow_mut() {
        Ok(mut g) => match g.as_mut() {
            Some(ctx) => {
                SEND_POSTED.store(false, Ordering::SeqCst);
                std::mem::replace(&mut ctx.pending, Vec::with_capacity(32))
            }
            None => Vec::new(),
        },
        // Busy right now (shouldn't happen here): try again in a moment.
        Err(_) => {
            PostThreadMessageW(GetCurrentThreadId(), WM_CATKEY_SEND, 0, 0);
            Vec::new()
        }
    });
    if outs.is_empty() {
        return;
    }
    let started = GetTickCount();
    SENDING.store(true, Ordering::SeqCst);
    let (asked, sent, err) = send_outputs(&outs);
    SENDING.store(false, Ordering::SeqCst);
    let took = GetTickCount().wrapping_sub(started);
    if sent < asked {
        let last = LAST_SLOW_LOG.load(Ordering::Relaxed);
        if last == 0 || started.wrapping_sub(last) > 60_000 {
            LAST_SLOW_LOG.store(started, Ordering::Relaxed);
            crate::log(&format!("Windows accepted only {sent} of {asked} keys (error {err}); an app running as administrator?"));
        }
    }
    if diag_on() {
        diag_line(format!(
            "   sent {sent}/{asked} in {took} ms{}",
            if sent < asked { format!("  ← Windows refused the rest (error {err})") } else { String::new() }
        ));
    }
    if took > SLOW_SEND_MS {
        let last = LAST_SLOW_LOG.load(Ordering::Relaxed);
        if last == 0 || started.wrapping_sub(last) > 60_000 {
            LAST_SLOW_LOG.store(started, Ordering::Relaxed);
            crate::log(&format!("sending {} keys took {took} ms (another program's keyboard hook is slow)", outs.len()));
        }
    }
}

unsafe fn drain_commands() {
    let cmds: Vec<Cmd> = match PENDING.lock() {
        Ok(mut q) => std::mem::take(&mut *q),
        Err(_) => return,
    };
    if cmds.is_empty() {
        return;
    }
    let mut quit = false;
    CTX.with(|c| {
        let Ok(mut g) = c.try_borrow_mut() else { return };
        let Some(ctx) = g.as_mut() else { return };
        for cmd in cmds {
            match cmd {
                Cmd::Config(compiled) => ctx.engine.set_compiled(compiled),
                Cmd::Enabled(on) => {
                    ctx.engine.set_enabled(on);
                    ctx.queue_out();
                }
                Cmd::StartRecord => ctx.engine.start_recording(GetTickCount()),
                Cmd::StopRecord => ctx.engine.stop_recording(),
                Cmd::Quit => {
                    ctx.engine.set_enabled(false);
                    ctx.queue_out();
                    quit = true;
                }
            }
        }
    });
    if quit {
        flush(); // let go of anything we hold before the thread ends
        PostQuitMessage(0);
    }
}

/// Lock screen, UAC prompt, Ctrl+Alt+Del: we won't see the key releases that
/// happen there, so start clean (and let go of anything we were holding).
unsafe extern "system" fn on_desktop_switch(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _obj: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    CTX.with(|c| {
        if let Ok(mut g) = c.try_borrow_mut() {
            if let Some(ctx) = g.as_mut() {
                ctx.engine.hard_reset();
                ctx.queue_out();
            }
        }
    });
    refresh_hook();
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && lparam != 0 {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
        if kb.dwExtraInfo == MARK && diag_on() {
            let down = wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN;
            diag_line(format!("      ↳ ours: {}{}", catkey_core::keys::key_name(kb.vkCode as u8), if down { "↓" } else { "↑" }));
        }
        if kb.dwExtraInfo != MARK && kb.vkCode < 256 {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handle(kb, wparam)));
            match r {
                Ok(true) => return 1,
                Ok(false) => {}
                Err(_) => recover(),
            }
        }
    }
    CallNextHookEx(null_mut(), code, wparam, lparam)
}

unsafe fn handle(kb: &KBDLLHOOKSTRUCT, wparam: WPARAM) -> bool {
    CTX.with(|c| {
        let Ok(mut g) = c.try_borrow_mut() else { return false };
        let Some(ctx) = g.as_mut() else { return false };
        let key = (kb.time, kb.vkCode, kb.flags, kb.scanCode);
        if key == ctx.last {
            return false; // the same event twice (hook refresh race): handled already
        }
        ctx.last = key;
        let down = wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN;
        let ev = KeyEvent {
            vk: kb.vkCode as u8,
            scan: kb.scanCode as u16,
            down,
            extended: kb.flags & LLKHF_EXTENDED != 0,
            time: kb.time,
            injected: kb.flags & LLKHF_INJECTED != 0,
        };
        let logging = diag_on();
        let held = if logging { held_now() } else { String::new() };
        let queued_before = ctx.pending.len();
        let mut swallow = ctx.engine.handle(ev, &mut ctx.env);
        let mut deferred = false;
        if !swallow && !ev.injected && ctx.sending() {
            deferred = true;
            // A real key arrived while our keys are still on their way: send it after
            // them, so it can't overtake (e.g. land before the Ctrl we are releasing).
            // Keys from other programs keep their own markers, so those always pass.
            ctx.pending.push(Out::Raw { vk: ev.vk, scan: ev.scan, extended: ev.extended, down: ev.down });
            ctx.post_send();
            swallow = true;
        }
        ctx.queue_out();
        if logging {
            let src = if ev.injected { format!("typed by another program (tag {:#x})", kb.dwExtraInfo) } else { "keyboard".to_string() };
            let app = ctx.env.foreground_exe().to_string();
            let what = if deferred { "held back (keys still going out)" } else if swallow { "swallowed" } else { "passed" };
            let sends = fmt_outs(&ctx.pending[queued_before.min(ctx.pending.len())..]);
            diag_line(format!(
                "{}{}  from {}  held: {}  → {}{}  [{}]",
                catkey_core::keys::key_name(ev.vk),
                if ev.down { "↓" } else { "↑" },
                src,
                held,
                what,
                if sends.is_empty() { String::new() } else { format!("  then send: {sends}") },
                if app.is_empty() { "?" } else { app.as_str() },
            ));
        }
        for e in ctx.engine.effects.drain(..) {
            let _ = ctx.effects.send(e);
        }
        swallow
    })
}

/// A bug in the engine must never leave keys stuck: reset and carry on.
unsafe fn recover() {
    crate::log("keyboard engine panicked; state reset");
    CTX.with(|c| {
        if let Ok(mut g) = c.try_borrow_mut() {
            if let Some(ctx) = g.as_mut() {
                ctx.engine.hard_reset();
                ctx.queue_out();
            }
        }
    });
}

fn is_extended(vk: u8) -> bool {
    matches!(
        vk,
        0x03 | 0x21..=0x28 | 0x2C | 0x2D | 0x2E | 0x5B | 0x5C | 0x5D | 0x6F | 0x90 | 0xA3 | 0xA5 | 0xA6..=0xB7
    )
}

unsafe fn key_input(vk: u8, scan: Option<u16>, extended: Option<bool>, down: bool) -> INPUT {
    let sc = scan.unwrap_or_else(|| MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) as u16);
    let ext = extended.unwrap_or_else(|| (sc & 0xFF00) == 0xE000 || is_extended(vk));
    let mut flags = if down { 0 } else { KEYEVENTF_KEYUP };
    if ext {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk as u16, wScan: sc & 0xFF, dwFlags: flags, time: 0, dwExtraInfo: MARK },
        },
    }
}

/// Returns (events asked, events Windows accepted, last error).
unsafe fn send_outputs(outs: &[Out]) -> (u32, u32, u32) {
    if outs.is_empty() {
        return (0, 0, 0);
    }
    let mut buf: Vec<INPUT> = Vec::with_capacity(outs.len() + 2);
    for o in outs {
        match *o {
            Out::Key { vk, down } => buf.push(key_input(vk, None, None, down)),
            Out::Raw { vk, scan, extended, down } => buf.push(key_input(vk, Some(scan), Some(extended), down)),
            Out::Unicode { unit, down } => buf.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: 0,
                        wScan: unit,
                        dwFlags: KEYEVENTF_UNICODE | if down { 0 } else { KEYEVENTF_KEYUP },
                        time: 0,
                        dwExtraInfo: MARK,
                    },
                },
            }),
            Out::Dummy => {
                buf.push(key_input(DUMMY_VK, Some(0), Some(false), true));
                buf.push(key_input(DUMMY_VK, Some(0), Some(false), false));
            }
        }
    }
    let asked = buf.len() as u32;
    let sent = SendInput(asked, buf.as_ptr(), size_of::<INPUT>() as i32);
    let err = if sent < asked { GetLastError() } else { 0 };
    (asked, sent, err)
}
