//! The low-level keyboard hook.
//!
//! One dedicated, high-priority thread owns the hook and the [`Engine`]. The hook
//! callback never blocks, never allocates in the common case and never waits on
//! the UI: config changes arrive as thread messages, and everything the UI needs
//! to hear about leaves through a channel.

use catkey_core::engine::{Effect, Engine, Env, KeyEvent, Out};
use catkey_core::keys::DUMMY_VK;
use catkey_core::Compiled;
use std::cell::RefCell;
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
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
    HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WINEVENT_OUTOFCONTEXT, WM_APP,
    WM_KEYDOWN, WM_SYSKEYDOWN, WM_TIMER, WM_USER,
};

/// `dwExtraInfo` on every event we inject, so the hook lets them through.
pub const MARK: usize = 0x0CA7_CA7E;

const WM_CATKEY: u32 = WM_APP + 0x0CA;
/// Re-install the hook every so often: keeps us first in line and recovers if
/// Windows ever drops a hook that was slow (for example while the PC was busy).
const REFRESH_MS: u32 = 60_000;

pub enum Cmd {
    Config(Arc<Compiled>),
    Enabled(bool),
    StartRecord,
    StopRecord,
    Quit,
}

static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static HOOK_OK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    inputs: Vec<INPUT>,
    /// (time, vk, flags, scan) of the last event, to ignore an exact duplicate.
    last: (u32, u32, u32, u32),
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
            inputs: Vec::with_capacity(32),
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
            WM_CATKEY => drain_commands(),
            WM_TIMER => refresh_hook(),
            _ => {
                DispatchMessageW(&msg);
            }
        }
    }
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
                }
            }
        }
    });
}

unsafe fn drain_commands() {
    let cmds: Vec<Cmd> = match PENDING.lock() {
        Ok(mut q) => std::mem::take(&mut *q),
        Err(_) => return,
    };
    if cmds.is_empty() {
        return;
    }
    CTX.with(|c| {
        let Ok(mut g) = c.try_borrow_mut() else { return };
        let Some(ctx) = g.as_mut() else { return };
        for cmd in cmds {
            match cmd {
                Cmd::Config(compiled) => ctx.engine.set_compiled(compiled),
                Cmd::Enabled(on) => {
                    ctx.engine.set_enabled(on);
                    send_outputs(&mut ctx.inputs, &ctx.engine.out);
                }
                Cmd::StartRecord => ctx.engine.start_recording(GetTickCount()),
                Cmd::StopRecord => ctx.engine.stop_recording(),
                Cmd::Quit => {
                    ctx.engine.set_enabled(false);
                    send_outputs(&mut ctx.inputs, &ctx.engine.out);
                    PostQuitMessage(0);
                }
            }
        }
    });
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
                send_outputs(&mut ctx.inputs, &ctx.engine.out);
            }
        }
    });
    refresh_hook();
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && lparam != 0 {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
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
        };
        let swallow = ctx.engine.handle(ev, &mut ctx.env);
        if !ctx.engine.out.is_empty() {
            send_outputs(&mut ctx.inputs, &ctx.engine.out);
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
                send_outputs(&mut ctx.inputs, &ctx.engine.out);
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

unsafe fn send_outputs(buf: &mut Vec<INPUT>, outs: &[Out]) {
    if outs.is_empty() {
        return;
    }
    buf.clear();
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
    SendInput(buf.len() as u32, buf.as_ptr(), size_of::<INPUT>() as i32);
}
