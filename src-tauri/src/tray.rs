//! The icon in the notification area. With it on, minimizing or closing the window hides
//! Mantel there instead of on the taskbar; the icon brings it back, its menu quits.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

use crate::store::AppState;

const ID: &str = "mantel";

/// Whether the icon is up; read on every window event, so kept out of the settings lock.
static ON: AtomicBool = AtomicBool::new(false);

/// Puts the icon up or takes it down, with menu labels in the interface's language.
pub fn set(app: &AppHandle, on: bool, show: &str, quit: &str) -> Result<(), String> {
    ON.store(on, Ordering::Relaxed);
    let _ = app.remove_tray_by_id(ID);
    if !on {
        // A hidden window has nothing left to bring it back.
        crate::launcher::to_front(app);
        return Ok(());
    }
    let show = MenuItem::with_id(app, "tray-show", show, true, None::<&str>).map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "tray-quit", quit, true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&show, &quit]).map_err(|e| e.to_string())?;
    let mut builder = TrayIconBuilder::with_id(ID)
        .tooltip("Mantel")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id().as_ref() {
            "tray-show" => crate::launcher::to_front(app),
            "tray-quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|icon, e| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                crate::launcher::to_front(icon.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app).map_err(|e| e.to_string())?;
    Ok(())
}

/// Shows the icon at startup when the settings ask for it; the interface relabels it.
pub fn init(app: &AppHandle) {
    let on = app.state::<AppState>().user.lock().unwrap().settings.tray;
    if on {
        if let Err(e) = set(app, true, "Show Mantel", "Quit Mantel") {
            crate::log::warn(&format!("tray: {e}"));
        }
    }
    let Some(window) = app.get_webview_window("main") else { return };
    let w = window.clone();
    window.on_window_event(move |e| {
        if !ON.load(Ordering::Relaxed) {
            return;
        }
        match e {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = w.hide();
            }
            WindowEvent::Resized(_) if w.is_minimized().unwrap_or(false) => {
                let _ = w.hide();
            }
            _ => {}
        }
    });
}
