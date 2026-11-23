use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Power {
    pub has_battery: bool,
    pub percent: Option<u8>,
    pub charging: bool,
}

#[cfg(windows)]
pub fn power() -> Power {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut s: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    if unsafe { GetSystemPowerStatus(&mut s) } == 0 {
        return Power::default();
    }
    Power {
        // 128 = no system battery, 255 = unknown.
        has_battery: s.BatteryFlag != 128 && s.BatteryFlag != 255,
        percent: (s.BatteryLifePercent <= 100).then_some(s.BatteryLifePercent),
        charging: s.ACLineStatus == 1,
    }
}

#[cfg(not(windows))]
pub fn power() -> Power {
    Power::default()
}

/// What the control center can show and change. Anything the device lacks stays empty.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quick {
    /// 0–100; empty without an output device.
    pub volume: Option<u8>,
    pub muted: bool,
    /// 0–100; empty when the screen can't be dimmed from here, like an external monitor.
    pub brightness: Option<u8>,
    /// Which screens show the picture; empty when Windows doesn't say.
    pub display: Option<crate::display::Mode>,
}


#[cfg(windows)]
pub fn quick() -> Quick {
    let (volume, muted) = controls::volume().map_or((None, false), |(v, m)| (Some(v), m));
    Quick { volume, muted, brightness: controls::brightness(), display: crate::display::mode() }
}

#[cfg(not(windows))]
pub fn quick() -> Quick {
    Quick::default()
}

#[cfg(windows)]
pub use controls::{set_brightness, set_muted, set_volume};

#[cfg(not(windows))]
pub fn set_volume(_: u8) -> Result<(), String> {
    Err("Volume is only available on Windows".into())
}

#[cfg(not(windows))]
pub fn set_muted(_: bool) -> Result<(), String> {
    Err("Volume is only available on Windows".into())
}

#[cfg(not(windows))]
pub fn set_brightness(_: u8) -> Result<(), String> {
    Err("Brightness is only available on Windows".into())
}

/// Volume through Core Audio, the built-in panel's brightness through WMI. Commands call these on worker threads, each joining COM for the call.
#[cfg(windows)]
mod controls {
    use windows::core::{w, BSTR, PCWSTR};
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoSetProxyBlanket, CoUninitialize, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE,
        RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
    };
    use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
    use windows::Win32::System::Variant::{VariantClear, VARENUM, VARIANT, VT_BSTR, VT_I4, VT_UI1};
    use windows::Win32::System::Wmi::{
        IWbemClassObject, IWbemContext, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_GENERIC_FLAG_TYPE,
        WBEM_INFINITE,
    };


    struct Com(bool);

    impl Com {
        fn join() -> Self {
            Com(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn text(e: windows::core::Error) -> String {
        e.message()
    }

    fn speakers() -> windows::core::Result<IAudioEndpointVolume> {
        unsafe {
            let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            devices.GetDefaultAudioEndpoint(eRender, eConsole)?.Activate(CLSCTX_ALL, None)
        }
    }

    pub fn volume() -> Option<(u8, bool)> {
        let _com = Com::join();
        let out = speakers().ok()?;
        unsafe { Some(((out.GetMasterVolumeLevelScalar().ok()? * 100.0).round() as u8, out.GetMute().ok()?.as_bool())) }
    }

    /// Like the volume keys, turning it up also unmutes.
    pub fn set_volume(level: u8) -> Result<(), String> {
        let _com = Com::join();
        let out = speakers().map_err(text)?;
        unsafe {
            out.SetMasterVolumeLevelScalar(f32::from(level.min(100)) / 100.0, std::ptr::null()).map_err(text)?;
            if level > 0 {
                out.SetMute(false, std::ptr::null()).map_err(text)?;
            }
        }
        Ok(())
    }

    pub fn set_muted(muted: bool) -> Result<(), String> {
        let _com = Com::join();
        unsafe { speakers().and_then(|out| out.SetMute(muted, std::ptr::null())).map_err(text) }
    }

    fn wmi() -> windows::core::Result<IWbemServices> {
        unsafe {
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
            let none = BSTR::new();
            let services = locator.ConnectServer(&BSTR::from(r"ROOT\WMI"), &none, &none, &none, 0, &none, None::<&IWbemContext>)?;
            CoSetProxyBlanket(&services, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, PCWSTR::null(), RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, None, EOAC_NONE)?;
            Ok(services)
        }
    }

    /// The first object a query returns: the built-in panel is the only active monitor
    /// these classes know, external ones don't take brightness through WMI.
    fn first(services: &IWbemServices, query: &str) -> Option<IWbemClassObject> {
        unsafe {
            let rows = services
                .ExecQuery(&BSTR::from("WQL"), &BSTR::from(query), WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, None::<&IWbemContext>)
                .ok()?;
            let mut row = [None];
            let mut found = 0;
            rows.Next(WBEM_INFINITE, &mut row, &mut found).ok().ok()?;
            row[0].take().filter(|_| found > 0)
        }
    }

    fn variant(vt: VARENUM, set: impl FnOnce(&mut windows::Win32::System::Variant::VARIANT_0_0_0)) -> VARIANT {
        let mut v = VARIANT::default();
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = vt;
            set(&mut inner.Anonymous);
        }
        v
    }

    pub fn brightness() -> Option<u8> {
        let _com = Com::join();
        let services = wmi().ok()?;
        let row = first(&services, "SELECT CurrentBrightness FROM WmiMonitorBrightness WHERE Active = TRUE")?;
        let mut v = VARIANT::default();
        unsafe {
            row.Get(w!("CurrentBrightness"), 0, &mut v, None, None).ok()?;
            let inner = &*v.Anonymous.Anonymous;
            (inner.vt == VT_UI1).then(|| inner.Anonymous.bVal)
        }
    }

    pub fn set_brightness(level: u8) -> Result<(), String> {
        let _com = Com::join();
        let services = wmi().map_err(text)?;
        let row = first(&services, "SELECT * FROM WmiMonitorBrightnessMethods WHERE Active = TRUE").ok_or("This screen's brightness can't be set")?;
        unsafe {
            let mut path = VARIANT::default();
            row.Get(w!("__PATH"), 0, &mut path, None, None).map_err(text)?;
            let inner = &*path.Anonymous.Anonymous;
            let path_text = if inner.vt == VT_BSTR { (*inner.Anonymous.bstrVal).clone() } else { BSTR::new() };
            let _ = VariantClear(&mut path);

            let mut class = None;
            services
                .GetObject(&BSTR::from("WmiMonitorBrightnessMethods"), WBEM_GENERIC_FLAG_TYPE(0), None::<&IWbemContext>, Some(&mut class), None)
                .map_err(text)?;
            let mut input = None;
            class.ok_or("No brightness control")?.GetMethod(w!("WmiSetBrightness"), 0, &mut input, std::ptr::null_mut()).map_err(text)?;
            let params = input.ok_or("No brightness control")?.SpawnInstance(0).map_err(text)?;
            // uint32 travels as VT_I4 in WMI.
            params.Put(w!("Timeout"), 0, &variant(VT_I4, |x| x.lVal = 0), 0).map_err(text)?;
            params.Put(w!("Brightness"), 0, &variant(VT_UI1, |x| x.bVal = level.min(100)), 0).map_err(text)?;
            services
                .ExecMethod(&path_text, &BSTR::from("WmiSetBrightness"), WBEM_GENERIC_FLAG_TYPE(0), None::<&IWbemContext>, &params, None, None)
                .map_err(text)?;
        }
        Ok(())
    }
}

pub fn action(app: &AppHandle, action: &str) -> Result<(), String> {
    let settings = app.state::<crate::store::AppState>().user.lock().unwrap().settings.clone();
    match action {
        "shutdown" => power_off(false, &settings.power_off_args),
        "restart" => power_off(true, &settings.restart_args),
        "desktop" => {
            // In the full screen experience there is no desktop under us until Windows switches to it.
            if crate::fse::active() && !crate::fse::switch(false) {
                crate::log::warn("couldn't switch Windows from the full screen experience to the desktop");
            }
            if let Some(w) = app.get_webview_window("main") {
                w.minimize().map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        // Windows then brings its home app, us, to the front.
        "fullscreen" => crate::fse::switch(true).then_some(()).ok_or_else(|| "Windows didn't switch to full screen mode".into()),
        "quit" => {
            app.exit(0);
            Ok(())
        }
        other => Err(format!("unknown action {other}")),
    }
}

/// shutdown.exe's switches when Settings leave them empty: /s power off or /r restart, /f close
/// programs without asking, /t 0 at once.
pub fn default_shutdown_args(restart: bool) -> Vec<String> {
    [if restart { "/r" } else { "/s" }, "/f", "/t", "0"].map(String::from).to_vec()
}

/// The switches shutdown.exe may get from Settings: what it does to this PC and when. Left out
/// are those that reach other PCs (/m), take free text (/c), open its window (/i) or are only
/// for administrators (/e).
const SHUTDOWN_SWITCHES: &[&str] = &["s", "r", "g", "sg", "h", "p", "l", "hybrid", "fw", "o", "soft", "f", "t", "d", "a"];
/// Switches that say what happens; one of them is needed.
const SHUTDOWN_ACTIONS: &[&str] = &["s", "r", "g", "sg", "h", "p", "l", "a"];

/// Settings' switches for shutdown.exe, checked one by one: only known switches, a number of
/// seconds after /t, a reason code after /d. Empty gives the defaults.
pub fn shutdown_args(text: &str, restart: bool) -> Result<Vec<String>, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Ok(default_shutdown_args(restart));
    }
    if words.len() > 10 {
        return Err("Too many switches".into());
    }
    let mut out = Vec::new();
    let mut action = false;
    let mut it = words.into_iter();
    while let Some(w) = it.next() {
        let name = w.strip_prefix('/').or_else(|| w.strip_prefix('-')).map(|n| n.to_ascii_lowercase());
        let Some(name) = name.filter(|n| SHUTDOWN_SWITCHES.contains(&n.as_str())) else {
            return Err(format!("\"{w}\" isn't a switch Mantel passes on"));
        };
        action |= SHUTDOWN_ACTIONS.contains(&name.as_str());
        out.push(format!("/{name}"));
        match name.as_str() {
            "t" => {
                let v = it.next().filter(|v| v.parse::<u32>().is_ok_and(|n| n <= 315_360_000)).ok_or("/t needs a number of seconds")?;
                out.push(v.to_string());
            }
            "d" => {
                let v = it.next().filter(|v| reason_code(v)).ok_or("/d needs a reason such as p:0:0")?;
                out.push(v.to_ascii_lowercase());
            }
            _ => {}
        }
    }
    if !action {
        return Err("Add what to do: /s, /r, /h…".into());
    }
    Ok(out)
}

/// shutdown.exe's reason: [p|u:]major:minor, in numbers.
fn reason_code(v: &str) -> bool {
    let v = v.to_ascii_lowercase();
    let v = v.strip_prefix("p:").or_else(|| v.strip_prefix("u:")).unwrap_or(&v).to_string();
    let parts: Vec<&str> = v.split(':').collect();
    parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.len() <= 5 && p.chars().all(|c| c.is_ascii_digit()))
}

/// Powers the PC off or restarts it, forced: Windows' own shutdown.exe, from System32 by its full
/// path (never a program of that name found elsewhere), with fixed arguments and no console.
/// Running programs are closed without asking, games included. Should Windows refuse to start
/// it, the same is asked of Windows directly.
#[cfg(windows)]
fn power_off(restart: bool, custom: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let args = shutdown_args(custom, restart).unwrap_or_else(|e| {
        crate::log::warn(&format!("power: the switches in Settings aren't used: {e}"));
        default_shutdown_args(restart)
    });
    let exe = system_dir().join("shutdown.exe");
    let run = std::process::Command::new(&exe)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    match run {
        Ok(_) => Ok(()),
        Err(e) => {
            crate::log::warn(&format!("power: {} didn't start ({e}), asking Windows directly", exe.display()));
            initiate_shutdown(restart)
        }
    }
}

/// The same, asked of Windows directly, forced as well.
#[cfg(windows)]
fn initiate_shutdown(restart: bool) -> Result<(), String> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
    use windows_sys::Win32::Security::{AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY};
    use windows_sys::Win32::System::Shutdown::{InitiateShutdownW, SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_OTHER, SHTDN_REASON_MINOR_OTHER, SHUTDOWN_FORCE_OTHERS, SHUTDOWN_POWEROFF, SHUTDOWN_RESTART};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        // Every program may shut Windows down, but has to switch that right on for itself first.
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
            return Err(format!("Windows didn't allow shutting down ({})", GetLastError()));
        }
        let name: Vec<u16> = "SeShutdownPrivilege".encode_utf16().chain(Some(0)).collect();
        let mut luid = LUID { LowPart: 0, HighPart: 0 };
        let mut ok = LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) != 0;
        if ok {
            let tp = TOKEN_PRIVILEGES { PrivilegeCount: 1, Privileges: [LUID_AND_ATTRIBUTES { Luid: luid, Attributes: SE_PRIVILEGE_ENABLED }] };
            ok = AdjustTokenPrivileges(token, 0, &tp, 0, std::ptr::null_mut(), std::ptr::null_mut()) != 0 && GetLastError() == 0;
        }
        CloseHandle(token);
        if !ok {
            return Err(format!("Windows didn't allow shutting down ({})", GetLastError()));
        }
        let how = (if restart { SHUTDOWN_RESTART } else { SHUTDOWN_POWEROFF }) | SHUTDOWN_FORCE_OTHERS;
        let why = SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER | SHTDN_REASON_FLAG_PLANNED;
        match InitiateShutdownW(std::ptr::null(), std::ptr::null(), 0, how, why) {
            0 => Ok(()),
            e => Err(format!("Windows didn't shut down ({e})")),
        }
    }
}

#[cfg(not(windows))]
fn power_off(_restart: bool, _custom: &str) -> Result<(), String> {
    Err("Powering off works on Windows only".into())
}

/// System32, as Windows itself reports it: not from an environment variable another program
/// could have set.
#[cfg(windows)]
fn system_dir() -> std::path::PathBuf {
    use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;
    let mut buf = [0u16; 260];
    let n = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    if n == 0 || n >= buf.len() {
        return "C:\\Windows\\System32".into();
    }
    std::path::PathBuf::from(String::from_utf16_lossy(&buf[..n]))
}

/// A program of Windows' own in the Windows folder, by its full path, so nothing else of that
/// name is run instead. The folder is the one Windows itself reports.
#[cfg(windows)]
pub fn windows_program(name: &str) -> std::path::PathBuf {
    use windows_sys::Win32::System::SystemInformation::GetWindowsDirectoryW;
    let mut buf = [0u16; 260];
    let n = unsafe { GetWindowsDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    let dir = if n == 0 || n >= buf.len() { "C:\\Windows".into() } else { String::from_utf16_lossy(&buf[..n]) };
    std::path::PathBuf::from(dir).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_switches_from_settings() {
        assert_eq!(shutdown_args("", false).unwrap(), ["/s", "/f", "/t", "0"]);
        assert_eq!(shutdown_args("  ", true).unwrap(), ["/r", "/f", "/t", "0"]);
        assert_eq!(shutdown_args("/s /hybrid /F -t 5", false).unwrap(), ["/s", "/hybrid", "/f", "/t", "5"]);
        assert_eq!(shutdown_args("/r /d P:4:1", true).unwrap(), ["/r", "/d", "p:4:1"]);
        assert!(shutdown_args("/s /m \\\\pc", false).is_err());
        assert!(shutdown_args("/s /c hello", false).is_err());
        assert!(shutdown_args("/s & calc", false).is_err());
        assert!(shutdown_args("/s /t soon", false).is_err());
        assert!(shutdown_args("/f /t 0", false).is_err());
        assert!(shutdown_args("/d p:x:1 /s", false).is_err());
    }
}
