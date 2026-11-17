//! The icon in the notification area. With it on, minimizing or closing the window hides
//! Mantel there instead of on the taskbar; the icon brings it back, its menu quits.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

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

/// Hides the window in the tray. Its taskbar button goes first: a hidden window that Windows
/// restores minimized (as it can after a game) would otherwise come back on the taskbar.
fn to_tray(w: &tauri::WebviewWindow) {
    let _ = w.set_skip_taskbar(true);
    let _ = w.hide();
    away(w, true);
}

/// Tells the interface whether Mantel is out of sight, in the tray or minimized: nothing like
/// a trailer should play then. WebView2 doesn't notice a hidden window by itself.
pub fn away(w: &tauri::WebviewWindow, gone: bool) {
    let _ = w.emit("window-away", gone);
}

/// Out of the way while a game runs: into the tray when it's on, else minimized.
pub fn step_aside(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    if ON.load(Ordering::Relaxed) {
        to_tray(&w);
    } else {
        let _ = w.minimize();
        away(&w, true);
    }
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
        let tray = ON.load(Ordering::Relaxed);
        match e {
            WindowEvent::CloseRequested { api, .. } if tray => {
                api.prevent_close();
                to_tray(&w);
            }
            WindowEvent::Resized(_) => {
                let minimized = w.is_minimized().unwrap_or(false);
                if minimized && tray {
                    to_tray(&w);
                } else {
                    // Minimized to the taskbar, or brought back from it.
                    away(&w, minimized);
                }
            }
            _ => {}
        }
    });
}
