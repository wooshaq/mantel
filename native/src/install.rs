//! Progress of installs Mantel follows, reported to the UI as "install-progress" events. Only
//! Steam installs are followed here (steam_install.rs): this build (wooshaq) doesn't install Store
//! games itself. Those open on their page in the Xbox app or the Microsoft Store, which installs
//! them, so Mantel never runs PowerShell or touches Windows' installer.

use std::sync::Mutex;

use serde::Serialize;
use crate::host::{AppHandle, Emitter};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// The game's id in Mantel.
    pub id: String,
    /// "pending", "downloading", "installing", "paused", "done", "canceled" or "error".
    pub state: String,
    pub percent: f32,
    /// Bytes so far and in all; 0 until the store knows.
    pub downloaded: u64,
    pub total: u64,
    pub error: Option<String>,
}

/// Installs Mantel follows, by game id; finished ones drop out.
static ACTIVE: Mutex<Vec<Progress>> = Mutex::new(Vec::new());

pub(crate) fn publish(app: &AppHandle, p: Progress) {
    {
        let mut active = ACTIVE.lock().unwrap();
        active.retain(|a| a.id != p.id);
        if !matches!(p.state.as_str(), "done" | "canceled" | "error") {
            active.push(p.clone());
        }
    }
    let _ = app.emit("install-progress", &p);
}

pub fn active() -> Vec<Progress> {
    ACTIVE.lock().unwrap().clone()
}
