//! RTX HDR for the length of a game: the NVIDIA driver turns the game's SDR picture into HDR.
//! Mantel sets the driver's own per-game switch for it ("RTX HDR - Driver Flags", as NVIDIA
//! Profile Inspector names it) in the driver's profiles, through NVAPI's documented profile
//! functions, and puts every profile back as it was when the game ends. Nothing else of the
//! driver's settings is touched, and the NVIDIA App or its overlay isn't needed.
//!
//! The setting's id isn't in NVIDIA's public SDK; it comes from NVIDIA Profile Inspector's list
//! of driver settings. A driver that doesn't know it just ignores it.

#![cfg_attr(not(windows), allow(dead_code))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

/// "RTX HDR - Driver Flags".
pub const DRIVER_FLAGS: u32 = 0x0043_2F84;
/// "Enabled via driver", with the debanding NVIDIA ships by default (VeryHigh).
pub const ON: u32 = 0x02;

/// One change in the driver's profiles, kept to undo it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Change {
    /// A profile Mantel made for the game's programs: deleted afterwards.
    Profile { name: String },
    /// The setting in a profile that already had the program (often one of NVIDIA's own), with
    /// its value before, if that profile had one of its own.
    Setting { exe: String, before: Option<u32> },
}

/// Programs that come with games but aren't the game: installers, crash reporters, runtimes.
const NOT_THE_GAME: &[&str] = &["unins", "crash", "redist", "vcredist", "vc_redist", "dxsetup", "dotnet", "directx", "physx", "easyanticheat_setup", "be_service", "cefprocess", "webhelper", "reporter"];
/// Enough for a game's own programs; a folder with more is mostly tools.
const MAX_EXES: usize = 48;

/// The program names RTX HDR is set for: the ones Mantel knows the game by, and every program
/// in its folder up to three levels down, the game's own first.
pub fn exes(dir: Option<&Path>, known: &[String]) -> Vec<String> {
    let mut out: Vec<String> = known.iter().map(|n| n.to_lowercase()).collect();
    if let Some(dir) = dir {
        walk(dir, 0, &mut out);
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|n| n.ends_with(".exe") && !NOT_THE_GAME.iter().any(|x| n.contains(x)) && seen.insert(n.clone()));
    out.truncate(MAX_EXES);
    out
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            dirs.push(path);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.to_lowercase().ends_with(".exe") {
                out.push(name.to_lowercase());
            }
        }
    }
    if depth < 3 {
        dirs.sort();
        for d in dirs {
            walk(&d, depth + 1, out);
        }
    }
}

/// The name of the profile Mantel makes for a game.
fn profile_name(game: &str) -> String {
    format!("Mantel RTX HDR {game}")
}

/// Sets RTX HDR on for the game's programs. Returns what changed, to undo with [`restore`].
pub fn enable(game: &str, exes: &[String]) -> Result<Vec<Change>, String> {
    if exes.is_empty() {
        return Err("Mantel doesn't know this game's programs, so it can't set RTX HDR for it".into());
    }
    let changes = imp::enable(&profile_name(game), exes)?;
    remember(game, &changes);
    Ok(changes)
}

/// Puts back what [`enable`] changed.
pub fn restore(game: &str, changes: &[Change]) {
    if let Err(e) = imp::restore(changes) {
        crate::log::warn(&format!("rtx hdr: couldn't put the driver profiles back for {game}: {e}"));
        return;
    }
    forget(game);
}

// What's set and not yet put back is also kept on disk, so a crash or a power cut doesn't leave
// RTX HDR on for good: the next start puts it back.

static FILE: OnceLock<PathBuf> = OnceLock::new();
static PENDING: Mutex<()> = Mutex::new(());
const PENDING_FILE: &str = "rtx-hdr.json";

fn load() -> HashMap<String, Vec<Change>> {
    FILE.get().and_then(|f| std::fs::read(f).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn store(map: &HashMap<String, Vec<Change>>) {
    let Some(f) = FILE.get() else { return };
    if map.is_empty() {
        let _ = std::fs::remove_file(f);
    } else if let Ok(json) = serde_json::to_vec(map) {
        let _ = std::fs::write(f, json);
    }
}

fn remember(game: &str, changes: &[Change]) {
    let _g = PENDING.lock().unwrap();
    let mut map = load();
    map.insert(game.to_string(), changes.to_vec());
    store(&map);
}

fn forget(game: &str) {
    let _g = PENDING.lock().unwrap();
    let mut map = load();
    if map.remove(game).is_some() {
        store(&map);
    }
}

/// At startup: where the list is kept, and anything a previous run left set is put back.
pub fn init(dir: &Path) {
    let _ = FILE.set(dir.join(PENDING_FILE));
    for (game, changes) in load() {
        crate::log::info(&format!("rtx hdr: putting back what was left set for {game}"));
        restore(&game, &changes);
    }
}

#[cfg(windows)]
mod imp {
    use super::{Change, DRIVER_FLAGS, ON};
    use std::ffi::c_void;

    // NVAPI's profile functions, by their ids in NVIDIA's nvapi_interface.h.
    const CREATE_SESSION: u32 = 0x0694_D52E;
    const DESTROY_SESSION: u32 = 0xDAD9_CFF8;
    const LOAD_SETTINGS: u32 = 0x375D_BD6B;
    const SAVE_SETTINGS: u32 = 0xFCBC_7E14;
    const CREATE_PROFILE: u32 = 0xCC17_6068;
    const DELETE_PROFILE: u32 = 0x1709_3206;
    const FIND_PROFILE_BY_NAME: u32 = 0x7E4A_9A0B;
    const CREATE_APPLICATION: u32 = 0x4347_A9DE;
    const FIND_APPLICATION_BY_NAME: u32 = 0xEEE5_66B2;
    const SET_SETTING: u32 = 0x577D_D202;
    const GET_SETTING: u32 = 0x73BF_8338;
    const DELETE_PROFILE_SETTING: u32 = 0xE4A2_6362;

    const PROFILE_NOT_FOUND: i32 = -163;
    const EXECUTABLE_NOT_FOUND: i32 = -166;
    const SETTING_NOT_FOUND: i32 = -160;
    /// NVDRS_CURRENT_PROFILE_LOCATION: the value is the profile's own, not inherited.
    const CURRENT_PROFILE: u32 = 0;

    const STR: usize = 2048;
    type Wide = [u16; STR];

    /// NVDRS_PROFILE_V1.
    #[repr(C)]
    pub(super) struct Profile {
        version: u32,
        name: Wide,
        gpu_support: u32,
        is_predefined: u32,
        num_apps: u32,
        num_settings: u32,
    }

    /// NVDRS_APPLICATION_V4.
    #[repr(C)]
    pub(super) struct Application {
        version: u32,
        is_predefined: u32,
        app_name: Wide,
        friendly_name: Wide,
        launcher: Wide,
        file_in_folder: Wide,
        flags: u32,
        command_line: Wide,
    }

    /// NVDRS_SETTING_V1: the two value unions are 4100 bytes each (a binary value's length and
    /// its 4096 bytes); a DWORD value is the first four.
    #[repr(C)]
    pub(super) struct Setting {
        version: u32,
        name: Wide,
        id: u32,
        kind: u32,
        location: u32,
        is_current_predefined: u32,
        is_predefined_valid: u32,
        predefined: [u32; 1025],
        current: [u32; 1025],
    }

    fn zeroed<T>() -> Box<T> {
        // Plain integers and arrays of them: all zeroes is a valid value.
        unsafe { Box::new(std::mem::zeroed()) }
    }

    fn versioned<T>(v: u32) -> u32 {
        std::mem::size_of::<T>() as u32 | (v << 16)
    }

    fn wide(s: &str) -> Box<Wide> {
        let mut w: Box<Wide> = zeroed();
        for (i, c) in s.encode_utf16().take(STR - 1).enumerate() {
            w[i] = c;
        }
        w
    }

    type H = *mut c_void;
    struct Api {
        create_session: unsafe extern "C" fn(*mut H) -> i32,
        destroy_session: unsafe extern "C" fn(H) -> i32,
        load: unsafe extern "C" fn(H) -> i32,
        save: unsafe extern "C" fn(H) -> i32,
        create_profile: unsafe extern "C" fn(H, *mut Profile, *mut H) -> i32,
        delete_profile: unsafe extern "C" fn(H, H) -> i32,
        find_profile: unsafe extern "C" fn(H, *const u16, *mut H) -> i32,
        create_app: unsafe extern "C" fn(H, H, *mut Application) -> i32,
        find_app: unsafe extern "C" fn(H, *const u16, *mut H, *mut Application) -> i32,
        set: unsafe extern "C" fn(H, H, *mut Setting) -> i32,
        get: unsafe extern "C" fn(H, H, u32, *mut Setting) -> i32,
        delete_setting: unsafe extern "C" fn(H, H, u32) -> i32,
    }

    /// A loaded copy of the driver's profiles, saved on [`Session::save`] and let go on drop.
    struct Session {
        api: Api,
        h: H,
    }

    impl Session {
        fn open() -> Result<Session, String> {
            let query = crate::hdr::nvapi_query()?;
            let f = |id: u32| -> Result<*const c_void, String> {
                let p = unsafe { query(id) };
                if p.is_null() {
                    Err("This NVIDIA driver doesn't offer per-game profiles".into())
                } else {
                    Ok(p)
                }
            };
            let api = unsafe {
                Api {
                    create_session: std::mem::transmute(f(CREATE_SESSION)?),
                    destroy_session: std::mem::transmute(f(DESTROY_SESSION)?),
                    load: std::mem::transmute(f(LOAD_SETTINGS)?),
                    save: std::mem::transmute(f(SAVE_SETTINGS)?),
                    create_profile: std::mem::transmute(f(CREATE_PROFILE)?),
                    delete_profile: std::mem::transmute(f(DELETE_PROFILE)?),
                    find_profile: std::mem::transmute(f(FIND_PROFILE_BY_NAME)?),
                    create_app: std::mem::transmute(f(CREATE_APPLICATION)?),
                    find_app: std::mem::transmute(f(FIND_APPLICATION_BY_NAME)?),
                    set: std::mem::transmute(f(SET_SETTING)?),
                    get: std::mem::transmute(f(GET_SETTING)?),
                    delete_setting: std::mem::transmute(f(DELETE_PROFILE_SETTING)?),
                }
            };
            let mut h: H = std::ptr::null_mut();
            let r = unsafe { (api.create_session)(&mut h) };
            if r != 0 {
                return Err(format!("The NVIDIA driver didn't open its profiles ({r})"));
            }
            let s = Session { api, h };
            let r = unsafe { (s.api.load)(s.h) };
            if r != 0 {
                return Err(format!("The NVIDIA driver didn't load its profiles ({r})"));
            }
            Ok(s)
        }

        fn save(&self) -> Result<(), String> {
            match unsafe { (self.api.save)(self.h) } {
                0 => Ok(()),
                r => Err(format!("The NVIDIA driver didn't save its profiles ({r})")),
            }
        }

        /// The profile a program belongs to, if any.
        fn app_profile(&self, exe: &str) -> Result<Option<H>, String> {
            let name = wide(exe);
            let mut app: Box<Application> = zeroed();
            app.version = versioned::<Application>(4);
            let mut p: H = std::ptr::null_mut();
            match unsafe { (self.api.find_app)(self.h, name.as_ptr(), &mut p, &mut *app) } {
                0 => Ok(Some(p)),
                EXECUTABLE_NOT_FOUND | PROFILE_NOT_FOUND => Ok(None),
                r => Err(format!("NVIDIA driver error {r} looking up {exe}")),
            }
        }

        fn profile(&self, name: &str) -> Option<H> {
            let w = wide(name);
            let mut p: H = std::ptr::null_mut();
            (unsafe { (self.api.find_profile)(self.h, w.as_ptr(), &mut p) } == 0).then_some(p)
        }

        /// The profile's own value of a setting, not one it inherits.
        fn own_value(&self, p: H, id: u32) -> Option<u32> {
            let mut s: Box<Setting> = zeroed();
            s.version = versioned::<Setting>(1);
            match unsafe { (self.api.get)(self.h, p, id, &mut *s) } {
                0 if s.location == CURRENT_PROFILE => Some(s.current[0]),
                _ => None,
            }
        }

        fn set(&self, p: H, id: u32, value: u32) -> Result<(), String> {
            let mut s: Box<Setting> = zeroed();
            s.version = versioned::<Setting>(1);
            s.id = id;
            s.kind = 0; // NVDRS_DWORD_TYPE
            s.current[0] = value;
            match unsafe { (self.api.set)(self.h, p, &mut *s) } {
                0 => Ok(()),
                r => Err(format!("The NVIDIA driver refused the setting ({r})")),
            }
        }

        fn unset(&self, p: H, id: u32) {
            let r = unsafe { (self.api.delete_setting)(self.h, p, id) };
            if r != 0 && r != SETTING_NOT_FOUND {
                crate::log::warn(&format!("rtx hdr: driver error {r} clearing the setting"));
            }
        }

        fn delete_profile(&self, name: &str) {
            if let Some(p) = self.profile(name) {
                unsafe { (self.api.delete_profile)(self.h, p) };
            }
        }
    }

    impl Drop for Session {
        fn drop(&mut self) {
            unsafe { (self.api.destroy_session)(self.h) };
        }
    }

    pub fn enable(profile: &str, exes: &[String]) -> Result<Vec<Change>, String> {
        let s = Session::open()?;
        // Left over from a run that never got to put it back.
        s.delete_profile(profile);
        let mut changes = Vec::new();
        let mut own: Option<H> = None;
        for exe in exes {
            match s.app_profile(exe)? {
                Some(p) => {
                    let before = s.own_value(p, DRIVER_FLAGS);
                    s.set(p, DRIVER_FLAGS, ON)?;
                    changes.push(Change::Setting { exe: exe.clone(), before });
                }
                None => {
                    let p = match own {
                        Some(p) => p,
                        None => {
                            let mut pr: Box<Profile> = zeroed();
                            pr.version = versioned::<Profile>(1);
                            pr.name = *wide(profile);
                            let mut p: H = std::ptr::null_mut();
                            let r = unsafe { (s.api.create_profile)(s.h, &mut *pr, &mut p) };
                            if r != 0 {
                                return Err(format!("The NVIDIA driver didn't make a profile ({r})"));
                            }
                            s.set(p, DRIVER_FLAGS, ON)?;
                            changes.push(Change::Profile { name: profile.to_string() });
                            own = Some(p);
                            p
                        }
                    };
                    let mut app: Box<Application> = zeroed();
                    app.version = versioned::<Application>(4);
                    app.app_name = *wide(exe);
                    let r = unsafe { (s.api.create_app)(s.h, p, &mut *app) };
                    if r != 0 {
                        crate::log::warn(&format!("rtx hdr: driver error {r} adding {exe}"));
                    }
                }
            }
        }
        s.save()?;
        Ok(changes)
    }

    pub fn restore(changes: &[Change]) -> Result<(), String> {
        let s = Session::open()?;
        for c in changes {
            match c {
                Change::Profile { name } => s.delete_profile(name),
                Change::Setting { exe, before } => {
                    let Some(p) = s.app_profile(exe)? else { continue };
                    match before {
                        Some(v) => s.set(p, DRIVER_FLAGS, *v)?,
                        None => s.unset(p, DRIVER_FLAGS),
                    }
                }
            }
        }
        s.save()
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn structs_match_the_sdk() {
            assert_eq!(std::mem::size_of::<super::Profile>(), 4116);
            assert_eq!(std::mem::size_of::<super::Application>(), 20492);
            assert_eq!(std::mem::size_of::<super::Setting>(), 12320);
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Change;

    pub fn enable(_profile: &str, _exes: &[String]) -> Result<Vec<Change>, String> {
        Err("RTX HDR works on Windows only".into())
    }

    pub fn restore(_changes: &[Change]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_games_programs_and_skips_the_rest() {
        let dir = std::env::temp_dir().join(format!("mantel-rtx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("bin/x64")).unwrap();
        std::fs::create_dir_all(dir.join("_CommonRedist")).unwrap();
        for f in ["Game.exe", "unins000.exe", "bin/x64/Game-Win64-Shipping.exe", "bin/x64/CrashReportClient.exe", "_CommonRedist/vc_redist.x64.exe", "readme.txt"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        let got = exes(Some(&dir), &["Game.exe".into()]);
        assert_eq!(got, ["game.exe", "game-win64-shipping.exe"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn changes_survive_a_restart() {
        let a = vec![Change::Profile { name: "Mantel RTX HDR steam:1".into() }, Change::Setting { exe: "game.exe".into(), before: Some(3) }];
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(serde_json::from_str::<Vec<Change>>(&json).unwrap(), a);
    }
}
