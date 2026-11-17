use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::model::{Art, Game};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Sources {
    pub steam: bool,
    pub epic: bool,
    pub gog: bool,
    pub xbox: bool,
    pub ubisoft: bool,
}

impl Default for Sources {
    fn default() -> Self {
        Sources { steam: true, epic: true, gog: true, xbox: true, ubisoft: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub sources: Sources,
    pub steamgriddb_key: String,
    pub sounds: bool,
    pub minimize_on_launch: bool,
    /// Muted trailers behind the focused game on Home.
    pub trailers: bool,
    /// Interface language ("en", "fr"); empty follows Windows.
    pub language: String,
    /// The Game Pass tab: the catalog is public, so it's on until the player hides it.
    pub game_pass: bool,
    /// SteamID64 for achievements from the public profile; empty takes the account Steam
    /// last signed in with on this PC.
    pub steam_id: String,
    /// Steam games start in Big Picture, which closes again when the game ends.
    pub big_picture: bool,
    /// How HDR is switched on for games that ask for it.
    pub hdr_method: crate::hdr::Method,
    /// An icon in the notification area; minimizing or closing the window hides it there.
    pub tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sources: Sources::default(),
            steamgriddb_key: String::new(),
            sounds: true,
            minimize_on_launch: true,
            trailers: true,
            language: String::new(),
            game_pass: true,
            steam_id: String::new(),
            big_picture: false,
            hdr_method: crate::hdr::Method::Windows,
            tray: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameMeta {
    pub favorite: bool,
    pub hidden: bool,
    /// Minutes Mantel measured itself, summed across sessions.
    pub tracked_minutes: u64,
    pub last_played: Option<i64>,
    pub added_at: Option<i64>,
    /// Images the player picked, shown ahead of everything the stores offer.
    pub art: Art,
    /// HDR on while the game runs.
    pub hdr: bool,
    /// The game's page on HowLongToBeat, linked by the player.
    pub hltb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomGame {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// For shortcuts: the exe they point at, which is what shows up as a running process.
    #[serde(default)]
    pub target: Option<String>,
}

/// Everything the user created: lost data here is lost for good.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserData {
    pub settings: Settings,
    pub meta: HashMap<String, GameMeta>,
    pub custom: Vec<CustomGame>,
    /// The Apps tab: apps the player added, and known ones that joined by themselves.
    pub apps: Vec<crate::apps::AppEntry>,
    /// Apps the player took off the tab, by what launches them, so they don't come back.
    pub dismissed_apps: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResolvedArt {
    pub art: Art,
    pub steam_app_id: Option<u32>,
    pub at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MeasuredSize {
    pub bytes: Option<u64>,
    pub at: i64,
}

/// Rebuildable data: the last scan and lookups against online services.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Cache {
    pub library: Vec<Game>,
    pub asset_dirs: Vec<PathBuf>,
    pub art: HashMap<String, ResolvedArt>,
    pub details: HashMap<u32, serde_json::Value>,
    /// Install folder sizes for stores that don't report one, by install folder.
    pub sizes: HashMap<String, MeasuredSize>,
    pub gamepass: crate::gamepass::Catalog,
    pub steam_facts: crate::steam_store::Facts,
    /// Achievement progress from the public Steam profile, by app id.
    pub steam_public: HashMap<u32, crate::steam_public::Progress>,
    /// Times read from HowLongToBeat, by its game id.
    pub hltb: HashMap<u32, crate::hltb::Hltb>,
}

/// One stretch of play, as the tracker saw it: start and end in Unix seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub game: String,
    pub start: i64,
    pub end: i64,
}

pub struct AppState {
    dir: PathBuf,
    pub user: Mutex<UserData>,
    pub cache: Mutex<Cache>,
    /// Games running now, oldest first, including a launch still waiting for its process.
    pub running: Mutex<Vec<crate::launcher::Running>>,
    /// What "Add an app" offered last: adding takes an app from here, never a path from the UI.
    pub app_choices: Mutex<Vec<crate::apps::AppEntry>>,
    /// An older version was signed in to Steam, whose session lives in the WebView's cookies.
    pub legacy_steam_session: bool,
    /// Every session played since this was added, oldest first. Kept apart from the cache,
    /// which can be thrown away, and from the settings, which are read far more often.
    pub sessions: Mutex<Vec<Session>>,
}

const USER_FILE: &str = "user.json";
const CACHE_FILE: &str = "cache.json";
const SESSIONS_FILE: &str = "sessions.json";

impl AppState {
    pub fn load(dir: PathBuf) -> Self {
        let legacy = legacy_accounts(&dir.join(USER_FILE));
        let mut state = AppState {
            user: Mutex::new(read_json(&dir.join(USER_FILE))),
            cache: Mutex::new(read_json(&dir.join(CACHE_FILE))),
            running: Mutex::new(Vec::new()),
            app_choices: Mutex::new(Vec::new()),
            legacy_steam_session: false,
            sessions: Mutex::new(read_json(&dir.join(SESSIONS_FILE))),
            dir,
        };
        // Sign-ins are gone from this build: tokens an older version kept are deleted from disk
        // right away, and whatever was learned through them from the cache.
        if legacy.is_some() {
            state.save_user(&state.user.lock().unwrap());
            let mut cache = state.cache.lock().unwrap();
            cache.library.retain(|g| g.installed);
            state.save_cache(&cache);
            crate::log::info("account sign-ins from an older version removed");
        }
        state.legacy_steam_session = legacy.unwrap_or(false);
        state
    }

    /// Copies of image files the player picked as art, so they outlive the originals.
    pub fn art_dir(&self) -> PathBuf {
        self.dir.join("art")
    }

    /// Where app icons taken from programs and sites are kept.
    pub fn icons_dir(&self) -> PathBuf {
        self.dir.join("icons")
    }

    pub fn save_user(&self, user: &UserData) {
        write_json(&self.dir.join(USER_FILE), user);
    }

    pub fn save_cache(&self, cache: &Cache) {
        write_json(&self.dir.join(CACHE_FILE), cache);
    }

    /// Adds a finished session to the play log.
    pub fn log_session(&self, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.push(session);
        write_json(&self.dir.join(SESSIONS_FILE), &*sessions);
    }
}

/// Whether `user.json` still has sign-ins from an older version: Some(true) with a Steam session.
fn legacy_accounts(path: &Path) -> Option<bool> {
    let v: serde_json::Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    let accounts = v.get("accounts")?;
    Some(accounts.get("steamSignedIn").and_then(serde_json::Value::as_bool).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_drops_old_sign_ins() {
        let dir = std::env::temp_dir().join("mantel-legacy-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(USER_FILE);
        std::fs::write(&file, r#"{"settings":{"sounds":false},"accounts":{"steamSignedIn":true,"steamToken":"dpapi:abc","xboxRefresh":"x"}}"#).unwrap();
        assert_eq!(legacy_accounts(&file), Some(true));
        let user: UserData = read_json(&file);
        assert!(!user.settings.sounds);
        let saved = serde_json::to_string(&user).unwrap();
        assert!(!saved.contains("accounts") && !saved.contains("dpapi") && !saved.contains("xboxRefresh"));
        std::fs::write(&file, r#"{"settings":{}}"#).unwrap();
        assert_eq!(legacy_accounts(&file), None);
    }
}

fn read_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Writes through a temp file so a crash mid-write never truncates the original.
fn write_json<T: Serialize>(path: &Path, value: &T) {
    let Ok(bytes) = serde_json::to_vec(value) else { return };
    let tmp = path.with_extension("json.tmp");
    if fs::write(&tmp, bytes).is_ok() {
        let _ = fs::rename(&tmp, path);
    }
}
