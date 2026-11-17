//! Which screen shows the picture: the handheld's own or an external one, as Windows+P and
//! DisplaySwitch.exe switch it. Done through the same Windows call DisplaySwitch makes, with
//! no program started.

use serde::Serialize;

/// The screens in use, as Windows names the arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Internal,
    External,
    Extend,
    Clone,
}

impl Mode {
    /// Where a press goes: from the built-in screen to the external one, from anything
    /// else back to the built-in screen.
    pub fn next(self) -> Mode {
        if self == Mode::Internal {
            Mode::External
        } else {
            Mode::Internal
        }
    }
}

#[cfg(windows)]
pub fn mode() -> Option<Mode> {
    use windows_sys::Win32::Devices::Display::{
        GetDisplayConfigBufferSizes, QueryDisplayConfig, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TOPOLOGY_CLONE, DISPLAYCONFIG_TOPOLOGY_EXTEND,
        DISPLAYCONFIG_TOPOLOGY_EXTERNAL, DISPLAYCONFIG_TOPOLOGY_INTERNAL, QDC_DATABASE_CURRENT,
    };
    unsafe {
        let (mut n_paths, mut n_modes) = (0u32, 0u32);
        if GetDisplayConfigBufferSizes(QDC_DATABASE_CURRENT, &mut n_paths, &mut n_modes) != 0 {
            return None;
        }
        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![std::mem::zeroed(); n_paths as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![std::mem::zeroed(); n_modes as usize];
        let mut topology = 0i32;
        if QueryDisplayConfig(QDC_DATABASE_CURRENT, &mut n_paths, paths.as_mut_ptr(), &mut n_modes, modes.as_mut_ptr(), &mut topology) != 0 {
            return None;
        }
        match topology {
            DISPLAYCONFIG_TOPOLOGY_INTERNAL => Some(Mode::Internal),
            DISPLAYCONFIG_TOPOLOGY_EXTERNAL => Some(Mode::External),
            DISPLAYCONFIG_TOPOLOGY_EXTEND => Some(Mode::Extend),
            DISPLAYCONFIG_TOPOLOGY_CLONE => Some(Mode::Clone),
            _ => None,
        }
    }
}

/// Switches to the other screen and returns the new arrangement.
#[cfg(windows)]
pub fn switch() -> Result<Mode, String> {
    use windows_sys::Win32::Devices::Display::{SetDisplayConfig, SDC_APPLY, SDC_TOPOLOGY_EXTERNAL, SDC_TOPOLOGY_INTERNAL};
    let target = mode().unwrap_or(Mode::Internal).next();
    let flag = if target == Mode::External { SDC_TOPOLOGY_EXTERNAL } else { SDC_TOPOLOGY_INTERNAL };
    let r = unsafe { SetDisplayConfig(0, std::ptr::null(), 0, std::ptr::null(), SDC_APPLY | flag) };
    match r {
        0 => Ok(target),
        _ if target == Mode::External => Err("No external screen is connected".into()),
        r => Err(format!("Windows didn't switch the screen ({r})")),
    }
}

#[cfg(not(windows))]
pub fn mode() -> Option<Mode> {
    None
}

#[cfg(not(windows))]
pub fn switch() -> Result<Mode, String> {
    Err("Switching screens works on Windows only".into())
}

#[cfg(test)]
mod tests {
    use super::Mode;

    #[test]
    fn a_press_goes_out_and_comes_back() {
        assert_eq!(Mode::Internal.next(), Mode::External);
        assert_eq!(Mode::External.next(), Mode::Internal);
        assert_eq!(Mode::Extend.next(), Mode::Internal);
        assert_eq!(serde_json::to_string(&Mode::Extend).unwrap(), "\"extend\"");
    }
}
