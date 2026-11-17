//! What the backend used to get from Tauri, for the native (Slint) build: the app's shared state,
//! events to the interface, its folders and version, and the main window. The names stay the
//! same (`AppHandle`, `state()`, `emit()`), so the backend modules carry over almost unchanged.

use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;

type Listener = Box<dyn Fn(&str, serde_json::Value) + Send + Sync>;

struct Inner {
    state: OnceLock<Box<dyn Any + Send + Sync>>,
    listener: Mutex<Option<Listener>>,
    data_dir: PathBuf,
    version: String,
    /// The main window's HWND once it exists, for showing and minimizing it from any thread.
    hwnd: AtomicIsize,
}

#[derive(Clone)]
pub struct AppHandle(Arc<Inner>);

impl AppHandle {
    pub fn new(data_dir: PathBuf, version: &str) -> Self {
        AppHandle(Arc::new(Inner { state: OnceLock::new(), listener: Mutex::new(None), data_dir, version: version.to_string(), hwnd: AtomicIsize::new(0) }))
    }

    /// Puts the app's one piece of shared state in place; `state()` hands it out afterwards.
    pub fn manage<T: Send + Sync + 'static>(&self, state: T) {
        let _ = self.0.state.set(Box::new(state));
    }

    /// Where events go: the interface sets this once its window exists.
    pub fn listen(&self, f: impl Fn(&str, serde_json::Value) + Send + Sync + 'static) {
        *self.0.listener.lock().unwrap() = Some(Box::new(f));
    }

    pub fn set_hwnd(&self, hwnd: isize) {
        self.0.hwnd.store(hwnd, Ordering::Relaxed);
    }

    pub fn hwnd(&self) -> isize {
        self.0.hwnd.load(Ordering::Relaxed)
    }

    pub fn path(&self) -> Paths<'_> {
        Paths(self)
    }

    pub fn package_info(&self) -> PackageInfo {
        PackageInfo { version: self.0.version.clone() }
    }

    /// Quits the app ("Quit Mantel" in the control center).
    pub fn exit(&self, code: i32) {
        crate::log::info("quit");
        std::process::exit(code);
    }

    /// Kept for the code that asked Tauri for a handle to itself.
    pub fn handle(&self) -> &AppHandle {
        self
    }
}

pub trait Manager {
    fn state<T: Send + Sync + 'static>(&self) -> &T;
    fn get_webview_window(&self, label: &str) -> Option<MainWindow>;
}

impl Manager for AppHandle {
    fn state<T: Send + Sync + 'static>(&self) -> &T {
        self.0.state.get().and_then(|s| s.downcast_ref::<T>()).expect("app state not managed yet")
    }

    /// Only "main" exists in the native build.
    fn get_webview_window(&self, label: &str) -> Option<MainWindow> {
        (label == "main" && self.hwnd() != 0).then(|| MainWindow(self.hwnd()))
    }
}

pub trait Emitter {
    fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String>;
}

impl Emitter for AppHandle {
    fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String> {
        let value = serde_json::to_value(payload).map_err(|e| e.to_string())?;
        if let Some(f) = self.0.listener.lock().unwrap().as_ref() {
            f(event, value);
        }
        Ok(())
    }
}

pub struct Paths<'a>(&'a AppHandle);

impl Paths<'_> {
    pub fn app_data_dir(&self) -> Result<PathBuf, String> {
        Ok(self.0 .0.data_dir.clone())
    }

    /// Next to the exe, where an installer puts the files it ships.
    pub fn resource_dir(&self) -> Result<PathBuf, String> {
        std::env::current_exe().map_err(|e| e.to_string())?.parent().map(PathBuf::from).ok_or_else(|| "no exe folder".into())
    }
}

pub struct PackageInfo {
    pub version: String,
}

/// The main window, by its HWND: Win32 calls work from any thread, unlike the toolkit's own.
pub struct MainWindow(isize);

impl MainWindow {
    pub fn hwnd(&self) -> Result<Hwnd, String> {
        Ok(Hwnd(self.0 as *mut std::ffi::c_void))
    }

    pub fn minimize(&self) -> Result<(), String> {
        self.show_cmd(6) // SW_MINIMIZE
    }

    pub fn unminimize(&self) -> Result<(), String> {
        self.show_cmd(9) // SW_RESTORE
    }

    pub fn show(&self) -> Result<(), String> {
        self.show_cmd(5) // SW_SHOW
    }

    pub fn set_focus(&self) -> Result<(), String> {
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(self.0 as _);
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn show_cmd(&self, cmd: i32) -> Result<(), String> {
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(self.0 as _, cmd);
        }
        Ok(())
    }
}

/// Matches Tauri's `hwnd()` result, whose `.0` is the raw handle.
pub struct Hwnd(pub *mut std::ffi::c_void);
