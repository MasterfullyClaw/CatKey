//! See-through windows (the CatGPT wheel, the Dock, tips, the cat walk…) clip themselves
//! to what they show. On some Windows PCs the graphics driver draws a see-through window's
//! empty parts as a solid box; clipped, nothing around the cat or the wheel can show, and
//! clicks between the petals reach the app underneath. On a Mac there's nothing to do.

use serde::Deserialize;
use tauri::WebviewWindow;

/// A shape in CSS pixels from the window's top-left: a rounded box, or an ellipse when the
/// corner radius is half the smaller side or more.
#[derive(Deserialize, Debug, Clone, Copy)]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct Shape {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    #[serde(default)]
    r: f64,
}

/// Clip the calling window to these shapes; none: nothing shows (nothing to show).
#[tauri::command]
pub fn shape_window(window: WebviewWindow, shapes: Vec<Shape>) {
    #[cfg(windows)]
    win::apply(&window, &shapes);
    #[cfg(not(windows))]
    let _ = (window, shapes);
}

#[cfg(windows)]
mod win {
    use super::Shape;
    use std::sync::Mutex;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE};
    use windows_sys::Win32::Graphics::Gdi::{CombineRgn, CreateEllipticRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, HRGN, RGN_OR};

    /// Windows we've already told not to round their corners or draw a border.
    static PLAIN: Mutex<Vec<isize>> = Mutex::new(Vec::new());

    pub fn apply(window: &tauri::WebviewWindow, shapes: &[Shape]) {
        let Ok(h) = window.hwnd() else { return };
        let hwnd = h.0 as HWND;
        let k = window.scale_factor().unwrap_or(1.0);
        unsafe {
            if plain(hwnd) {
                crate::log(&format!("shape: {} cut to {} parts", window.label(), shapes.len()));
            }
            let all: HRGN = CreateRectRgn(0, 0, 0, 0);
            for s in shapes {
                if !(s.w > 0.5 && s.h > 0.5) {
                    continue;
                }
                let l = (s.x * k).floor() as i32;
                let t = (s.y * k).floor() as i32;
                let r = ((s.x + s.w) * k).ceil() as i32 + 1;
                let b = ((s.y + s.h) * k).ceil() as i32 + 1;
                let one = if s.r * 2.0 >= s.w.min(s.h) - 0.5 {
                    CreateEllipticRgn(l, t, r, b)
                } else {
                    let d = (s.r * 2.0 * k).round().max(0.0) as i32;
                    CreateRoundRectRgn(l, t, r, b, d, d)
                };
                CombineRgn(all, all, one, RGN_OR);
                DeleteObject(one);
            }
            // Windows owns the region from here on.
            SetWindowRgn(hwnd, all, 1);
        }
    }

    /// Windows 11 rounds the corners of windows and may draw a thin border on them; a
    /// see-through window wants neither. True the first time for a window.
    unsafe fn plain(hwnd: HWND) -> bool {
        let key = hwnd as isize;
        let mut done = PLAIN.lock().unwrap_or_else(|e| e.into_inner());
        if done.contains(&key) {
            return false;
        }
        let corner: u32 = 1; // DWMWCP_DONOTROUND
        let none: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE
        DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE as u32, &corner as *const u32 as *const _, 4);
        DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR as u32, &none as *const u32 as *const _, 4);
        done.push(key);
        true
    }
}
