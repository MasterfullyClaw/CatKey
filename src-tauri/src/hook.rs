//! The low-level keyboard hook.
//!
//! Three parts, so that a slow program elsewhere can never make Cat Key miss a key:
//!
//! * The **hook thread** owns the hook and the [`Engine`]. Its callback only decides
//!   "swallow or pass" and queues the keys to send. It never sends and never waits.
//!   Windows skips a hook that answers late (the original Alt+W then reaches the app:
//!   the "ding") and silently removes it if that keeps happening.
//! * The **sender thread** sends the queued keys. `SendInput` can wait on every other
//!   program's keyboard hook (overlays, dictation and macro tools); only this thread
//!   waits, so the hook thread is always free to answer the next key at once.
//! * A **watchdog** on the hook thread gets every real key straight from the keyboard
//!   (raw input). If the hook missed a key release, the watchdog finishes that press,
//!   so the next press works the first time. If the hook stopped getting keys, the
//!   watchdog installs it again at once (not 20 seconds later).

use catkey_core::engine::{Effect, Engine, Env, KeyEvent, Out};
use catkey_core::keys::{key_name, DUMMY_VK};
use catkey_core::Compiled;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

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
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK,
    RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetForegroundWindow, GetGUIThreadInfo, GetMessageW,
    GetWindowThreadProcessId, KillTimer, PeekMessageW, PostQuitMessage, PostThreadMessageW, RegisterClassW, SetTimer,
    SetWindowsHookExW, UnhookWindowsHookEx, EVENT_SYSTEM_DESKTOPSWITCH, HC_ACTION, HHOOK, HWND_MESSAGE, KBDLLHOOKSTRUCT,
    LLKHF_EXTENDED, LLKHF_INJECTED, MSG, PM_NOREMOVE, RI_KEY_BREAK, RI_KEY_E0, RI_KEY_E1, WH_KEYBOARD_LL,
    GUITHREADINFO, WINEVENT_OUTOFCONTEXT, WM_APP, WM_INPUT, WM_KEYDOWN, WM_SYSKEYDOWN, WM_TIMER, WM_USER, WNDCLASSW,
};

/// `dwExtraInfo` on every event we inject, so the hook lets them through.
pub const MARK: usize = 0x0CA7_CA7E;

const WM_CATKEY: u32 = WM_APP + 0x0CA;
/// Re-install the hook every so often: keeps us first in line and recovers if
/// Windows ever drops a hook (for example after the PC was very busy).
const REFRESH_MS: u32 = 20_000;
const TIMER_REFRESH: usize = 1;
const TIMER_WATCH: usize = 2;
/// How long the hook may take to see a key the keyboard already reported. Windows
/// waits at most one second for a hook (LowLevelHooksTimeout) before skipping it.
const WATCH_GRACE_MS: u32 = 1_500;
/// A send that takes longer than this is written to the log (another program's
/// keyboard hook is slowing every key down).
const SLOW_SEND_MS: u32 = 150;
/// If queued keys haven't gone out after this long, stop holding real keys back
/// behind them (something is badly stuck; typing must keep working).
const STUCK_SEND_MS: u32 = 2_000;

pub enum Cmd {
    Config(Arc<Compiled>),
    Enabled(bool),
    StartRecord,
    StopRecord,
    Quit,
}

static THREAD_ID: AtomicU32 = AtomicU32::new(0);
/// Training: while a round runs and Cat Key's window is in front, every real key goes
/// to the trainer only. Nothing reaches Windows or another app (no Alt+F4 by accident).
static CAPTURE: AtomicBool = AtomicBool::new(false);

pub fn set_capture(on: bool) {
    CAPTURE.store(on, Ordering::SeqCst);
}

fn own_window_in_front() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, &mut pid);
        pid == windows_sys::Win32::System::Threading::GetCurrentProcessId()
    }
}
static HOOK_OK: AtomicBool = AtomicBool::new(false);
/// Treat keys typed by other programs as real key presses (only with `--test-keys`,
/// so automated tests can drive Cat Key).
static TEST_KEYS: AtomicBool = AtomicBool::new(false);
/// Keys the watchdog caught that the hook missed, and hook re-installs it caused.
pub static WATCH_FIXES: AtomicU32 = AtomicU32::new(0);
pub static WATCH_REINSTALLS: AtomicU32 = AtomicU32::new(0);

/// Shortcuts you're learning, counted when you use them for real (Train yourself).
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

/// A real key press: if it's a shortcut being learned, in its app, count it.
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

/// For the shortcut coach, on real presses only: count shortcuts used in each app
/// (Ctrl, Alt or Win with a key), and notice runs of the same editing key
/// (Backspace ×10 is a word Ctrl+Backspace could delete). A few integer checks:
/// nothing here waits.
fn coach_keys(ctx: &mut Ctx, ev: &KeyEvent, fresh: bool) {
    use catkey_core::keys::bit::*;
    if !fresh || ev.injected || ctx.engine.is_recording() || catkey_core::keys::is_modifier(ev.vk) {
        return;
    }
    let mods = ctx.engine.held_mods();
    // Ctrl+Alt is AltGr on many layouts: it types characters, it isn't a shortcut.
    let altgr = mods & (LCTRL | RCTRL) != 0 && mods & RALT != 0;
    if mods & (LCTRL | RCTRL | LALT | RALT | LWIN | RWIN) != 0 && !altgr {
        crate::coach::count_combo(ctx.env.foreground_exe());
    }
    let vk = ev.vk;
    let editing = matches!(vk, 0x08 | 0x2E | 0x25 | 0x26 | 0x27 | 0x28);
    let h = &mut ctx.habit;
    if !editing || mods & !(LSHIFT | RSHIFT) != 0 {
        *h = Habit::default();
        return;
    }
    if h.vk == vk && h.mods == mods && ev.time.wrapping_sub(h.last) < 1_500 {
        h.n += 1;
    } else {
        *h = Habit { vk, mods, n: 1, last: ev.time, told: false };
    }
    h.last = ev.time;
    let need = match vk {
        0x08 | 0x2E => 10,
        0x25 | 0x27 => 12,
        _ => 15,
    };
    if h.n >= need && !h.told {
        h.told = true;
        crate::coach::habit(vk, mods, h.n);
    }
}

pub fn set_test_keys(on: bool) {
    TEST_KEYS.store(on, Ordering::SeqCst);
}

// ---------------------------------------------------------------- key log
// "Settings → Try a shortcut → Key log": every key Cat Key sees, what it decided and
// what it sent. Off unless the window turns it on; stops by itself after 10 minutes.

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
        DIAG_T0.store(unsafe { GetTickCount() }, Ordering::SeqCst);
        diag_line(format!(
            "key log started · Cat Key {} · hook {} · watchdog {}",
            env!("CARGO_PKG_VERSION"),
            if engine_running() { "on" } else { "OFF" },
            if WATCH_ON.load(Ordering::SeqCst) { "on" } else { "off" },
        ));
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

// ---------------------------------------------------------------- sender thread

static OUTBOX: Mutex<Vec<Out>> = Mutex::new(Vec::new());
static OUTBOX_READY: Condvar = Condvar::new();
/// Keys queued that haven't gone out yet. Until then Windows' own key state lags
/// behind ours, and a real key must wait its turn behind them.
static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);
/// When the queue last went from empty to busy.
static BUSY_SINCE: AtomicU32 = AtomicU32::new(0);
static LAST_SLOW_LOG: AtomicU32 = AtomicU32::new(0);
static STUCK_LOGGED: AtomicBool = AtomicBool::new(false);

fn enqueue(outs: &[Out]) {
    if outs.is_empty() {
        return;
    }
    if OUTSTANDING.fetch_add(outs.len(), Ordering::SeqCst) == 0 {
        BUSY_SINCE.store(unsafe { GetTickCount() }, Ordering::SeqCst);
    }
    match OUTBOX.lock() {
        Ok(mut q) => q.extend_from_slice(outs),
        Err(p) => p.into_inner().extend_from_slice(outs),
    }
    OUTBOX_READY.notify_one();
}

/// Our keys are still on their way (and not hopelessly stuck).
fn sending() -> bool {
    if OUTSTANDING.load(Ordering::SeqCst) == 0 {
        return false;
    }
    let busy = unsafe { GetTickCount() }.wrapping_sub(BUSY_SINCE.load(Ordering::SeqCst));
    if busy > STUCK_SEND_MS {
        if !STUCK_LOGGED.swap(true, Ordering::SeqCst) {
            crate::log("sending keys is stuck (another program's keyboard hook isn't answering)");
        }
        return false;
    }
    true
}

/// Wait (briefly) until everything queued has gone out.
fn wait_sent(max_ms: u32) {
    let start = unsafe { GetTickCount() };
    while OUTSTANDING.load(Ordering::SeqCst) > 0 && unsafe { GetTickCount() }.wrapping_sub(start) < max_ms {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

fn start_sender() {
    std::thread::Builder::new()
        .name("catkey-sender".into())
        .spawn(|| unsafe { sender_loop() })
        .expect("start key sender thread");
}

unsafe fn sender_loop() {
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
    crate::win::opt_out_of_power_throttling_for_thread();
    loop {
        let batch: Vec<Out> = {
            let mut q = match OUTBOX.lock() {
                Ok(q) => q,
                Err(p) => p.into_inner(),
            };
            while q.is_empty() {
                q = match OUTBOX_READY.wait(q) {
                    Ok(q) => q,
                    Err(p) => p.into_inner(),
                };
            }
            std::mem::take(&mut *q)
        };
        let started = GetTickCount();
        let result = std::panic::catch_unwind(|| send_outputs(&batch));
        // Count them as gone whatever happened, or real keys would wait forever.
        OUTSTANDING.fetch_sub(batch.len(), Ordering::SeqCst);
        STUCK_LOGGED.store(false, Ordering::SeqCst);
        let Ok((asked, sent, err)) = result else { continue };
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
                crate::log(&format!("sending {} keys took {took} ms (another program's keyboard hook is slow)", batch.len()));
            }
        }
    }
}

// ---------------------------------------------------------------- hook thread state

/// True while the keyboard hook is installed and its thread is running.
pub fn engine_running() -> bool {
    HOOK_OK.load(Ordering::SeqCst) && THREAD_ID.load(Ordering::SeqCst) != 0
}
static PENDING: Mutex<Vec<Cmd>> = Mutex::new(Vec::new());
static WATCH_ON: AtomicBool = AtomicBool::new(false);

struct Ctx {
    engine: Engine,
    env: WinEnv,
    hook: HHOOK,
    effects: Sender<Effect>,
    /// (time, vk, flags, scan) of the last event, to ignore an exact duplicate.
    last: (u32, u32, u32, u32),
    watch: Watch,
    /// Real keys held (to count a tracked shortcut once per press, not per repeat).
    track_down: [bool; 256],
    /// The run of one editing key pressed again and again (for the shortcut coach).
    habit: Habit,
    /// Keys whose press training kept from Windows (so their release is kept too).
    cap_down: [bool; 256],
}

#[derive(Default, Clone, Copy)]
struct Habit {
    vk: u8,
    mods: u8,
    n: u32,
    last: u32,
    told: bool,
}

impl Ctx {
    /// Queue what the engine just asked for.
    fn queue_out(&mut self) {
        enqueue(&self.engine.out);
        self.engine.out.clear();
    }

    fn reinstall(&mut self) -> bool {
        let fresh = unsafe { install() };
        if fresh.is_null() {
            return false;
        }
        if !self.hook.is_null() {
            unsafe { UnhookWindowsHookEx(self.hook) };
        }
        self.hook = fresh;
        HOOK_OK.store(true, Ordering::SeqCst);
        true
    }
}

/// One key event as both the hook and the keyboard (raw input) describe it.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Phys {
    scan: u16,
    ext: bool,
    down: bool,
}

/// What the raw-input watchdog knows.
#[derive(Default)]
struct Watch {
    hwnd: usize,
    /// When the hook last saw a real key.
    hook_seen: u32,
    /// Keys the hook saw go down and hasn't seen come up: (scan, ext, vk).
    held: Vec<(u16, bool, u8)>,
    /// Reported by the keyboard, not seen by the hook yet (with when).
    waiting: Vec<(Phys, u32)>,
    /// Seen by the hook, not reported by the keyboard yet (with when).
    recent: VecDeque<(Phys, u32)>,
    last_reinstall: u32,
    timer_on: bool,
}

impl Watch {
    fn reset(&mut self) {
        self.held.clear();
        self.waiting.clear();
        self.recent.clear();
    }

    /// The hook saw a real key.
    fn hook(&mut self, p: Phys, vk: u8, now: u32) {
        self.hook_seen = now;
        if let Some(i) = self.waiting.iter().position(|w| w.0 == p) {
            self.waiting.remove(i);
        } else {
            if self.recent.len() >= 64 {
                self.recent.pop_front();
            }
            self.recent.push_back((p, now));
        }
        let at = self.held.iter().position(|h| h.0 == p.scan && h.1 == p.ext);
        match (p.down, at) {
            (true, None) => self.held.push((p.scan, p.ext, vk)),
            (false, Some(i)) => {
                self.held.remove(i);
            }
            _ => {}
        }
    }

    /// The keyboard reported a key. True if we now wait to see it in the hook.
    fn raw(&mut self, p: Phys, now: u32) -> bool {
        if let Some(i) = self.recent.iter().position(|r| r.0 == p && now.wrapping_sub(r.1) < WATCH_GRACE_MS) {
            self.recent.remove(i);
            return false;
        }
        // Presses tell us whether the hook is alive; releases matter for keys it holds.
        if p.down || self.held.iter().any(|h| h.0 == p.scan && h.1 == p.ext) {
            if self.waiting.len() >= 64 {
                self.waiting.remove(0);
            }
            self.waiting.push((p, now));
            return true;
        }
        false
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

    fn focus(&mut self) -> usize {
        unsafe { GetForegroundWindow() as usize }
    }

    fn typing(&mut self) -> bool {
        // A text box with a blinking cursor (Win32 caret) has the keyboard.
        unsafe {
            let mut gti: GUITHREADINFO = zeroed();
            gti.cbSize = size_of::<GUITHREADINFO>() as u32;
            GetGUIThreadInfo(0, &mut gti) != 0 && !gti.hwndCaret.is_null()
        }
    }

    fn key_down_async(&mut self, vk: u8) -> bool {
        // While our keys are still on their way, Windows' answer is out of date
        // (a held-back Alt hasn't reached it yet): trust our own record instead.
        if OUTSTANDING.load(Ordering::SeqCst) > 0 {
            return true;
        }
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
    start_sender();
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
    let hwnd = create_watch_window();
    CTX.with(|c| {
        *c.borrow_mut() = Some(Ctx {
            engine: Engine::new(compiled, enabled),
            env: WinEnv::default(),
            hook,
            effects,
            last: (0, 0, 0, 0),
            watch: Watch { hwnd: hwnd as usize, ..Watch::default() },
            track_down: [false; 256],
            habit: Habit::default(),
            cap_down: [false; 256],
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
    if hwnd.is_null() {
        SetTimer(null_mut(), 0, REFRESH_MS, None); // thread timer: WM_TIMER with no window
    }
    drain_commands();

    loop {
        let r = GetMessageW(&mut msg, null_mut(), 0, 0);
        if r == 0 || r == -1 {
            break;
        }
        if msg.message == WM_CATKEY {
            drain_commands();
        } else if msg.message == WM_TIMER && msg.hwnd.is_null() {
            refresh_hook();
        } else {
            DispatchMessageW(&msg);
        }
    }
    wait_sent(500);
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
                if !ctx.reinstall() && diag_on() {
                    diag_line(format!("could not re-install the keyboard hook (error {})", GetLastError()));
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
        wait_sent(300); // let go of anything we hold before the thread ends
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
                ctx.watch.reset();
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
            diag_line(format!("      ↳ ours: {}{}", key_name(kb.vkCode as u8), if down { "↓" } else { "↑" }));
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
        let really_injected = kb.flags & LLKHF_INJECTED != 0;
        let ev = KeyEvent {
            vk: kb.vkCode as u8,
            scan: kb.scanCode as u16,
            down,
            extended: kb.flags & LLKHF_EXTENDED != 0,
            time: kb.time,
            injected: really_injected && !TEST_KEYS.load(Ordering::Relaxed),
        };
        if !really_injected && kb.scanCode & 0x200 == 0 && kb.scanCode != 0 {
            let p = Phys { scan: (kb.scanCode & 0xFF) as u16, ext: ev.extended, down };
            ctx.watch.hook(p, ev.vk, GetTickCount());
        }
        let logging = diag_on();
        if !really_injected {
            let vk = ev.vk as usize;
            if !ev.down && ctx.cap_down[vk] {
                ctx.cap_down[vk] = false;
                if !ctx.engine.is_recording() {
                    return true; // its press never reached Windows: neither does its release
                }
            } else if ev.down && CAPTURE.load(Ordering::Relaxed) && own_window_in_front() {
                ctx.cap_down[vk] = true;
                if !ctx.engine.is_recording() {
                    if logging {
                        diag_line(format!("{}↓  kept for the trainer", key_name(ev.vk)));
                    }
                    return true;
                }
            }
        }
        let held = if logging { held_now() } else { String::new() };
        let mut swallow = ctx.engine.handle(ev, &mut ctx.env);
        if !really_injected {
            let fresh = ev.down && !ctx.track_down[ev.vk as usize];
            count_tracked(ctx, &ev);
            coach_keys(ctx, &ev, fresh);
        }
        let mut deferred = false;
        let mut sends: Vec<Out> = Vec::new();
        if !swallow && !ev.injected && sending() {
            deferred = true;
            // A real key arrived while our keys are still on their way: send it after
            // them, so it can't overtake (e.g. land before the Ctrl we are releasing).
            // Keys from other programs keep their own markers, so those always pass.
            sends.push(Out::Raw { vk: ev.vk, scan: ev.scan, extended: ev.extended, down: ev.down });
            swallow = true;
        }
        sends.extend_from_slice(&ctx.engine.out);
        ctx.engine.out.clear();
        enqueue(&sends);
        if logging {
            let src = if really_injected {
                format!("typed by another program (tag {:#x}){}", kb.dwExtraInfo, if ev.injected { "" } else { " · test mode" })
            } else {
                "keyboard".to_string()
            };
            let app = ctx.env.foreground_exe().to_string();
            let what = if deferred { "held back (keys still going out)" } else if swallow { "swallowed" } else { "passed" };
            let sends = fmt_outs(&sends[if deferred { 1 } else { 0 }..]);
            diag_line(format!(
                "{}{}  from {}  held: {}  → {}{}  [{}]",
                key_name(ev.vk),
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
                ctx.watch.reset();
            }
        }
    });
}

// ---------------------------------------------------------------- watchdog

unsafe fn create_watch_window() -> HWND {
    let class = crate::win::wide("CatKeyWatchdog");
    let hinst = GetModuleHandleW(null());
    let mut wc: WNDCLASSW = zeroed();
    wc.lpfnWndProc = Some(watch_proc);
    wc.hInstance = hinst;
    wc.lpszClassName = class.as_ptr();
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(0, class.as_ptr(), null(), 0, 0, 0, 0, 0, HWND_MESSAGE, null_mut(), hinst, null());
    if hwnd.is_null() {
        crate::log(&format!("watchdog window failed (error {})", GetLastError()));
        return hwnd;
    }
    SetTimer(hwnd, TIMER_REFRESH, REFRESH_MS, None);
    let rid = RAWINPUTDEVICE { usUsagePage: 0x01, usUsage: 0x06, dwFlags: RIDEV_INPUTSINK, hwndTarget: hwnd };
    if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
        crate::log(&format!("watchdog couldn't listen to the keyboard (error {})", GetLastError()));
    } else {
        WATCH_ON.store(true, Ordering::SeqCst);
    }
    hwnd
}

unsafe extern "system" fn watch_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_INPUT => on_raw_input(lparam),
        WM_TIMER => {
            match wparam {
                TIMER_REFRESH => refresh_hook(),
                TIMER_WATCH => watch_check(),
                _ => {}
            }
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn on_raw_input(lparam: LPARAM) {
    let mut ri: RAWINPUT = zeroed();
    let mut size = size_of::<RAWINPUT>() as u32;
    let got = GetRawInputData(
        lparam as HRAWINPUT,
        RID_INPUT,
        &mut ri as *mut RAWINPUT as *mut c_void,
        &mut size,
        size_of::<RAWINPUTHEADER>() as u32,
    );
    if got == u32::MAX || got == 0 || ri.header.dwType != RIM_TYPEKEYBOARD {
        return;
    }
    // Keys typed by programs have no keyboard behind them.
    if ri.header.hDevice.is_null() {
        return;
    }
    let kb = ri.data.keyboard;
    let flags = kb.Flags as u32;
    if flags & RI_KEY_E1 != 0 || kb.MakeCode == 0 || kb.MakeCode >= 0x80 || kb.VKey == 0xFF {
        return;
    }
    let ext = flags & RI_KEY_E0 != 0;
    if ext && (kb.MakeCode == 0x2A || kb.MakeCode == 0x36) {
        return; // "fake Shift" around arrow keys
    }
    let p = Phys { scan: kb.MakeCode, ext, down: flags & RI_KEY_BREAK == 0 };
    let now = GetTickCount();
    CTX.with(|c| {
        let Ok(mut g) = c.try_borrow_mut() else { return };
        let Some(ctx) = g.as_mut() else { return };
        if ctx.watch.raw(p, now) && !ctx.watch.timer_on {
            ctx.watch.timer_on = true;
            SetTimer(ctx.watch.hwnd as HWND, TIMER_WATCH, 400, None);
        }
    });
}

/// Keys the keyboard reported that the hook still hasn't seen.
unsafe fn watch_check() {
    let now = GetTickCount();
    CTX.with(|c| {
        let Ok(mut g) = c.try_borrow_mut() else { return };
        let Some(ctx) = g.as_mut() else { return };
        let mut missed_press = false;
        let mut missed_release: Vec<(u8, Phys)> = Vec::new();
        let hook_seen = ctx.watch.hook_seen;
        let mut held = std::mem::take(&mut ctx.watch.held);
        ctx.watch.waiting.retain(|(p, at)| {
            if now.wrapping_sub(*at) < WATCH_GRACE_MS {
                return true;
            }
            if p.down {
                // The hook saw nothing at all since: it is gone, not just slow.
                if (hook_seen.wrapping_sub(*at) as i32) < -100 {
                    missed_press = true;
                }
            } else if let Some(i) = held.iter().position(|h| h.0 == p.scan && h.1 == p.ext) {
                missed_release.push((held[i].2, *p));
                held.remove(i);
            }
            false
        });
        ctx.watch.held = held;
        let logging = diag_on();
        for (vk, _) in &missed_release {
            if ctx.engine.missed_release(*vk) {
                WATCH_FIXES.fetch_add(1, Ordering::SeqCst);
                if logging {
                    let fix = fmt_outs(&ctx.engine.out);
                    diag_line(format!(
                        "watchdog: the keyboard released {} but the hook never saw it → finished that press{}",
                        key_name(*vk),
                        if fix.is_empty() { String::new() } else { format!(" (send: {fix})") }
                    ));
                }
                ctx.queue_out();
            }
        }
        if (missed_press || !missed_release.is_empty()) && now.wrapping_sub(ctx.watch.last_reinstall) > 3_000 {
            ctx.watch.last_reinstall = now;
            if ctx.reinstall() {
                WATCH_REINSTALLS.fetch_add(1, Ordering::SeqCst);
                if logging {
                    diag_line("watchdog: the hook was missing keys → installed it again".into());
                }
            }
        }
        if ctx.watch.waiting.is_empty() {
            ctx.watch.timer_on = false;
            KillTimer(ctx.watch.hwnd as HWND, TIMER_WATCH);
        }
    });
}

// ---------------------------------------------------------------- sending

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
