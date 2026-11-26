//! The tray icon: left-click opens Cat Key, right-click shows the menu.

use crate::app::{self, AppState};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "catkey";

pub struct TrayState {
    pause: MenuItem<Wry>,
    purr: MenuItem<Wry>,
}

fn purr_text() -> &'static str {
    if crate::purr::is_on() {
        "Stop purring"
    } else {
        "Purr in the background"
    }
}

fn look(app: &AppHandle) -> (Image<'static>, String, &'static str) {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap();
    let on = cfg.rules.iter().filter(|r| r.enabled).count();
    if cfg.enabled {
        (
            Image::from_bytes(include_bytes!("../icons/tray-awake.png")).expect("tray icon"),
            format!("Cat Key · {on} shortcuts on"),
            "Pause all shortcuts",
        )
    } else {
        (
            Image::from_bytes(include_bytes!("../icons/tray-nap.png")).expect("tray icon"),
            "Cat Key · napping".to_string(),
            "Wake up (resume shortcuts)",
        )
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let (icon, tip, pause_text) = look(app);
    let open = MenuItem::with_id(app, "open", "Open Cat Key", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", pause_text, true, None::<&str>)?;
    let purr = MenuItem::with_id(app, "purr", purr_text(), true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Cat Key", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &pause, &purr, &sep, &quit])?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(tip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "open" => app::show_main(app),
            "pause" => app::toggle_enabled(app),
            "purr" => {
                let now = crate::purr::set(!crate::purr::is_on(), None);
                let _ = tauri::Emitter::emit(app, "ck://purr", now);
                refresh(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                app::show_main(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(TrayState { pause, purr });
    Ok(())
}

pub fn refresh(app: &AppHandle) {
    let (icon, tip, pause_text) = look(app);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_tooltip(Some(tip));
    }
    if let Some(ts) = app.try_state::<TrayState>() {
        let _ = ts.pause.set_text(pause_text);
        let _ = ts.purr.set_text(purr_text());
    }
}
