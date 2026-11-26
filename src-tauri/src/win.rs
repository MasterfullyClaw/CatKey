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
    AssocQueryStringW, SHDefExtractIconW, SHGetFileInfoW, ShellExecuteExW, ASSOCF_IS_PROTOCOL, ASSOCF_NOTRUNCATE, ASSOCSTR_COMMAND,
    SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
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
/// Apps in the Start menu (yours and everyone's): their name, exe and where they live.
pub fn installed_apps() -> Vec<AppInfo> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::{
        IShellLinkW, SHGetKnownFolderPath, ShellLink, FOLDERID_CommonPrograms, FOLDERID_Programs, KF_FLAG_DEFAULT,
    };

    fn links(dir: &Path, depth: u32, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if depth < 3 && !catkey_core::appscan::skip_folder(&name) {
                    links(&p, depth + 1, out);
                }
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")) {
                out.push(p);
            }
        }
    }

    let mut out: Vec<AppInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    unsafe {
        let inited = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let made: windows::core::Result<IShellLinkW> = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER);
        if let Ok(link) = made {
            if let Ok(file) = link.cast::<IPersistFile>() {
                for id in [&FOLDERID_Programs, &FOLDERID_CommonPrograms] {
                    let Ok(p) = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) else { continue };
                    let dir = p.to_string().unwrap_or_default();
                    CoTaskMemFree(Some(p.0 as *const c_void));
                    let mut found = Vec::new();
                    links(Path::new(&dir), 0, &mut found);
                    for lnk in found {
                        let w: Vec<u16> = lnk.as_os_str().encode_wide().chain(Some(0)).collect();
                        if file.Load(PCWSTR(w.as_ptr()), STGM_READ).is_err() {
                            continue;
                        }
                        let mut target = [0u16; 1024];
                        if link.GetPath(&mut target, null_mut(), 0).is_err() {
                            continue;
                        }
                        let target = from_wide(&target);
                        let mut args = [0u16; 1024];
                        let _ = link.GetArguments(&mut args);
                        let name = lnk.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                        let Some((name, exe)) = catkey_core::appscan::from_shortcut(&name, &target, &from_wide(&args)) else {
                            continue;
                        };
                        if !Path::new(&target).exists() || !seen.insert(exe.clone()) {
                            continue;
                        }
                        out.push(AppInfo { exe, name, path: target, title: String::new() });
                    }
                }
            }
        }
        if inited {
            CoUninitialize();
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

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

/// "2026-11-26 0352": for backup folder names.
pub fn local_stamp() -> String {
    unsafe {
        let mut st: SYSTEMTIME = zeroed();
        GetLocalTime(&mut st);
        format!("{:04}-{:02}-{:02} {:02}{:02}", st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute)
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
    let dir = if is_url {
        None
    } else {
        Path::new(&target).parent().filter(|p| p.is_dir()).map(|p| p.to_string_lossy().to_string())
    };
    // Running as admin: let Explorer start it, so it runs as the user and doesn't inherit admin rights.
    if is_elevated() {
        if shell_execute_via_explorer(&target, args.trim(), dir.as_deref().unwrap_or("")).is_ok() {
            return Ok(());
        }
        if args.trim().is_empty() {
            let quoted = format!("\"{}\"", target);
            if shell_execute(None, "explorer.exe", &quoted, None).is_ok() {
                return Ok(());
            }
        }
    }
    shell_execute(None, &target, args.trim(), dir.as_deref()).map_err(|e| format!("Windows couldn't open it (error {e})"))
}

// ---------------------------------------------------------------- links of any length

/// Open a link (https, or an app's own link like claude://) of any length, never as administrator.
/// Short links go the usual way. Long ones skip Windows' link handling, which cuts them short
/// (Explorer stops at 259 characters, the shell at about 2,000): the link goes straight to the
/// program that opens that kind of link, as its argument.
pub fn open_url(url: &str) -> Result<(), String> {
    let scheme = url.split("://").next().unwrap_or_default().to_ascii_lowercase();
    if url.len() <= 250 {
        return open_short(url);
    }
    if let Some((exe, args)) = protocol_command(&scheme, url) {
        if run_as_user(&exe, &args).is_ok() {
            return Ok(());
        }
    }
    if scheme == "https" {
        return open_via_page(url);
    }
    Err("That link is too long for the app that opens it.".into())
}

fn open_short(url: &str) -> Result<(), String> {
    // Running as admin: let Explorer open it so the browser doesn't inherit admin rights.
    if is_elevated() && shell_execute(None, "explorer.exe", &format!("\"{url}\""), None).is_ok() {
        return Ok(());
    }
    shell_execute(None, url, "", None).map_err(|e| format!("Windows couldn't open it (error {e})"))
}

/// True when some app has registered this kind of link (claude://, …).
pub fn has_protocol(scheme: &str) -> bool {
    assoc_command(scheme).is_some()
}

fn assoc_command(scheme: &str) -> Option<String> {
    unsafe {
        let assoc = wide(scheme);
        let verb = wide("open");
        let flags = ASSOCF_IS_PROTOCOL | ASSOCF_NOTRUNCATE;
        let mut len = 0u32;
        AssocQueryStringW(flags, ASSOCSTR_COMMAND, assoc.as_ptr(), verb.as_ptr(), null_mut(), &mut len);
        if len == 0 || len > 32_768 {
            return None;
        }
        let mut buf = vec![0u16; len as usize + 1];
        let mut cap = buf.len() as u32;
        if AssocQueryStringW(flags, ASSOCSTR_COMMAND, assoc.as_ptr(), verb.as_ptr(), buf.as_mut_ptr(), &mut cap) < 0 {
            return None;
        }
        let s = from_wide(&buf);
        (!s.trim().is_empty()).then_some(s)
    }
}

/// The program Windows runs for a kind of link, with this link put in its place:
/// ("C:\…\chrome.exe", "--single-argument https://…"). None for links a Store app handles.
pub fn protocol_command(scheme: &str, url: &str) -> Option<(String, String)> {
    split_command(&assoc_command(scheme)?, url)
}

fn split_command(cmd: &str, url: &str) -> Option<(String, String)> {
    let (exe, rest) = catkey_core::shellcmd::split(cmd)?;
    let exe = expand_env(&exe);
    if file_name_lower(&exe) == "rundll32.exe" || !Path::new(&exe).is_file() {
        return None;
    }
    Some((exe, catkey_core::shellcmd::fill(&rest, url)))
}

/// Start a program as the signed-in user, never as administrator (even when Cat Key is).
pub fn run_as_user(exe: &str, args: &str) -> Result<(), String> {
    if is_elevated() {
        return shell_execute_via_explorer(exe, args, "");
    }
    shell_execute(None, exe, args, None).map_err(|e| format!("Windows couldn't open it (error {e})"))
}

/// ShellExecute run by Explorer itself, so what starts runs as the user: the way Raymond Chen
/// describes ("How can I launch an unelevated process from my elevated process?").
fn shell_execute_via_explorer(file: &str, args: &str, dir: &str) -> Result<(), String> {
    use windows::core::{Interface, BSTR};
    use windows::Win32::System::Com::{CoCreateInstance, IDispatch, IServiceProvider, CLSCTX_ALL};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::Shell::{
        IShellBrowser, IShellDispatch2, IShellFolderViewDual, IShellView, IShellWindows, ShellWindows, CSIDL_DESKTOP,
        SID_STopLevelBrowser, SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH,
    };
    ensure_com();
    let run = || -> windows::core::Result<()> {
        unsafe {
            let wins: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL)?;
            let loc = VARIANT::from(CSIDL_DESKTOP as i32);
            let root = VARIANT::default();
            let mut hwnd = 0i32;
            let disp: IDispatch = wins.FindWindowSW(&loc, &root, SWC_DESKTOP, &mut hwnd, SWFO_NEEDDISPATCH)?;
            let sp: IServiceProvider = disp.cast()?;
            let browser: IShellBrowser = sp.QueryService(&SID_STopLevelBrowser)?;
            let view: IShellView = browser.QueryActiveShellView()?;
            let folder: IShellFolderViewDual = view.GetItemObject(SVGIO_BACKGROUND)?;
            let shell: IShellDispatch2 = folder.Application()?.cast()?;
            shell.ShellExecute(&BSTR::from(file), &VARIANT::from(BSTR::from(args)), &VARIANT::from(BSTR::from(dir)), &VARIANT::from(""), &VARIANT::from(1i32))
        }
    };
    run().map_err(|e| format!("Explorer couldn't open it ({e})"))
}

/// Last resort for a long https link: a tiny page on disk that forwards to it
/// (its path is short, and the browser takes links of any length).
fn open_via_page(url: &str) -> Result<(), String> {
    let dir = std::env::temp_dir().join("CatKey");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("open-link.html");
    let attr = url.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;");
    let js = serde_json::to_string(url).map_err(|e| e.to_string())?;
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Opening…</title><meta http-equiv=\"refresh\" content=\"0;url={attr}\"><script>location.replace({js})</script><p style=\"font:16px system-ui\"><a href=\"{attr}\">Open the link</a></p>"
    );
    std::fs::write(&path, html).map_err(|e| e.to_string())?;
    launch(&path.to_string_lossy(), "")
}

/// An installed app's Start-menu ID, by its name ("ChatGPT", "Claude"): exact name first, then
/// a name that starts with it. Works for Store apps and ordinary ones alike.
pub fn start_app_id(names: &[&str]) -> Option<String> {
    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | ForEach-Object { $_.Name + \"`t\" + $_.AppID }",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let rows: Vec<(String, String)> =
        text.lines().filter_map(|l| l.split_once('\t')).map(|(n, id)| (n.trim().to_lowercase(), id.trim().to_string())).collect();
    let pick = |ok: &dyn Fn(&str, &str) -> bool| {
        names.iter().find_map(|w| {
            let w = w.to_lowercase();
            rows.iter().find(|(n, id)| !id.is_empty() && ok(n, &w)).map(|(_, id)| id.clone())
        })
    };
    pick(&|n, w| n == w).or_else(|| pick(&|n, w| n.starts_with(w) && !n.contains("uninstall") && !n.contains("code")))
}

/// Start an app by its Start-menu ID (through Explorer, so it runs as the user).
pub fn launch_app_id(id: &str) -> Result<(), String> {
    shell_execute(None, "explorer.exe", &format!("shell:AppsFolder\\{id}"), None).map_err(|e| format!("Windows couldn't open it (error {e})"))
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

/// What's on the clipboard for the Cat Dock's history: text, or copied files.
pub struct ClipData {
    pub text: String,
    pub files: Vec<String>,
}

fn open_clipboard() -> bool {
    use windows_sys::Win32::System::DataExchange::OpenClipboard;
    for _ in 0..8 {
        if unsafe { OpenClipboard(null_mut()) } != 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// The clipboard for the history: `None` when it's empty, or when a password manager (or any
/// app) marked it as private or as not for clipboard history.
pub fn clipboard_read() -> Option<ClipData> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, RegisterClipboardFormatW};
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows_sys::Win32::UI::Shell::DragQueryFileW;
    const CF_UNICODETEXT: u32 = 13;
    const CF_HDROP: u32 = 15;
    unsafe {
        for name in ["ExcludeClipboardContentFromMonitorProcessing", "Clipboard Viewer Ignore"] {
            let f = RegisterClipboardFormatW(wide(name).as_ptr());
            if f != 0 && IsClipboardFormatAvailable(f) != 0 {
                return None;
            }
        }
        let has_files = IsClipboardFormatAvailable(CF_HDROP) != 0;
        let has_text = IsClipboardFormatAvailable(CF_UNICODETEXT) != 0;
        if !has_files && !has_text {
            return None;
        }
        let history = RegisterClipboardFormatW(wide("CanIncludeInClipboardHistory").as_ptr());
        if !open_clipboard() {
            return None;
        }
        let mut private = false;
        if history != 0 && IsClipboardFormatAvailable(history) != 0 {
            let h = GetClipboardData(history);
            let p = if h.is_null() { null_mut() } else { GlobalLock(h as _) as *const u32 };
            if !p.is_null() {
                private = *p == 0;
                GlobalUnlock(h as _);
            }
        }
        let mut out = ClipData { text: String::new(), files: Vec::new() };
        if !private && has_files {
            let h = GetClipboardData(CF_HDROP);
            if !h.is_null() {
                let n = DragQueryFileW(h as _, u32::MAX, null_mut(), 0);
                for i in 0..n.min(500) {
                    let len = DragQueryFileW(h as _, i, null_mut(), 0) as usize;
                    let mut buf = vec![0u16; len + 1];
                    DragQueryFileW(h as _, i, buf.as_mut_ptr(), buf.len() as u32);
                    out.files.push(String::from_utf16_lossy(&buf[..len]));
                }
            }
        } else if !private && has_text {
            let h = GetClipboardData(CF_UNICODETEXT);
            let p = if h.is_null() { null_mut() } else { GlobalLock(h as _) as *const u16 };
            if !p.is_null() {
                let max = (GlobalSize(h as _) / 2).min(4_000_000);
                let mut len = 0usize;
                while len < max && *p.add(len) != 0 {
                    len += 1;
                }
                out.text = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                GlobalUnlock(h as _);
            }
        }
        CloseClipboard();
        (!private && (!out.text.is_empty() || !out.files.is_empty())).then_some(out)
    }
}

/// Put bytes on the clipboard in one format (the clipboard owns them from then on).
unsafe fn put_clipboard(format: u32, bytes: &[u8]) -> bool {
    use windows_sys::Win32::System::DataExchange::SetClipboardData;
    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
    if h.is_null() {
        return false;
    }
    let p = GlobalLock(h) as *mut u8;
    if p.is_null() {
        GlobalFree(h);
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    GlobalUnlock(h);
    if SetClipboardData(format, h as _).is_null() {
        GlobalFree(h);
        return false;
    }
    true
}

pub fn clipboard_write_text(text: &str) -> bool {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard};
    let w: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let bytes: Vec<u8> = w.iter().flat_map(|c| c.to_le_bytes()).collect();
    unsafe {
        if !open_clipboard() {
            return false;
        }
        EmptyClipboard();
        let ok = put_clipboard(13, &bytes);
        CloseClipboard();
        ok
    }
}

/// Copied files, the way Explorer copies them; `cut` makes Paste move them instead.
pub fn clipboard_write_files(paths: &[String], cut: bool) -> bool {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, RegisterClipboardFormatW};
    // DROPFILES: where the list starts, a point, "not client area", "wide characters".
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend(20u32.to_le_bytes());
    bytes.extend([0u8; 8]);
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    for p in paths {
        for c in p.encode_utf16().chain(Some(0)) {
            bytes.extend(c.to_le_bytes());
        }
    }
    bytes.extend([0u8, 0u8]);
    unsafe {
        if !open_clipboard() {
            return false;
        }
        EmptyClipboard();
        let ok = put_clipboard(15, &bytes);
        let effect = RegisterClipboardFormatW(wide("Preferred DropEffect").as_ptr());
        if ok && effect != 0 {
            put_clipboard(effect, &(if cut { 2u32 } else { 1u32 }).to_le_bytes());
        }
        CloseClipboard();
        ok
    }
}

/// Whether the (main) mouse button is down right now.
pub fn mouse_down() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SWAPBUTTON};
    let vk = if unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0 { 0x02 } else { 0x01 };
    (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
}

/// Open a file with its usual app.
pub fn open_path(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("That file isn't there any more.".into());
    }
    launch(path, "")
}

/// Show a file in its folder, selected.
pub fn reveal_path(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("That file isn't there any more.".into());
    }
    Command::new("explorer.exe").raw_arg(format!("/select,\"{path}\"")).spawn().map(|_| ()).map_err(|e| e.to_string())
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

// ------------------------------------------------------------------- the app switcher

use crate::switcher::{Screen, SwitchItem};
use catkey_core::switcher::Rect as SwRect;

/// Where the pointer is, in screen pixels.
pub fn pointer() -> Option<(f64, f64)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut p = POINT { x: 0, y: 0 };
    (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x as f64, p.y as f64))
}

/// The screen under a point: all of it, the part the taskbar leaves free, and its scale.
pub fn screen_at(x: f64, y: f64) -> Option<Screen> {
    use windows_sys::Win32::Foundation::{POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONULL};
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    unsafe {
        let m = MonitorFromPoint(POINT { x: x.round() as i32, y: y.round() as i32 }, MONITOR_DEFAULTTONULL);
        if m.is_null() {
            return None;
        }
        let mut mi: MONITORINFO = zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(m, &mut mi) == 0 {
            return None;
        }
        let (mut dx, mut dy) = (96u32, 96u32);
        let scale = if GetDpiForMonitor(m, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) == 0 && dx > 0 { dx as f64 / 96.0 } else { 1.0 };
        let r = |r: RECT| SwRect::new(r.left as f64, r.top as f64, (r.right - r.left) as f64, (r.bottom - r.top) as f64);
        Some(Screen { full: r(mi.rcMonitor), work: r(mi.rcWork), scale })
    }
}

unsafe fn class_name(hwnd: HWND) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut buf = [0u16; 128];
    let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

unsafe fn title_of(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    from_wide(&buf)
}

/// App names by program path ("Google Chrome"), read once: the switcher opens often.
static APP_NAMES: Mutex<Option<std::collections::HashMap<String, String>>> = Mutex::new(None);

fn app_name_of(path: &str, exe: &str) -> String {
    if let Some(n) = APP_NAMES.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get(path).cloned())) {
        return n;
    }
    let name = file_description(path)
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| exe.trim_end_matches(".exe").to_string());
    if let Ok(mut g) = APP_NAMES.lock() {
        g.get_or_insert_with(Default::default).insert(path.to_string(), name.clone());
    }
    name
}

/// The windows you can switch to, front to back (the order Alt+Tab shows them in).
pub fn switch_windows() -> Vec<SwitchItem> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, IsIconic, WS_EX_APPWINDOW, WS_EX_NOACTIVATE};
    let mut hwnds: Vec<usize> = Vec::new();
    unsafe {
        EnumWindows(Some(collect_windows), &mut hwnds as *mut _ as LPARAM);
    }
    let me = unsafe { GetCurrentProcessId() };
    let front = unsafe { GetForegroundWindow() } as usize;
    let mut out: Vec<SwitchItem> = Vec::new();
    for h in hwnds {
        let hwnd = h as HWND;
        unsafe {
            if IsWindowVisible(hwnd) == 0 {
                continue;
            }
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            if ex & WS_EX_APPWINDOW == 0 && (ex & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) != 0 || !GetWindow(hwnd, GW_OWNER).is_null()) {
                continue;
            }
            // Windows on another desktop (and Store apps' hidden frames) are cloaked.
            let mut cloaked: u32 = 0;
            if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED as u32, &mut cloaked as *mut _ as *mut c_void, 4) == 0 && cloaked != 0 {
                continue;
            }
            let class = class_name(hwnd);
            if matches!(
                class.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" | "Windows.UI.Core.CoreWindow" | "XamlExplorerHostIslandWindow"
            ) {
                continue;
            }
            let min = IsIconic(hwnd) != 0;
            if !min {
                let mut r: RECT = zeroed();
                if GetWindowRect(hwnd, &mut r) == 0 || r.right - r.left < 8 || r.bottom - r.top < 8 {
                    continue;
                }
            }
            let title = title_of(hwnd);
            if title.trim().is_empty() {
                continue;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == me {
                continue;
            }
            let (path, exe) = match process_path(pid) {
                Some(p0) => {
                    let exe0 = file_name_lower(&p0);
                    let rp = real_pid(hwnd, pid, &exe0);
                    let p = if rp != pid { process_path(rp).unwrap_or(p0) } else { p0 };
                    let exe = file_name_lower(&p);
                    (p, exe)
                }
                None => (String::new(), String::new()),
            };
            // A Store app that's asleep: only its frame is left, so it goes by its title.
            let hosted = exe.is_empty() || exe == "applicationframehost.exe";
            let app = if hosted { title.trim().to_string() } else { app_name_of(&path, &exe) };
            out.push(SwitchItem {
                id: h.to_string(),
                title: title.trim().to_string(),
                app,
                exe: if hosted { String::new() } else { exe },
                path: if hosted { String::new() } else { path },
                front: h == front,
                min,
                can_close: true,
            });
        }
    }
    out
}

/// Bring a window to the front (back from the taskbar if it was minimised).
pub fn switch_to(id: &str, _reopen: bool) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_MOUSE};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, IsIconic, IsWindow, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };
    let Ok(h) = id.parse::<usize>() else { return false };
    let hwnd = h as HWND;
    unsafe {
        if IsWindow(hwnd) == 0 {
            return false;
        }
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        // An empty mouse event first: Windows then lets us hand the front over.
        let mut input: INPUT = zeroed();
        input.r#type = INPUT_MOUSE;
        SendInput(1, &input, size_of::<INPUT>() as i32);
        SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
        GetForegroundWindow() == hwnd
    }
}

/// Ask a window to close (it can still ask to save first).
pub fn switch_close(id: &str) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindow, PostMessageW, WM_CLOSE};
    let Ok(h) = id.parse::<usize>() else { return false };
    let hwnd = h as HWND;
    unsafe { IsWindow(hwnd) != 0 && PostMessageW(hwnd, WM_CLOSE, 0, 0) != 0 }
}

/// A window's own icon, for Store apps whose program has none (PNG data URL).
pub fn window_icon(id: &str) -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassLongPtrW, SendMessageTimeoutW, GCLP_HICON, ICON_BIG, SMTO_ABORTIFHUNG, WM_GETICON};
    let hwnd = id.parse::<usize>().ok()? as HWND;
    unsafe {
        let mut res: usize = 0;
        SendMessageTimeoutW(hwnd, WM_GETICON, ICON_BIG as usize, 0, SMTO_ABORTIFHUNG, 80, &mut res);
        let icon = if res != 0 { res as HICON } else { GetClassLongPtrW(hwnd, GCLP_HICON) as HICON };
        if icon.is_null() {
            return None;
        }
        // The window owns this icon: don't destroy it.
        hicon_to_png(icon).map(|b| format!("data:image/png;base64,{}", base64(&b)))
    }
}

/// Cat Key runs from a temporary folder (opened straight from a zip, say): what points at
/// it, like an AI app's connection, would break when that folder is cleaned up.
pub fn running_from_temp_place() -> bool {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_lowercase()).unwrap_or_default();
    let tmp = std::env::temp_dir().to_string_lossy().to_lowercase();
    !tmp.is_empty() && exe.starts_with(tmp.trim_end_matches('\\'))
}
