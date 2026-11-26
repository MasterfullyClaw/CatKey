//! Small, safe wrappers around the Windows APIs Cat Key needs
//! (foreground app, running apps, icons, startup, elevation, launching).

use serde::Serialize;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::ptr::{null, null_mut};

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_CANCELLED, ERROR_FILE_NOT_FOUND, HANDLE,
    HWND, INVALID_HANDLE_VALUE, LPARAM, SYSTEMTIME,
};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS,
};
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;
use windows_sys::Win32::System::Registry::{
    RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ,
};
use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, GetCurrentProcessId, GetCurrentThread, OpenProcess, OpenProcessToken, ProcessPowerThrottling,
    QueryFullProcessImageNameW, SetProcessInformation, SetThreadInformation, ThreadPowerThrottling, WaitForSingleObject,
    PROCESS_NAME_WIN32, PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED, THREAD_POWER_THROTTLING_STATE,
};
use windows_sys::Win32::UI::Shell::{
    SHDefExtractIconW, SHGetFileInfoW, ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW,
    SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, DestroyIcon, ASFW_ANY, EnumChildWindows, EnumWindows, GetIconInfo, GetWindow, GetWindowLongPtrW, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GW_OWNER, HICON, ICONINFO, SW_SHOWNORMAL,
    WS_EX_TOOLWINDOW,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Shell APIs (opening files/links, icons) want COM on the calling thread.
fn ensure_com() {
    thread_local! {
        static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    DONE.with(|d| {
        if !d.get() {
            unsafe {
                CoInitializeEx(null(), (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32);
            }
            d.set(true);
        }
    });
}

/// Let another process take the foreground (we are the one the user just started).
pub fn allow_any_foreground() {
    unsafe {
        AllowSetForegroundWindow(ASFW_ANY);
    }
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn file_name_lower(path: &str) -> String {
    path.rsplit(['\\', '/']).next().unwrap_or(path).to_lowercase()
}

/// Full path of a running process.
pub fn process_path(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        CloseHandle(h);
        if ok == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

unsafe extern "system" fn find_other_pid(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam as *mut (u32, u32));
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != 0 && pid != data.0 {
        data.1 = pid;
        return 0;
    }
    1
}

/// Store apps live inside ApplicationFrameHost; find the real app's process.
fn real_pid(hwnd: HWND, pid: u32, exe: &str) -> u32 {
    if exe != "applicationframehost.exe" {
        return pid;
    }
    let mut data: (u32, u32) = (pid, 0);
    unsafe {
        EnumChildWindows(hwnd, Some(find_other_pid), &mut data as *mut _ as LPARAM);
    }
    if data.1 != 0 {
        data.1
    } else {
        pid
    }
}

/// Lowercase exe name for a window, e.g. "chrome.exe".
pub fn window_exe(hwnd: HWND, pid: u32) -> Option<String> {
    let path = process_path(pid)?;
    let exe = file_name_lower(&path);
    let real = real_pid(hwnd, pid, &exe);
    if real != pid {
        return process_path(real).map(|p| file_name_lower(&p));
    }
    Some(exe)
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub exe: String,
    pub name: String,
    pub path: String,
    pub title: String,
}

unsafe extern "system" fn collect_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = &mut *(lparam as *mut Vec<usize>);
    list.push(hwnd as usize);
    1
}

/// Apps with a visible window right now (for the app picker).
pub fn running_apps() -> Vec<AppInfo> {
    let mut hwnds: Vec<usize> = Vec::new();
    unsafe {
        EnumWindows(Some(collect_windows), &mut hwnds as *mut _ as LPARAM);
    }
    let me = unsafe { GetCurrentProcessId() };
    let mut out: Vec<AppInfo> = Vec::new();
    for h in hwnds {
        let hwnd = h as HWND;
        unsafe {
            if IsWindowVisible(hwnd) == 0 || !GetWindow(hwnd, GW_OWNER).is_null() {
                continue;
            }
            if (GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32) & WS_EX_TOOLWINDOW != 0 {
                continue;
            }
            let mut cloaked: u32 = 0;
            if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED as u32, &mut cloaked as *mut _ as *mut c_void, 4) == 0
                && cloaked != 0
            {
                continue;
            }
            let len = GetWindowTextLengthW(hwnd);
            if len <= 0 {
                continue;
            }
            let mut buf = vec![0u16; len as usize + 1];
            GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            let title = from_wide(&buf);
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == me {
                continue;
            }
            let Some(path0) = process_path(pid) else { continue };
            let exe0 = file_name_lower(&path0);
            let rp = real_pid(hwnd, pid, &exe0);
            let path = if rp != pid { process_path(rp).unwrap_or(path0) } else { path0 };
            let exe = file_name_lower(&path);
            if exe.is_empty() || exe == "applicationframehost.exe" || out.iter().any(|a| a.exe == exe) {
                continue;
            }
            let name = file_description(&path)
                .filter(|d| !d.trim().is_empty())
                .unwrap_or_else(|| exe.trim_end_matches(".exe").to_string());
            out.push(AppInfo { exe, name, path, title });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// "Google Chrome" for chrome.exe, from the file's version info.
pub fn file_description(path: &str) -> Option<String> {
    unsafe {
        let w = wide(path);
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(w.as_ptr(), &mut handle);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(w.as_ptr(), 0, size, data.as_mut_ptr() as *mut c_void) == 0 {
            return None;
        }
        let mut ptr: *mut c_void = null_mut();
        let mut len = 0u32;
        let mut pairs: Vec<(u16, u16)> = Vec::new();
        let tkey = wide("\\VarFileInfo\\Translation");
        if VerQueryValueW(data.as_ptr() as *const c_void, tkey.as_ptr(), &mut ptr, &mut len) != 0 && len >= 4 {
            let p = ptr as *const u16;
            pairs.push((*p, *p.add(1)));
        }
        pairs.push((0x0409, 0x04B0));
        pairs.push((0x0409, 0x04E4));
        for (lang, cp) in pairs {
            let key = wide(&format!("\\StringFileInfo\\{:04x}{:04x}\\FileDescription", lang, cp));
            if VerQueryValueW(data.as_ptr() as *const c_void, key.as_ptr(), &mut ptr, &mut len) != 0 && len > 1 {
                let s = std::slice::from_raw_parts(ptr as *const u16, len as usize);
                let text = from_wide(s);
                if !text.trim().is_empty() {
                    return Some(text.trim().to_string());
                }
            }
        }
        None
    }
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

unsafe fn hicon_to_png(icon: HICON) -> Option<Vec<u8>> {
    let mut ii: ICONINFO = zeroed();
    if GetIconInfo(icon, &mut ii) == 0 {
        return None;
    }
    let cleanup = |ii: &ICONINFO| {
        if !ii.hbmColor.is_null() {
            DeleteObject(ii.hbmColor);
        }
        if !ii.hbmMask.is_null() {
            DeleteObject(ii.hbmMask);
        }
    };
    if ii.hbmColor.is_null() {
        cleanup(&ii);
        return None;
    }
    let mut bm: BITMAP = zeroed();
    GetObjectW(ii.hbmColor, size_of::<BITMAP>() as i32, &mut bm as *mut _ as *mut c_void);
    let (w, h) = (bm.bmWidth, bm.bmHeight);
    if w <= 0 || h <= 0 || w > 512 || h > 512 {
        cleanup(&ii);
        return None;
    }
    let hdc = CreateCompatibleDC(null_mut());
    let mut bi: BITMAPINFO = zeroed();
    bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = w;
    bi.bmiHeader.biHeight = -h;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    let mut px = vec![0u8; (w * h * 4) as usize];
    let got = GetDIBits(hdc, ii.hbmColor, 0, h as u32, px.as_mut_ptr() as *mut c_void, &mut bi, DIB_RGB_COLORS);
    if got == 0 {
        DeleteDC(hdc);
        cleanup(&ii);
        return None;
    }
    if !px.chunks_exact(4).any(|p| p[3] != 0) && !ii.hbmMask.is_null() {
        // Old-style icon without alpha: build alpha from the mask.
        let mut mask = vec![0u8; (w * h * 4) as usize];
        let mut mi: BITMAPINFO = zeroed();
        mi.bmiHeader = bi.bmiHeader;
        if GetDIBits(hdc, ii.hbmMask, 0, h as u32, mask.as_mut_ptr() as *mut c_void, &mut mi, DIB_RGB_COLORS) != 0 {
            for (p, m) in px.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
                p[3] = if m[0] == 0 { 255 } else { 0 };
            }
        } else {
            for p in px.chunks_exact_mut(4) {
                p[3] = 255;
            }
        }
    }
    DeleteDC(hdc);
    cleanup(&ii);
    for p in px.chunks_exact_mut(4) {
        p.swap(0, 2); // BGRA -> RGBA
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().ok()?;
        wr.write_image_data(&px).ok()?;
    }
    Some(out)
}

/// The icon of a program / file as a PNG data URL (48 px when available).
pub fn icon_data_url(path: &str) -> Option<String> {
    ensure_com();
    let path = expand_env(path);
    if path.is_empty() || path.contains("://") {
        return None;
    }
    unsafe {
        let w = wide(&path);
        let mut large: HICON = null_mut();
        let hr = SHDefExtractIconW(w.as_ptr(), 0, 0, &mut large, null_mut(), 48 | (16 << 16));
        if hr != 0 || large.is_null() {
            let mut sfi: SHFILEINFOW = zeroed();
            let r = SHGetFileInfoW(w.as_ptr(), 0, &mut sfi, size_of::<SHFILEINFOW>() as u32, SHGFI_ICON | SHGFI_LARGEICON);
            if r == 0 || sfi.hIcon.is_null() {
                return None;
            }
            large = sfi.hIcon;
        }
        let png = hicon_to_png(large);
        DestroyIcon(large);
        png.map(|b| format!("data:image/png;base64,{}", base64(&b)))
    }
}

pub fn expand_env(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    unsafe {
        let w = wide(s);
        let mut buf = vec![0u16; 2048];
        let n = ExpandEnvironmentStringsW(w.as_ptr(), buf.as_mut_ptr(), buf.len() as u32);
        if n == 0 || n as usize > buf.len() {
            return s.to_string();
        }
        from_wide(&buf)
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elev: TOKEN_ELEVATION = zeroed();
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elev as *mut _ as *mut c_void,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        );
        CloseHandle(token);
        ok != 0 && elev.TokenIsElevated != 0
    }
}

/// PowerToys "Grab And Move" is switched on. It watches the Alt key (its default) to
/// move windows with Alt + drag, which briefly hides Alt from other programs.
pub fn powertoys_grab_and_move_on() -> bool {
    let Ok(local) = std::env::var("LOCALAPPDATA") else { return false };
    let path = std::path::Path::new(&local).join("Microsoft").join("PowerToys").join("settings.json");
    let Ok(text) = std::fs::read_to_string(path) else { return false };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return false };
    let Some(enabled) = v.get("enabled").and_then(|e| e.as_object()) else { return false };
    enabled.iter().any(|(k, on)| {
        let k: String = k.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
        k == "grabandmove" && on.as_bool() == Some(true)
    }) && process_running("PowerToys.exe")
}

/// Is a process with this exe name running? (Case-insensitive.)
pub fn process_running(name: &str) -> bool {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut pe: PROCESSENTRY32W = zeroed();
        pe.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        if Process32FirstW(snap, &mut pe) != 0 {
            loop {
                if from_wide(&pe.szExeFile).eq_ignore_ascii_case(name) {
                    found = true;
                    break;
                }
                if Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}

/// Full path of the first running process with this exe name.
pub fn find_process_path(name: &str) -> Option<String> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut pe: PROCESSENTRY32W = zeroed();
        pe.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut found = None;
        if Process32FirstW(snap, &mut pe) != 0 {
            loop {
                if from_wide(&pe.szExeFile).eq_ignore_ascii_case(name) {
                    found = process_path(pe.th32ProcessID);
                    if found.is_some() {
                        break;
                    }
                }
                if Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}

/// Keep Windows 11 "efficiency mode" from slowing the keyboard thread down.
pub fn opt_out_of_power_throttling_for_process() {
    unsafe {
        let st = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &st as *const _ as *const c_void,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

pub fn opt_out_of_power_throttling_for_thread() {
    unsafe {
        let st = THREAD_POWER_THROTTLING_STATE {
            Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        SetThreadInformation(
            GetCurrentThread(),
            ThreadPowerThrottling,
            &st as *const _ as *const c_void,
            size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

/// True if another Cat Key is already running. The first copy keeps its mutex
/// for its whole life, so a second copy never installs a second keyboard hook.
pub fn another_instance_running() -> bool {
    unsafe {
        let name = wide("Local\\CatKey.SingleInstance");
        let h = CreateMutexW(null(), 0, name.as_ptr());
        let err = GetLastError();
        if h.is_null() {
            // Exists but belongs to an admin copy we can't open.
            return err == ERROR_ACCESS_DENIED;
        }
        if err == ERROR_ALREADY_EXISTS {
            CloseHandle(h);
            return true;
        }
        // Keep the handle open until the process exits.
        false
    }
}

pub fn wait_for_pid(pid: u32, timeout_ms: u32) {
    unsafe {
        let h = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !h.is_null() {
            WaitForSingleObject(h, timeout_ms);
            CloseHandle(h);
        }
    }
}

pub fn local_date() -> String {
    unsafe {
        let mut st: SYSTEMTIME = zeroed();
        GetLocalTime(&mut st);
        format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
    }
}

fn shell_execute(verb: Option<&str>, file: &str, params: &str, dir: Option<&str>) -> Result<(), u32> {
    ensure_com();
    unsafe {
        let verb_w = verb.map(wide);
        let file_w = wide(file);
        let params_w = wide(params);
        let dir_w = dir.map(wide);
        let mut sei: SHELLEXECUTEINFOW = zeroed();
        sei.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI;
        sei.lpVerb = verb_w.as_ref().map(|v| v.as_ptr()).unwrap_or(null());
        sei.lpFile = file_w.as_ptr();
        sei.lpParameters = if params.is_empty() { null() } else { params_w.as_ptr() };
        sei.lpDirectory = dir_w.as_ref().map(|d| d.as_ptr()).unwrap_or(null());
        sei.nShow = SW_SHOWNORMAL;
        if ShellExecuteExW(&mut sei) == 0 {
            Err(GetLastError())
        } else {
            Ok(())
        }
    }
}

/// Open a program, file, folder or URL.
pub fn launch(target: &str, args: &str) -> Result<(), String> {
    let target = expand_env(target.trim().trim_matches('"'));
    if target.is_empty() {
        return Err("nothing to open".into());
    }
    let is_url = target.contains("://") || target.starts_with("mailto:");
    // Running as admin: let Explorer open it so the app doesn't inherit admin rights.
    if is_elevated() && args.trim().is_empty() {
        let quoted = format!("\"{}\"", target);
        if shell_execute(None, "explorer.exe", &quoted, None).is_ok() {
            return Ok(());
        }
    }
    let dir = if is_url {
        None
    } else {
        Path::new(&target).parent().filter(|p| p.is_dir()).map(|p| p.to_string_lossy().to_string())
    };
    shell_execute(None, &target, args.trim(), dir.as_deref()).map_err(|e| format!("Windows couldn't open it (error {e})"))
}

/// Ask Windows (UAC) to start a new copy of Cat Key as administrator.
pub fn relaunch_elevated(args: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    match shell_execute(Some("runas"), &exe.to_string_lossy(), args, None) {
        Ok(()) => Ok(()),
        Err(e) if e == ERROR_CANCELLED => Err("cancelled".into()),
        Err(e) => Err(format!("Windows refused (error {e})")),
    }
}

/// From an admin copy: start a normal copy after this one exits.
pub fn relaunch_unelevated_after_exit() {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let _ = Command::new("cmd")
        .args(["/c", &format!("ping -n 3 127.0.0.1 >nul & explorer.exe \"{}\"", exe)])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

// ------------------------------------------------------------------- start with Windows

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_NAME: &str = "CatKey";
pub const TASK_NAME: &str = "CatKey (admin)";

pub fn autostart_command() -> String {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    format!("\"{}\" --autostart", exe)
}

pub fn get_run_entry() -> Option<String> {
    unsafe {
        let key = wide(RUN_KEY);
        let name = wide(RUN_NAME);
        let mut buf = vec![0u16; 2048];
        let mut size = (buf.len() * 2) as u32;
        let r = RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
        );
        if r != 0 {
            return None;
        }
        Some(from_wide(&buf))
    }
}

/// Windows' own on/off switch for our startup entry (Settings › Apps › Startup,
/// Task Manager › Startup apps). `Some(false)` = switched off there.
pub fn startup_approved() -> Option<bool> {
    unsafe {
        let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run");
        let name = wide(RUN_NAME);
        let mut buf = [0u8; 32];
        let mut size = buf.len() as u32;
        let r = RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_BINARY,
            null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
        );
        if r != 0 || size == 0 {
            return None;
        }
        Some(buf[0] & 1 == 0)
    }
}

pub fn set_run_entry(command: Option<&str>) -> bool {
    unsafe {
        let key = wide(RUN_KEY);
        let name = wide(RUN_NAME);
        match command {
            Some(c) => {
                let v = wide(c);
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    v.as_ptr() as *const c_void,
                    (v.len() * 2) as u32,
                ) == 0
            }
            None => {
                let r = RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr());
                r == 0 || r == ERROR_FILE_NOT_FOUND
            }
        }
    }
}

fn schtasks(args: &[&str]) -> Result<String, String> {
    let out = Command::new("schtasks.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Last answer from `schtasks /Query` (it starts a process, so don't ask every few seconds).
static TASK_CACHE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

pub fn admin_task_exists() -> bool {
    // Fast path: registered tasks live as files under %SystemRoot%\System32\Tasks.
    if let Ok(root) = std::env::var("SystemRoot") {
        if std::path::Path::new(&root).join("System32").join("Tasks").join(TASK_NAME).exists() {
            return true;
        }
    }
    let mut cache = TASK_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, exists)) = *cache {
        if at.elapsed() < Duration::from_secs(30) {
            return exists;
        }
    }
    let exists = schtasks(&["/Query", "/TN", TASK_NAME]).is_ok();
    *cache = Some((Instant::now(), exists));
    exists
}

fn forget_task_cache() {
    *TASK_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A logon task that starts Cat Key with admin rights (needs an admin process).
/// Unlike schtasks' defaults it never stops after 3 days and runs on battery.
pub fn create_admin_task() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?.to_string_lossy().to_string();
    let user = match (std::env::var("USERDOMAIN"), std::env::var("USERNAME")) {
        (Ok(d), Ok(u)) => format!("{d}\\{u}"),
        (_, Ok(u)) => u,
        _ => return Err("unknown user".into()),
    };
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Starts Cat Key with admin rights so shortcuts also work in admin apps.</Description></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId><Delay>PT2S</Delay></LogonTrigger></Triggers>
  <Principals><Principal id="Author"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings><StopOnIdleEnd>false</StopOnIdleEnd><RestartOnIdle>false</RestartOnIdle></IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author"><Exec><Command>{exe}</Command><Arguments>--autostart</Arguments></Exec></Actions>
</Task>
"#,
        user = xml_escape(&user),
        exe = xml_escape(&exe)
    );
    let path = std::env::temp_dir().join("catkey-task.xml");
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
    for u in xml.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let p = path.to_string_lossy().to_string();
    let r = schtasks(&["/Create", "/TN", TASK_NAME, "/XML", &p, "/F"]).map(|_| ());
    forget_task_cache();
    let _ = std::fs::remove_file(&path);
    r
}

pub fn delete_admin_task() -> Result<(), String> {
    if !admin_task_exists() {
        return Ok(());
    }
    let r = schtasks(&["/Delete", "/TN", TASK_NAME, "/F"]).map(|_| ());
    forget_task_cache();
    r
}

// ------------------------------------------------------------------- clipboard (Cat Builder's way back)

/// Changes every time anything is copied. Cheap: no clipboard access.
pub fn clipboard_seq() -> u32 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() }
}

/// The text on the clipboard, if any (up to 2 MB).
pub fn clipboard_text() -> Option<String> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard};
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    const CF_UNICODETEXT: u32 = 13;
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 || OpenClipboard(null_mut()) == 0 {
            return None;
        }
        let mut out = None;
        let h = GetClipboardData(CF_UNICODETEXT);
        if !h.is_null() {
            let p = GlobalLock(h as _) as *const u16;
            if !p.is_null() {
                let mut len = 0usize;
                while len < 1_000_000 && *p.add(len) != 0 {
                    len += 1;
                }
                out = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, len)));
                GlobalUnlock(h as _);
            }
        }
        CloseClipboard();
        out
    }
}
