//! Windows: what's dropped on the Cat Dock (files, text, links). Like wry's file-drop
//! handler, this takes the place of WebView2's own drop target on the Dock's window, but it
//! also takes text and links.

use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller4;
use windows::core::{implement, Interface, Ref, BOOL};
use windows::Win32::Foundation::{DRAGDROP_E_INVALIDHWND, HWND, LPARAM, POINTL, S_OK};
use windows::Win32::System::Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, OleInitialize, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop, CF_HDROP, CF_UNICODETEXT,
    DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
};

thread_local! {
    /// The targets must live as long as their windows (on the main thread, where OLE calls them).
    static TARGETS: RefCell<Vec<(isize, IDropTarget)>> = const { RefCell::new(Vec::new()) };
}

#[implement(IDropTarget)]
struct DockTarget {
    ok: Cell<bool>,
}

fn format(cf: u16) -> FORMATETC {
    FORMATETC { cfFormat: cf, ptd: null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: -1, tymed: TYMED_HGLOBAL.0 as u32 }
}

unsafe fn offers(obj: &IDataObject, cf: u16) -> bool {
    obj.QueryGetData(&format(cf)) == S_OK
}

/// The dropped files, or else the dropped text.
unsafe fn read(obj: &IDataObject) -> (Vec<String>, String) {
    let mut paths = Vec::new();
    if let Ok(mut m) = obj.GetData(&format(CF_HDROP.0)) {
        let h = HDROP(m.u.hGlobal.0 as _);
        let n = DragQueryFileW(h, u32::MAX, None);
        for i in 0..n.min(500) {
            let len = DragQueryFileW(h, i, None) as usize;
            let mut buf = vec![0u16; len + 1];
            DragQueryFileW(h, i, Some(&mut buf));
            paths.push(String::from_utf16_lossy(&buf[..len]));
        }
        ReleaseStgMedium(&mut m);
    }
    let mut text = String::new();
    if paths.is_empty() {
        if let Ok(mut m) = obj.GetData(&format(CF_UNICODETEXT.0)) {
            let g = m.u.hGlobal;
            let p = GlobalLock(g) as *const u16;
            if !p.is_null() {
                let max = GlobalSize(g) / 2;
                let mut len = 0usize;
                while len < max && *p.add(len) != 0 {
                    len += 1;
                }
                text = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                let _ = GlobalUnlock(g);
            }
            ReleaseStgMedium(&mut m);
        }
    }
    (paths, text)
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for DockTarget_Impl {
    fn DragEnter(&self, obj: Ref<'_, IDataObject>, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows::core::Result<()> {
        let ok = obj.as_ref().is_some_and(|o| unsafe { offers(o, CF_HDROP.0) || offers(o, CF_UNICODETEXT.0) });
        self.ok.set(ok);
        unsafe { *effect = if ok { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
        if ok {
            std::thread::spawn(|| crate::dock::drag_phase("enter"));
        }
        Ok(())
    }

    fn DragOver(&self, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows::core::Result<()> {
        unsafe { *effect = if self.ok.get() { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        if self.ok.get() {
            std::thread::spawn(|| crate::dock::drag_phase("leave"));
        }
        Ok(())
    }

    fn Drop(&self, obj: Ref<'_, IDataObject>, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows::core::Result<()> {
        if !self.ok.get() {
            unsafe { *effect = DROPEFFECT_NONE };
            return Ok(());
        }
        let Some(o) = obj.as_ref() else { return Ok(()) };
        // The data is only there during this call: read it now, handle it on another thread.
        let (paths, text) = unsafe { read(o) };
        unsafe { *effect = DROPEFFECT_COPY };
        std::thread::spawn(move || {
            crate::dock::dropped(paths, text);
            crate::dock::drag_phase("drop");
        });
        Ok(())
    }
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = &mut *(lparam.0 as *mut Vec<HWND>);
    list.push(hwnd);
    BOOL(1)
}

/// Make the Dock's window a tool window (no taskbar button, not in Alt+Tab) and catch drops.
pub fn attach(w: &tauri::WebviewWindow) {
    if let Ok(h) = w.hwnd() {
        let hwnd = HWND(h.0);
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_TOOLWINDOW.0 as isize);
        }
    }
    let _ = w.with_webview(|pw| unsafe {
        let _ = OleInitialize(None);
        let controller = pw.controller();
        // WebView2 must leave outside drops to us.
        if let Ok(c4) = controller.cast::<ICoreWebView2Controller4>() {
            let _ = c4.SetAllowExternalDrop(false);
        }
        let mut parent = HWND::default();
        if controller.ParentWindow(&mut parent).is_err() || parent.is_invalid() {
            return;
        }
        let mut kids: Vec<HWND> = Vec::new();
        let _ = EnumChildWindows(Some(parent), Some(collect), LPARAM(&mut kids as *mut Vec<HWND> as isize));
        let mut caught = 0;
        for kid in kids {
            let target: IDropTarget = DockTarget { ok: Cell::new(false) }.into();
            if RevokeDragDrop(kid) != Err(DRAGDROP_E_INVALIDHWND.into()) && RegisterDragDrop(kid, &target).is_ok() {
                TARGETS.with(|t| t.borrow_mut().push((kid.0 as isize, target)));
                caught += 1;
            }
        }
        crate::log(&format!("dock: catching drops on {caught} windows"));
    });
}

/// A window that shows things but never takes clicks or the keyboard (the cat walk).
pub fn no_activate(w: &tauri::WebviewWindow) {
    if let Ok(h) = w.hwnd() {
        let hwnd = HWND(h.0);
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0) as isize);
        }
    }
}

/// A window that stays out of Alt+Tab and the taskbar but still takes clicks (the switcher).
pub fn tool_window(w: &tauri::WebviewWindow) {
    if let Ok(h) = w.hwnd() {
        let hwnd = HWND(h.0);
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_TOOLWINDOW.0 as isize);
        }
    }
}
