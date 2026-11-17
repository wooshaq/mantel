//! HDR for the length of a game: switched on before a game the player marked starts, and off
//! again when it ends, but only on screens Mantel switched on itself. Two ways to do it:
//! Windows' own display settings, or the NVIDIA driver (NVAPI) for cards that prefer it.

use serde::{Deserialize, Serialize};

/// How HDR gets switched on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    #[default]
    Windows,
    Nvidia,
}

/// A screen Mantel switched to HDR, to switch back afterwards.
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Switched {
    Windows { adapter: (u32, i32), target: u32, modern: bool },
    Nvidia { display: u32 },
}

/// Switches HDR on for every screen that supports it and has it off. Returns what it changed;
/// screens already in HDR are left alone, so they stay in HDR after the game too.
pub fn enable(method: Method) -> Result<Vec<Switched>, String> {
    let switched = match method {
        Method::Windows => imp::windows_on()?,
        Method::Nvidia => imp::nvidia_on()?,
    };
    if !switched.is_empty() {
        // The screen goes dark for a moment while it changes mode; a game that looks for HDR
        // at startup should find it on.
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
    Ok(switched)
}

/// Switches back what [`enable`] changed.
pub fn restore(switched: &[Switched]) {
    for s in switched {
        if let Err(e) = imp::off(s) {
            crate::log::warn(&format!("hdr: couldn't switch back {s:?}: {e}"));
        }
    }
}

#[cfg(windows)]
mod imp {
    use super::Switched;
    use std::ffi::c_void;
    use windows_sys::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo, DisplayConfigSetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, QDC_ONLY_ACTIVE_PATHS,
    };
    use windows_sys::Win32::Foundation::LUID;

    const GET_SOURCE_NAME: i32 = 1;
    const GET_ADVANCED_COLOR_INFO: i32 = 9;
    const SET_ADVANCED_COLOR_STATE: i32 = 10;
    /// Windows 11 24H2 and later: HDR on its own, apart from the colour management that the
    /// older call also turns on.
    const GET_ADVANCED_COLOR_INFO_2: i32 = 15;
    const SET_HDR_STATE: i32 = 16;
    const MODE_HDR: i32 = 2;

    #[repr(C)]
    struct ColorInfo {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
        encoding: i32,
        bits: u32,
    }

    #[repr(C)]
    struct ColorInfo2 {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
        encoding: i32,
        bits: u32,
        mode: i32,
    }

    #[repr(C)]
    struct SetState {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        value: u32,
    }

    #[repr(C)]
    struct SourceName {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        gdi_name: [u16; 32],
    }

    fn header(kind: i32, size: usize, adapter: LUID, id: u32) -> DISPLAYCONFIG_DEVICE_INFO_HEADER {
        DISPLAYCONFIG_DEVICE_INFO_HEADER { r#type: kind, size: size as u32, adapterId: adapter, id }
    }

    /// The screens in use: adapter, target and GDI name ("\\.\DISPLAY1") of each.
    fn screens() -> Result<Vec<(LUID, u32, String)>, String> {
        unsafe {
            let (mut n_paths, mut n_modes) = (0u32, 0u32);
            let r = GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut n_paths, &mut n_modes);
            if r != 0 {
                return Err(format!("display list unavailable ({r})"));
            }
            let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![std::mem::zeroed(); n_paths as usize];
            let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![std::mem::zeroed(); n_modes as usize];
            let r = QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS, &mut n_paths, paths.as_mut_ptr(), &mut n_modes, modes.as_mut_ptr(), std::ptr::null_mut());
            if r != 0 {
                return Err(format!("display list unavailable ({r})"));
            }
            paths.truncate(n_paths as usize);
            Ok(paths
                .iter()
                .map(|p| {
                    let mut name = SourceName {
                        header: header(GET_SOURCE_NAME, std::mem::size_of::<SourceName>(), p.sourceInfo.adapterId, p.sourceInfo.id),
                        gdi_name: [0; 32],
                    };
                    DisplayConfigGetDeviceInfo(&mut name.header);
                    let len = name.gdi_name.iter().position(|&c| c == 0).unwrap_or(32);
                    (p.targetInfo.adapterId, p.targetInfo.id, String::from_utf16_lossy(&name.gdi_name[..len]))
                })
                .collect())
        }
    }

    /// Whether the screen can show HDR and has it on, and whether the newer calls work here.
    fn state(adapter: LUID, target: u32) -> Option<(bool, bool, bool)> {
        unsafe {
            let mut info2 = ColorInfo2 { header: header(GET_ADVANCED_COLOR_INFO_2, std::mem::size_of::<ColorInfo2>(), adapter, target), value: 0, encoding: 0, bits: 0, mode: 0 };
            if DisplayConfigGetDeviceInfo(&mut info2.header) == 0 {
                let supported = info2.value & (1 << 4) != 0;
                return Some((supported, info2.mode == MODE_HDR, true));
            }
            let mut info = ColorInfo { header: header(GET_ADVANCED_COLOR_INFO, std::mem::size_of::<ColorInfo>(), adapter, target), value: 0, encoding: 0, bits: 0 };
            if DisplayConfigGetDeviceInfo(&mut info.header) == 0 {
                let supported = info.value & 1 != 0;
                let force_off = info.value & (1 << 3) != 0;
                return Some((supported && !force_off, info.value & (1 << 1) != 0, false));
            }
            None
        }
    }

    fn set(adapter: LUID, target: u32, modern: bool, on: bool) -> Result<(), String> {
        let kind = if modern { SET_HDR_STATE } else { SET_ADVANCED_COLOR_STATE };
        let mut s = SetState { header: header(kind, std::mem::size_of::<SetState>(), adapter, target), value: on as u32 };
        let r = unsafe { DisplayConfigSetDeviceInfo(&mut s.header) };
        if r == 0 {
            Ok(())
        } else {
            Err(format!("Windows refused ({r})"))
        }
    }

    pub fn windows_on() -> Result<Vec<Switched>, String> {
        let mut out = Vec::new();
        let mut any = false;
        for (adapter, target, _) in screens()? {
            let Some((supported, on, modern)) = state(adapter, target) else { continue };
            any |= supported;
            if supported && !on {
                set(adapter, target, modern, true)?;
                out.push(Switched::Windows { adapter: (adapter.LowPart, adapter.HighPart), target, modern });
            }
        }
        if !any {
            return Err("No screen here can show HDR".into());
        }
        Ok(out)
    }

    // NVAPI: nvapi64.dll ships with the NVIDIA driver; every function is found through
    // nvapi_QueryInterface by its id from NVIDIA's nvapi_interface.h.
    const NVAPI_INITIALIZE: u32 = 0x0150_E828;
    const NVAPI_GET_DISPLAY_ID_BY_NAME: u32 = 0xAE45_7190;
    const NVAPI_HDR_COLOR_CONTROL: u32 = 0x351D_A224;
    const HDR_CMD_GET: u32 = 0;
    const HDR_CMD_SET: u32 = 1;
    const HDR_MODE_OFF: u32 = 0;
    const HDR_MODE_UHDA: u32 = 2;

    /// NV_HDR_COLOR_DATA_V2.
    #[repr(C)]
    #[derive(Default)]
    struct HdrColorData {
        version: u32,
        cmd: u32,
        mode: u32,
        metadata_id: u32,
        mastering: [u16; 12],
        color_format: u32,
        dynamic_range: u32,
        bpc: u32,
    }

    impl HdrColorData {
        fn new(cmd: u32) -> Self {
            HdrColorData { version: std::mem::size_of::<Self>() as u32 | (2 << 16), cmd, ..Default::default() }
        }
    }

    type QueryInterface = unsafe extern "C" fn(u32) -> *const c_void;
    type Initialize = unsafe extern "C" fn() -> i32;
    type DisplayIdByName = unsafe extern "C" fn(*const u8, *mut u32) -> i32;
    type HdrColorControl = unsafe extern "C" fn(u32, *mut HdrColorData) -> i32;

    struct Nvapi {
        display_id: DisplayIdByName,
        control: HdrColorControl,
    }

    fn nvapi() -> Result<Nvapi, String> {
        use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};
        let missing = || "The NVIDIA driver isn't installed here: switch HDR to Windows in Settings".to_string();
        unsafe {
            // Only from System32, where the driver puts it.
            let name: Vec<u16> = "nvapi64.dll\0".encode_utf16().collect();
            let lib = LoadLibraryExW(name.as_ptr(), std::ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32);
            if lib.is_null() {
                return Err(missing());
            }
            let query = GetProcAddress(lib, c"nvapi_QueryInterface".as_ptr() as *const u8).ok_or_else(missing)?;
            let query: QueryInterface = std::mem::transmute(query);
            let get = |id: u32| {
                let f = query(id);
                (!f.is_null()).then_some(f)
            };
            let init: Initialize = std::mem::transmute(get(NVAPI_INITIALIZE).ok_or_else(missing)?);
            if init() != 0 {
                return Err("The NVIDIA driver didn't answer".into());
            }
            Ok(Nvapi {
                display_id: std::mem::transmute(get(NVAPI_GET_DISPLAY_ID_BY_NAME).ok_or_else(missing)?),
                control: std::mem::transmute(get(NVAPI_HDR_COLOR_CONTROL).ok_or_else(missing)?),
            })
        }
    }

    pub fn nvidia_on() -> Result<Vec<Switched>, String> {
        let api = nvapi()?;
        let mut out = Vec::new();
        let mut any = false;
        for (adapter, target, name) in screens()? {
            let mut id = 0u32;
            let cname = format!("{name}\0");
            if unsafe { (api.display_id)(cname.as_ptr(), &mut id) } != 0 {
                continue; // A screen on another card.
            }
            let mut data = HdrColorData::new(HDR_CMD_GET);
            if unsafe { (api.control)(id, &mut data) } != 0 {
                continue;
            }
            // NVAPI says what's on, Windows whether the screen can do it at all.
            if state(adapter, target).is_some_and(|(supported, ..)| !supported) {
                continue;
            }
            any = true;
            if data.mode == HDR_MODE_UHDA {
                continue;
            }
            data.cmd = HDR_CMD_SET;
            data.mode = HDR_MODE_UHDA;
            let r = unsafe { (api.control)(id, &mut data) };
            if r != 0 {
                return Err(format!("The NVIDIA driver refused HDR ({r})"));
            }
            out.push(Switched::Nvidia { display: id });
        }
        if !any {
            return Err("No screen on the NVIDIA card can show HDR".into());
        }
        Ok(out)
    }

    pub fn off(s: &Switched) -> Result<(), String> {
        match *s {
            Switched::Windows { adapter, target, modern } => set(LUID { LowPart: adapter.0, HighPart: adapter.1 }, target, modern, false),
            Switched::Nvidia { display } => {
                let api = nvapi()?;
                let mut data = HdrColorData::new(HDR_CMD_GET);
                unsafe { (api.control)(display, &mut data) };
                data.cmd = HDR_CMD_SET;
                data.mode = HDR_MODE_OFF;
                match unsafe { (api.control)(display, &mut data) } {
                    0 => Ok(()),
                    r => Err(format!("NVIDIA driver error {r}")),
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn nvapi_struct_matches_the_sdk() {
            // NV_HDR_COLOR_DATA_V2 is 52 bytes, and its version says so.
            assert_eq!(std::mem::size_of::<super::HdrColorData>(), 52);
            assert_eq!(super::HdrColorData::new(0).version, 52 | (2 << 16));
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Switched;

    pub fn windows_on() -> Result<Vec<Switched>, String> {
        Err("HDR switching works on Windows only".into())
    }

    pub fn nvidia_on() -> Result<Vec<Switched>, String> {
        windows_on()
    }

    pub fn off(_s: &Switched) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Method;

    #[test]
    fn method_reads_as_settings_save_it() {
        assert_eq!(serde_json::from_str::<Method>("\"nvidia\"").unwrap(), Method::Nvidia);
        assert_eq!(serde_json::to_string(&Method::Windows).unwrap(), "\"windows\"");
    }
}
