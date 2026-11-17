//! Uninstalling games: Steam games through Steam's own dialog, and "game-uninstalled" says when
//! the game is gone. A Store game is uninstalled in Windows Settings, which this opens: this build
//! (wooshaq) never removes packages itself, so it never runs PowerShell.

use std::path::Path;
use std::time::{Duration, Instant};

use sysinfo::System;
use tauri::{AppHandle, Emitter};

use crate::model::Game;

/// How long Steam's dialog may wait for an answer before Mantel stops watching for one.
const STEAM_WAIT: Duration = Duration::from_secs(600);

/// Steam asks the player itself and does the removing; the game's manifest going away says it's
/// done. It sits two folders above the game: steamapps/common/<game>.
fn steam(app: &AppHandle, game: &Game) -> Result<(), String> {
    let dir = game.install_dir.as_deref().ok_or("Steam didn't say where this game is")?;
    let apps = Path::new(dir).parent().and_then(Path::parent).ok_or("Steam didn't say where this game is")?;
    let manifest = apps.join(format!("appmanifest_{}.acf", game.source_id));
    let before = crate::launcher::steam_dialogs(&mut System::new());
    crate::launcher::open_external(&format!("steam://uninstall/{}", game.source_id))?;
    let (app, id) = (app.clone(), game.id.clone());
    std::thread::spawn(move || {
        let start = Instant::now();
        let (mut sys, mut dialog) = (System::new(), false);
        while start.elapsed() < STEAM_WAIT {
            std::thread::sleep(Duration::from_secs(1));
            // Steam's main window stays up in front of Mantel, whichever way the dialog goes.
            if !manifest.exists() {
                crate::launcher::to_front(&app);
                return gone(&app, &id);
            }
            let up = crate::launcher::steam_dialogs(&mut sys).iter().any(|d| !before.contains(d));
            if dialog && !up {
                crate::launcher::to_front(&app);
            }
            dialog = up;
        }
    });
    Ok(())
}

fn gone(app: &AppHandle, id: &str) {
    crate::log::info(&format!("uninstalled {id}"));
    let _ = app.emit("game-uninstalled", id);
}

/// Starts removing an installed game: Steam's dialog, or Windows Settings' list of apps for a
/// Store game, where the player removes it; the next scan notices.
pub fn start(app: &AppHandle, game: &Game) -> Result<(), String> {
    crate::log::info(&format!("uninstall {}", game.id));
    match game.source.as_str() {
        "xbox" => crate::launcher::open_external("ms-settings:appsfeatures"),
        "steam" => steam(app, game),
        _ => Err("Mantel can only uninstall Xbox and Steam games for now".into()),
    }
}
