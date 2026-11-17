//! Windows' full screen experience (Xbox mode): no desktop or taskbar. Mantel only reads whether
//! it is there and switches it on or off from the control center. This build (wooshaq) never makes
//! itself the home app: no package registration, no registry writes, no Developer Mode.
//! The API exists only on builds that have the mode, so it is looked up at run time.

#[cfg(windows)]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
fn proc(name: &[u8]) -> windows_sys::Win32::Foundation::FARPROC {
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};
    let dll = wide("api-ms-win-gaming-experience-l1-1-0.dll");
    let module = unsafe { LoadLibraryExW(dll.as_ptr(), std::ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32) };
    if module.is_null() {
        return None;
    }
    unsafe { GetProcAddress(module, name.as_ptr()) }
}

#[cfg(windows)]
fn flag(name: &[u8]) -> bool {
    type Get = unsafe extern "system" fn() -> i32;
    proc(name).is_some_and(|f| unsafe { std::mem::transmute::<_, Get>(f)() != 0 })
}

#[cfg(windows)]
pub fn supported() -> bool {
    flag(b"IsGamingFullScreenExperienceSupported\0")
}

#[cfg(windows)]
pub fn active() -> bool {
    flag(b"IsGamingFullScreenExperienceActive\0")
}

#[cfg(windows)]
pub fn can_switch() -> bool {
    flag(b"CanSetGamingFullScreenExperience\0")
}

/// Turns the mode on or off, as Game Bar and the mode's own "Switch to desktop" do.
#[cfg(windows)]
pub fn switch(on: bool) -> bool {
    type Set = unsafe extern "system" fn(i32) -> i32;
    can_switch() && proc(b"SetGamingFullScreenExperience\0").is_some_and(|f| unsafe { std::mem::transmute::<_, Set>(f)(on as i32) >= 0 })
}

/// Whether this process runs with a package identity (logged at start only).
#[cfg(windows)]
pub fn packaged() -> bool {
    use windows_sys::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    let mut len = 0;
    unsafe { GetCurrentPackageFullName(&mut len, std::ptr::null_mut()) != APPMODEL_ERROR_NO_PACKAGE }
}

#[cfg(not(windows))]
pub fn supported() -> bool {
    false
}

#[cfg(not(windows))]
pub fn active() -> bool {
    false
}

#[cfg(not(windows))]
pub fn can_switch() -> bool {
    false
}

#[cfg(not(windows))]
pub fn switch(_on: bool) -> bool {
    false
}

#[cfg(not(windows))]
pub fn packaged() -> bool {
    false
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    pub supported: bool,
    pub active: bool,
    pub can_switch: bool,
}

pub fn mode() -> Mode {
    Mode { supported: supported(), active: active(), can_switch: can_switch() }
}
