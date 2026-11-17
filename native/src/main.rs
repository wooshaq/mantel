// No console window in release builds; they log to mantel.log instead.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Backend functions the later stages of the native interface will call.
#![allow(dead_code)]

mod accounts;
mod apps;
mod art;
mod core;
mod demo;
mod fse;
mod gamepass;
mod host;
mod i18n;
mod icons;
mod images;
mod install;
mod launcher;
mod library;
mod log;
mod migrate;
mod model;
mod pad;
mod pick;
mod steam_install;
mod steam_store;
mod store;
mod system;
mod ui;
mod uninstall;
mod vdf;
mod xbox_store;

use std::path::PathBuf;
use std::sync::OnceLock;

slint::include_modules!();

/// The same folder as the web build's, so the library, playtime and settings carry over.
const IDENTIFIER: &str = "com.playmantel.launcher";

/// One runtime for every network call and background job, shared by the whole app.
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().enable_all().worker_threads(2).build().expect("tokio runtime"))
}

fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("MANTEL_DATA") {
        return dir.into();
    }
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share"));
    base.unwrap_or_else(std::env::temp_dir).join(IDENTIFIER)
}

/// A second Mantel only brings the first one to the front.
#[cfg(windows)]
fn already_running() -> bool {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};
    let name: Vec<u16> = "Local\\MantelWsQ".encode_utf16().chain(Some(0)).collect();
    unsafe {
        CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return false;
        }
        let title: Vec<u16> = "Mantel".encode_utf16().chain(Some(0)).collect();
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
    true
}

#[cfg(not(windows))]
fn already_running() -> bool {
    false
}

fn main() {
    if already_running() {
        return;
    }
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let version = env!("CARGO_PKG_VERSION");
    log::init(&dir, version);
    log::info("native build (Slint)");
    let app = host::AppHandle::new(dir.clone(), version);
    let state = store::AppState::load(dir.clone());
    let language = state.user.lock().unwrap().settings.language.clone();
    i18n::set_language(&language);
    images::init(dir.join("imagecache"));
    app.manage(state);
    let demo = std::env::var_os("MANTEL_DEMO").map(|_| demo::games());
    if demo.is_none() {
        launcher::spawn_tracker(app.clone());
        launcher::watch_front(&app);
    }
    if let Err(e) = ui::run(app, demo) {
        log::warn(&format!("window: {e}"));
    }
}
