//! Windows: the cat-paw pointer. An arrow in your cat's fur colour with a little paw print,
//! drawn at the system's pointer size and set for the whole system. The normal pointer comes
//! back when it's turned off, when Cat Key quits, and on the next start after a crash.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows_sys::Win32::Graphics::Gdi::{CreateBitmap, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, GetSystemMetrics, SetSystemCursor, SystemParametersInfoW, ICONINFO, OCR_NORMAL, SM_CXCURSOR, SPI_SETCURSORS,
};

static APPLIED: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn marker() -> std::path::PathBuf {
    crate::app::config_dir().join(".paw-cursor")
}

/// Turn the paw pointer on (in these colours) or off.
pub fn apply(on: bool, colors: &[String]) {
    if on {
        let same = APPLIED.load(Ordering::SeqCst) && LAST.lock().map(|l| *l == colors).unwrap_or(false);
        if same {
            return;
        }
        if set(colors) {
            APPLIED.store(true, Ordering::SeqCst);
            if let Ok(mut l) = LAST.lock() {
                *l = colors.to_vec();
            }
            let _ = std::fs::write(marker(), b"1");
        }
    } else if APPLIED.swap(false, Ordering::SeqCst) || marker().exists() {
        restore();
    }
}

/// The user's own pointers again.
pub fn restore() {
    unsafe { SystemParametersInfoW(SPI_SETCURSORS, 0, std::ptr::null_mut(), 0) };
    let _ = std::fs::remove_file(marker());
    APPLIED.store(false, Ordering::SeqCst);
}

fn hex(s: Option<&String>, fallback: [u8; 3]) -> [u8; 3] {
    let Some(s) = s else { return fallback };
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 {
        return fallback;
    }
    let v = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).ok();
    match (v(0), v(2), v(4)) {
        (Some(r), Some(g), Some(b)) => [r, g, b],
        _ => fallback,
    }
}

/// Signed distance to a polygon (negative inside).
fn poly_sd(p: (f32, f32), pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    let mut inside = false;
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let (wx, wy) = (p.0 - a.0, p.1 - a.1);
        let t = ((wx * ex + wy * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        let (dx, dy) = (wx - ex * t, wy - ey * t);
        d = d.min((dx * dx + dy * dy).sqrt());
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            inside = !inside;
        }
    }
    if inside {
        -d
    } else {
        d
    }
}

fn ellipse_sd(p: (f32, f32), c: (f32, f32), r: (f32, f32)) -> f32 {
    let (x, y) = ((p.0 - c.0) / r.0, (p.1 - c.1) / r.1);
    ((x * x + y * y).sqrt() - 1.0) * r.0.min(r.1)
}

/// The picture, `n`×`n`, as top-down BGRA with straight alpha. Drawn on a 32-unit grid.
fn draw(n: usize, fur: [u8; 3], line: [u8; 3], bean: [u8; 3]) -> Vec<u8> {
    let arrow = [(1.5, 1.5), (1.5, 23.0), (7.0, 17.8), (10.6, 26.4), (14.0, 25.0), (10.4, 16.6), (17.4, 16.6)];
    let s = 32.0 / n as f32;
    let mut out = vec![0u8; n * n * 4];
    const SUB: usize = 4;
    for py in 0..n {
        for px in 0..n {
            let mut acc = [0f32; 4];
            for sy in 0..SUB {
                for sx in 0..SUB {
                    let p = ((px as f32 + (sx as f32 + 0.5) / SUB as f32) * s, (py as f32 + (sy as f32 + 0.5) / SUB as f32) * s);
                    // the paw print (pad and four toes), then the arrow over it
                    let pad = ellipse_sd(p, (23.5, 26.0), (4.6, 3.8));
                    let toes = [(18.6, 21.4), (21.4, 19.0), (25.6, 19.0), (28.4, 21.4)]
                        .iter()
                        .map(|&c| ellipse_sd(p, c, (1.9, 2.2)))
                        .fold(f32::MAX, f32::min);
                    let paw = pad.min(toes);
                    let a = poly_sd(p, &arrow);
                    // A soft white halo keeps it visible on dark backgrounds too.
                    let px_col = if a <= -0.9 {
                        Some((fur, 1.0))
                    } else if a <= 0.7 {
                        Some((line, 1.0))
                    } else if paw <= -0.8 {
                        Some((bean, 1.0))
                    } else if paw <= 0.6 {
                        Some((line, 1.0))
                    } else if a <= 1.8 || paw <= 1.6 {
                        Some(([255, 255, 255], 0.7))
                    } else {
                        None
                    };
                    if let Some((c, w)) = px_col {
                        acc[0] += c[2] as f32 * w;
                        acc[1] += c[1] as f32 * w;
                        acc[2] += c[0] as f32 * w;
                        acc[3] += w;
                    }
                }
            }
            let cov = acc[3] / (SUB * SUB) as f32;
            if cov > 0.0 {
                let i = (py * n + px) * 4;
                out[i] = (acc[0] / acc[3]) as u8;
                out[i + 1] = (acc[1] / acc[3]) as u8;
                out[i + 2] = (acc[2] / acc[3]) as u8;
                out[i + 3] = (cov * 255.0).round() as u8;
            }
        }
    }
    out
}

fn set(colors: &[String]) -> bool {
    let fur = hex(colors.first(), [0xFF, 0xF7, 0xF0]);
    let line = hex(colors.get(1), [0x6B, 0x4A, 0x5E]);
    let bean = hex(colors.get(2), [0xFF, 0xB8, 0xD2]);
    unsafe {
        let n = GetSystemMetrics(SM_CXCURSOR).clamp(32, 128) as usize;
        let pixels = draw(n, fur, line, bean);
        let mut bi: BITMAPINFO = std::mem::zeroed();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = n as i32;
        bi.bmiHeader.biHeight = -(n as i32); // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;
        let dc = GetDC(std::ptr::null_mut());
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let color = CreateDIBSection(dc, &bi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        ReleaseDC(std::ptr::null_mut(), dc);
        if color.is_null() || bits.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
        let mask_bits = vec![0u8; n.div_ceil(16) * 2 * n];
        let mask = CreateBitmap(n as i32, n as i32, 1, 1, mask_bits.as_ptr() as *const _);
        let hot = (1.5 * n as f32 / 32.0).round() as u32;
        let info = ICONINFO { fIcon: 0, xHotspot: hot, yHotspot: hot, hbmMask: mask, hbmColor: color };
        let cursor = CreateIconIndirect(&info);
        DeleteObject(color);
        DeleteObject(mask);
        if cursor.is_null() {
            return false;
        }
        // Windows owns the cursor from here on.
        SetSystemCursor(cursor, OCR_NORMAL) != 0
    }
}
