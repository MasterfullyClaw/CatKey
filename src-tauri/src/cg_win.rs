//! CatGPT on Windows: screenshots of the screen under the pointer (GDI, saved as small
//! JPEGs), very light screen recordings (Media Foundation's H.264 encoder at a low frame
//! rate and bitrate, so text stays sharp and files stay tiny), the AI apps' windows, and
//! the keys that paste into them.

use std::mem::{size_of, zeroed};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, SetBrushOrgEx, SetStretchBltMode,
    StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HALFTONE, HDC, SRCCOPY,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DrawIconEx, GetCursorInfo, GetForegroundWindow, GetIconInfo, GetWindowThreadProcessId, CURSORINFO, CURSOR_SHOWING, DI_NORMAL, ICONINFO,
};

/// A screen rectangle in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// The whole screen the pointer is on.
pub fn screen_under_pointer() -> Option<Area> {
    let (px, py) = crate::win::pointer()?;
    let s = crate::win::screen_at(px, py)?;
    Some(Area { x: s.full.x as i32, y: s.full.y as i32, w: s.full.w as i32, h: s.full.h as i32 })
}

/// Width and height after fitting into `max_w` (even numbers: the video encoder wants them).
fn fit(a: Area, max_w: i32) -> (i32, i32) {
    let (w, h) = if a.w > max_w { (max_w, (a.h as f64 * max_w as f64 / a.w as f64).round() as i32) } else { (a.w, a.h) };
    ((w / 2 * 2).max(2), (h / 2 * 2).max(2))
}

/// A bitmap the screen is copied into, scaled to (tw, th), top row first, BGRA.
struct Grab {
    screen: HDC,
    mem: HDC,
    dib: *mut core::ffi::c_void,
    old: *mut core::ffi::c_void,
    bits: *mut u8,
    tw: i32,
    th: i32,
}

impl Grab {
    unsafe fn new(tw: i32, th: i32) -> Option<Grab> {
        let screen = GetDC(std::ptr::null_mut());
        if screen.is_null() {
            return None;
        }
        let mem = CreateCompatibleDC(screen);
        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: tw,
            biHeight: -th,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..zeroed()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(mem, &bmi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        if dib.is_null() || bits.is_null() {
            DeleteDC(mem);
            ReleaseDC(std::ptr::null_mut(), screen);
            return None;
        }
        let old = SelectObject(mem, dib);
        SetStretchBltMode(mem, HALFTONE);
        SetBrushOrgEx(mem, 0, 0, std::ptr::null_mut());
        Some(Grab { screen, mem, dib, old, bits: bits as *mut u8, tw, th })
    }

    /// Copy the area in (scaled when needed), with the pointer drawn on top.
    unsafe fn take(&self, a: Area) -> bool {
        let ok = if a.w == self.tw && a.h == self.th {
            BitBlt(self.mem, 0, 0, self.tw, self.th, self.screen, a.x, a.y, SRCCOPY | CAPTUREBLT)
        } else {
            StretchBlt(self.mem, 0, 0, self.tw, self.th, self.screen, a.x, a.y, a.w, a.h, SRCCOPY | CAPTUREBLT)
        };
        if ok == 0 {
            return false;
        }
        self.cursor(a);
        // GDI leaves the alpha byte at 0: make every pixel opaque.
        let px = std::slice::from_raw_parts_mut(self.bits, (self.tw * self.th * 4) as usize);
        for p in px.chunks_exact_mut(4) {
            p[3] = 255;
        }
        true
    }

    unsafe fn cursor(&self, a: Area) {
        let mut ci: CURSORINFO = zeroed();
        ci.cbSize = size_of::<CURSORINFO>() as u32;
        if GetCursorInfo(&mut ci) == 0 || ci.flags & CURSOR_SHOWING == 0 || ci.hCursor.is_null() {
            return;
        }
        let mut ii: ICONINFO = zeroed();
        let (hx, hy) = if GetIconInfo(ci.hCursor, &mut ii) != 0 {
            if !ii.hbmMask.is_null() {
                DeleteObject(ii.hbmMask);
            }
            if !ii.hbmColor.is_null() {
                DeleteObject(ii.hbmColor);
            }
            (ii.xHotspot as i32, ii.yHotspot as i32)
        } else {
            (0, 0)
        };
        let k = self.tw as f64 / a.w as f64;
        let x = ((ci.ptScreenPos.x - a.x - hx) as f64 * k).round() as i32;
        let y = ((ci.ptScreenPos.y - a.y - hy) as f64 * k).round() as i32;
        let size = (32.0 * k.max(0.6)).round() as i32;
        DrawIconEx(self.mem, x, y, ci.hCursor, size, size, 0, std::ptr::null_mut(), DI_NORMAL);
    }

    fn pixels(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bits, (self.tw * self.th * 4) as usize) }
    }
}

impl Drop for Grab {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.mem, self.old);
            DeleteObject(self.dib);
            DeleteDC(self.mem);
            ReleaseDC(std::ptr::null_mut(), self.screen);
        }
    }
}

fn save_jpeg(path: &Path, bgra: &[u8], w: i32, h: i32, quality: u8) -> Result<(), String> {
    let enc = jpeg_encoder::Encoder::new_file(path, quality).map_err(|e| e.to_string())?;
    enc.encode(bgra, w as u16, h as u16, jpeg_encoder::ColorType::Bgra).map_err(|e| e.to_string())
}

/// A picture of the screen under the pointer, as a small JPEG. Returns (width, height).
pub fn screenshot(out: &Path, max_w: i32, quality: u8) -> Result<(u32, u32), String> {
    let a = screen_under_pointer().ok_or("No screen found under the pointer.")?;
    let (tw, th) = fit(a, max_w);
    unsafe {
        let g = Grab::new(tw, th).ok_or("Windows didn't give Cat Key the screen.")?;
        if !g.take(a) {
            return Err("Windows didn't give Cat Key the screen.".into());
        }
        save_jpeg(out, g.pixels(), tw, th, quality)?;
    }
    Ok((tw as u32, th as u32))
}

// ------------------------------------------------------------------ light screen recording

/// A recording that's running: stop it to get the file.
pub struct Recording {
    stop: Arc<AtomicBool>,
    job: std::thread::JoinHandle<Result<RecDone, String>>,
}

pub struct RecDone {
    pub w: u32,
    pub h: u32,
    pub secs: f64,
    pub frames: Vec<String>,
}

pub const FPS: u32 = 8;

/// Start recording the screen under the pointer into `out` (an .mp4). Every few seconds a
/// still is kept in `frames_dir` too: AI chats read pictures, not videos.
pub fn record_start(out: &Path, frames_dir: &Path, max_secs: u64) -> Result<Recording, String> {
    let a = screen_under_pointer().ok_or("No screen found under the pointer.")?;
    let (tw, th) = fit(a, 1920);
    let stop = Arc::new(AtomicBool::new(false));
    let (out, frames_dir, stop2) = (out.to_path_buf(), frames_dir.to_path_buf(), stop.clone());
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let job = std::thread::Builder::new()
        .name("catgpt-record".into())
        .spawn(move || record_loop(a, tw, th, &out, &frames_dir, &stop2, max_secs, tx))
        .map_err(|e| e.to_string())?;
    // Wait until the encoder is ready (or says why it isn't).
    match rx.recv_timeout(Duration::from_secs(6)) {
        Ok(Ok(())) => Ok(Recording { stop, job }),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            stop.store(true, Ordering::SeqCst);
            Err("The video encoder didn't start.".into())
        }
    }
}

impl Recording {
    pub fn stop(self) -> Result<RecDone, String> {
        self.stop.store(true, Ordering::SeqCst);
        self.job.join().map_err(|_| "The recording stopped badly.".to_string())?
    }
}

#[allow(clippy::too_many_arguments)]
fn record_loop(
    a: Area,
    tw: i32,
    th: i32,
    out: &Path,
    frames_dir: &Path,
    stop: &AtomicBool,
    max_secs: u64,
    ready: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<RecDone, String> {
    use windows::core::HSTRING;
    use windows::Win32::Media::MediaFoundation::*;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    struct Mf;
    impl Drop for Mf {
        fn drop(&mut self) {
            unsafe {
                let _ = MFShutdown();
                CoUninitialize();
            }
        }
    }

    let fail = |e: String| {
        let _ = ready.send(Err(e.clone()));
        Err(e)
    };
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        if let Err(e) = MFStartup(MF_VERSION, MFSTARTUP_FULL) {
            CoUninitialize();
            return fail(format!("Media Foundation isn't available here ({e})."));
        }
        let _mf = Mf;
        let pack = |hi: u32, lo: u32| ((hi as u64) << 32) | lo as u64;
        // Low frame rate, modest bitrate: screens hardly change between frames, so text stays sharp.
        let bitrate = ((tw as f64 * th as f64 * FPS as f64 * 0.055) as u32).clamp(350_000, 1_800_000);
        let setup = || -> windows::core::Result<(IMFSinkWriter, u32)> {
            let writer = MFCreateSinkWriterFromURL(&HSTRING::from(out.to_string_lossy().as_ref()), None, None)?;
            let ot = MFCreateMediaType()?;
            ot.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            ot.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
            ot.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)?;
            ot.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            ot.SetUINT64(&MF_MT_FRAME_SIZE, pack(tw as u32, th as u32))?;
            ot.SetUINT64(&MF_MT_FRAME_RATE, pack(FPS, 1))?;
            ot.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack(1, 1))?;
            let stream = writer.AddStream(&ot)?;
            let it = MFCreateMediaType()?;
            it.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            it.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
            it.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            it.SetUINT64(&MF_MT_FRAME_SIZE, pack(tw as u32, th as u32))?;
            it.SetUINT64(&MF_MT_FRAME_RATE, pack(FPS, 1))?;
            it.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack(1, 1))?;
            // A positive stride: the rows come top first.
            it.SetUINT32(&MF_MT_DEFAULT_STRIDE, (tw * 4) as u32)?;
            writer.SetInputMediaType(stream, &it, None)?;
            writer.BeginWriting()?;
            Ok((writer, stream))
        };
        let (writer, stream) = match setup() {
            Ok(x) => x,
            Err(e) => return fail(format!("The video encoder didn't start ({e}).")),
        };
        let Some(g) = Grab::new(tw, th) else { return fail("Windows didn't give Cat Key the screen.".into()) };
        let _ = ready.send(Ok(()));

        let len = (tw * th * 4) as u32;
        let frame = Duration::from_nanos(1_000_000_000 / FPS as u64);
        let start = Instant::now();
        let mut n: u64 = 0;
        let mut frames: Vec<String> = Vec::new();
        let mut next_still = Duration::ZERO;
        let mut last_err = None;
        while !stop.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(max_secs) {
            let due = frame * n as u32;
            let now = start.elapsed();
            if now < due {
                std::thread::sleep(due - now);
            }
            if !g.take(a) {
                continue;
            }
            let write = || -> windows::core::Result<()> {
                let buf = MFCreateMemoryBuffer(len)?;
                let mut p: *mut u8 = std::ptr::null_mut();
                buf.Lock(&mut p, None, None)?;
                std::ptr::copy_nonoverlapping(g.pixels().as_ptr(), p, len as usize);
                buf.Unlock()?;
                buf.SetCurrentLength(len)?;
                let s = MFCreateSample()?;
                s.AddBuffer(&buf)?;
                s.SetSampleTime((n as i64) * 10_000_000 / FPS as i64)?;
                s.SetSampleDuration(10_000_000 / FPS as i64)?;
                writer.WriteSample(stream, &s)
            };
            if let Err(e) = write() {
                last_err = Some(e.to_string());
                break;
            }
            // A still for the AI every 3 seconds (the first one at once).
            if start.elapsed() >= next_still && frames.len() < 40 {
                let p = frames_dir.join(format!("frame{:02}.jpg", frames.len() + 1));
                if save_jpeg(&p, g.pixels(), tw, th, 78).is_ok() {
                    frames.push(p.to_string_lossy().to_string());
                }
                next_still += Duration::from_secs(3);
            }
            n += 1;
        }
        let secs = n as f64 / FPS as f64;
        writer.Finalize().map_err(|e| format!("The video couldn't be finished ({e})."))?;
        if let Some(e) = last_err {
            if n < 2 {
                return Err(format!("The recording stopped: {e}"));
            }
        }
        Ok(RecDone { w: tw as u32, h: th as u32, secs, frames })
    }
}

// ------------------------------------------------------------------ the AI apps

/// The exe of the window in front, lower case ("claude.exe").
pub fn front_exe() -> String {
    unsafe {
        let fg: HWND = GetForegroundWindow();
        if fg.is_null() {
            return String::new();
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, &mut pid);
        crate::win::process_path(pid).map(|p| crate::win::file_name_lower(&p)).unwrap_or_default()
    }
}

/// Bring a running app's window to the front (by words in its exe name). False if none is open.
pub fn activate(words: &[&str]) -> bool {
    let items = crate::win::switch_windows();
    let hit = items.iter().find(|i| {
        let e = i.exe.to_lowercase();
        words.iter().any(|w| e.contains(w))
    });
    match hit {
        Some(i) => crate::win::switch_to(&i.id, false),
        None => false,
    }
}

fn key(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: if up { KEYEVENTF_KEYUP } else { 0 }, time: 0, dwExtraInfo: crate::hook::MARK },
        },
    }
}

/// Press a shortcut in whatever is in front: modifiers ("ctrl", "alt", "shift") and a key
/// ("v", "n", "enter", "1"…"9").
pub fn press(mods: &[&str], k: &str) {
    let vk_of = |m: &str| match m {
        "ctrl" => 0x11u16,
        "alt" => 0x12,
        "shift" => 0x10,
        _ => 0,
    };
    let main: u16 = match k {
        "enter" => 0x0D,
        "tab" => 0x09,
        k if k.len() == 1 => k.to_ascii_uppercase().as_bytes()[0] as u16,
        _ => return,
    };
    let mut seq: Vec<INPUT> = Vec::new();
    for m in mods {
        let v = vk_of(m);
        if v != 0 {
            seq.push(key(v, false));
        }
    }
    seq.push(key(main, false));
    seq.push(key(main, true));
    for m in mods.iter().rev() {
        let v = vk_of(m);
        if v != 0 {
            seq.push(key(v, true));
        }
    }
    unsafe {
        SendInput(seq.len() as u32, seq.as_ptr(), size_of::<INPUT>() as i32);
    }
}

/// The paste shortcut on this computer.
pub fn paste() {
    press(&["ctrl"], "v");
}


/// The window in front now, as the app switcher names it (its handle), to go back to later.
pub fn front_id() -> Option<String> {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, &mut pid);
        (pid != std::process::id()).then(|| (fg as usize).to_string())
    }
}

// ------------------------------------------------------------------ Claude's recent chats

/// The chats in the Claude app's sidebar, most recent first: (title, chat id). Read through UI
/// Automation, the way screen readers read them.
pub fn claude_recent() -> Vec<(String, String)> {
    use windows::core::{Interface, BSTR};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::Accessibility::*;
    let Some(win) = crate::win::switch_windows().into_iter().find(|i| i.exe.to_lowercase().contains("claude")) else { return Vec::new() };
    let Ok(h) = win.id.parse::<usize>() else { return Vec::new() };
    let mut out: Vec<(String, String)> = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let mut run = || -> windows::core::Result<()> {
            let auto: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
                .or_else(|_| CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER))?;
            if let Ok(a2) = auto.cast::<IUIAutomation2>() {
                let _ = a2.SetConnectionTimeout(2000);
                let _ = a2.SetTransactionTimeout(3000);
            }
            let root = auto.ElementFromHandle(windows::Win32::Foundation::HWND(h as *mut core::ffi::c_void))?;
            let cond = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(UIA_HyperlinkControlTypeId.0))?;
            let all = root.FindAll(TreeScope_Descendants, &cond)?;
            for i in 0..all.Length()?.min(400) {
                let Ok(el) = all.GetElement(i) else { continue };
                let url = el.GetCurrentPropertyValue(UIA_ValueValuePropertyId).ok().and_then(|v| BSTR::try_from(&v).ok()).map(|b| b.to_string()).unwrap_or_default();
                let Some(rest) = url.split("/chat/").nth(1) else { continue };
                let id: String = rest.chars().take_while(|c| c.is_ascii_hexdigit() || *c == '-').collect();
                if id.len() < 32 || out.iter().any(|(_, x)| *x == id) {
                    continue;
                }
                let name = el.CurrentName().map(|b| b.to_string()).unwrap_or_default();
                out.push((if name.trim().is_empty() { "A chat".into() } else { name.chars().take(80).collect() }, id));
                if out.len() >= 15 {
                    break;
                }
            }
            Ok(())
        };
        if let Err(e) = run() {
            crate::log(&format!("catgpt: Claude's chats couldn't be read ({e})"));
        }
    }
    out
}
