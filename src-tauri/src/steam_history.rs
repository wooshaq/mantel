//! Past Steam sessions, read from the log the Steam client keeps of the games it runs
//! (`logs/gameprocess_log.txt`, and the older part in `gameprocess_log.previous.txt`). Each game
//! start and stop is there with its local date and time, as far back as Steam keeps the files.
//! Mantel only reads them.

use std::collections::HashMap;
use std::path::Path;

use crate::store::Session;

/// Sessions shorter than this are launchers that closed at once, not play.
const MIN_SECS: i64 = 60;
/// Longer than this isn't a sitting: Steam missed the end (it was closed, or the PC slept with
/// the game open), or the program runs all day in the background, like a wallpaper app.
pub const MAX_SECS: i64 = 12 * 3600;
/// Steam app types that run but aren't games.
const NOT_GAMES: &[&str] = &["application", "tool", "music", "video", "series", "episode", "hardware", "advertising"];

/// What Steam's store says an app is.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppInfo {
    pub name: String,
    /// "game", "application"…; empty when the store has no page for it.
    pub kind: String,
    pub at: i64,
}

impl AppInfo {
    pub fn is_game(&self) -> bool {
        !NOT_GAMES.contains(&self.kind.as_str())
    }
}

/// How long an unknown app's answer is kept before Steam is asked again.
const INFO_SECS: i64 = 30 * 24 * 3600;

/// Name and type of each app id not known yet, from the store's public app details.
pub async fn app_info(ids: Vec<u32>, known: &HashMap<u32, AppInfo>) -> HashMap<u32, AppInfo> {
    use futures::StreamExt;
    let t = crate::model::now();
    let todo: Vec<u32> = ids.into_iter().filter(|id| known.get(id).is_none_or(|i| i.kind.is_empty() && t - i.at > INFO_SECS)).collect();
    let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).user_agent("Mantel/0.1 (game launcher)").build().unwrap_or_default();
    futures::stream::iter(todo)
        .map(|id| {
            let http = http.clone();
            async move {
                let v: serde_json::Value = http
                    .get("https://store.steampowered.com/api/appdetails")
                    .query(&[("appids", id.to_string()), ("filters", "basic".into())])
                    .send()
                    .await
                    .ok()?
                    .json()
                    .await
                    .ok()?;
                let data = v.get(id.to_string()).and_then(|x| x.get("data"));
                let text = |k: &str| data.and_then(|d| d.get(k)).and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
                Some((id, AppInfo { name: text("name"), kind: text("type"), at: t }))
            }
        })
        .buffer_unordered(2)
        .filter_map(|x| async { x })
        .collect()
        .await
}

/// Takes out of the log what isn't play: sittings too long to be real, and apps that aren't
/// games. Whether anything went.
pub fn prune(log: &mut Vec<Session>, apps: &HashMap<u32, AppInfo>) -> bool {
    let before = log.len();
    log.retain(|s| {
        let app = s.game.strip_prefix("steam:").and_then(|id| id.parse::<u32>().ok()).and_then(|id| apps.get(&id));
        s.end - s.start <= MAX_SECS && app.is_none_or(AppInfo::is_game)
    });
    log.len() != before
}

/// App ids of the Steam games in sessions.
pub fn steam_ids(sessions: &[Session]) -> Vec<u32> {
    let mut ids: Vec<u32> = sessions.iter().filter_map(|s| s.game.strip_prefix("steam:")?.parse().ok()).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Every finished session in Steam's logs, oldest first, as `steam:<appid>` games.
pub fn read(logs: &Path) -> Vec<Session> {
    let mut text = String::new();
    for name in ["gameprocess_log.previous.txt", "gameprocess_log.txt"] {
        if let Ok(bytes) = std::fs::read(logs.join(name)) {
            text.push_str(&String::from_utf8_lossy(&bytes));
            text.push('\n');
        }
    }
    parse(&text, local_to_unix)
}

/// The sessions in log text; `to_unix` turns a local date and time into Unix seconds.
fn parse(text: &str, to_unix: impl Fn(&str) -> Option<i64>) -> Vec<Session> {
    let mut open: HashMap<u32, i64> = HashMap::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('[') else { continue };
        let Some((stamp, event)) = rest.split_once("] ") else { continue };
        let Some(at) = to_unix(stamp) else { continue };
        if let Some(tail) = event.strip_prefix("AppID ") {
            // "AppID 1091500 adding PID 1234 as a tracked process ..."
            let Some((id, what)) = tail.split_once(' ') else { continue };
            if what.starts_with("adding PID") {
                if let Ok(id) = id.parse::<u32>() {
                    let start = open.entry(id).or_insert(at);
                    // A start with no end for longer than a sitting: Steam missed that end.
                    if at - *start > MAX_SECS {
                        *start = at;
                    }
                }
            }
        } else if let Some(tail) = event.strip_prefix("Remove ") {
            // "Remove 1091500 from running list"
            let Some(id) = tail.strip_suffix(" from running list").and_then(|s| s.trim().parse::<u32>().ok()) else { continue };
            if let Some(start) = open.remove(&id) {
                if at - start >= MIN_SECS && at - start <= MAX_SECS {
                    out.push(Session { game: format!("steam:{id}"), start, end: at });
                }
            }
        }
    }
    out.sort_by_key(|s| s.start);
    out
}

/// Adds the sessions that aren't there yet: one overlapping a session of the same game already
/// in the log, such as one Mantel timed itself, is the same play. Returns how many were added.
pub fn merge(log: &mut Vec<Session>, found: Vec<Session>) -> usize {
    let before = log.len();
    for s in found {
        let known = log.iter().any(|k| k.game == s.game && k.start < s.end && s.start < k.end);
        if !known {
            log.push(s);
        }
    }
    let added = log.len() - before;
    if added > 0 {
        log.sort_by_key(|s| s.start);
    }
    added
}

/// "2025-07-17 19:18:13", in the PC's time zone with its summer time rules for that date.
#[cfg(windows)]
fn local_to_unix(stamp: &str) -> Option<i64> {
    use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows_sys::Win32::System::Time::{SystemTimeToFileTime, TzSpecificLocalTimeToSystemTime};
    let (y, mo, d, h, mi, s) = fields(stamp)?;
    let local = SYSTEMTIME { wYear: y, wMonth: mo, wDayOfWeek: 0, wDay: d, wHour: h, wMinute: mi, wSecond: s, wMilliseconds: 0 };
    let mut utc: SYSTEMTIME = unsafe { std::mem::zeroed() };
    let mut ft = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    unsafe {
        if TzSpecificLocalTimeToSystemTime(std::ptr::null(), &local, &mut utc) == 0 || SystemTimeToFileTime(&utc, &mut ft) == 0 {
            return None;
        }
    }
    let ticks = ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64;
    Some((ticks as i64 - 116_444_736_000_000_000) / 10_000_000)
}

/// Elsewhere (development) the stamp is read as UTC.
#[cfg(not(windows))]
fn local_to_unix(stamp: &str) -> Option<i64> {
    let (y, mo, d, h, mi, s) = fields(stamp)?;
    Some(days_from_civil(y as i64, mo as i64, d as i64) * 86400 + h as i64 * 3600 + mi as i64 * 60 + s as i64)
}

fn fields(stamp: &str) -> Option<(u16, u16, u16, u16, u16, u16)> {
    let (date, time) = stamp.trim().split_once(' ')?;
    let mut d = date.split('-').map(|x| x.parse::<u16>().ok());
    let mut t = time.split(':').map(|x| x.parse::<u16>().ok());
    Some((d.next()??, d.next()??, d.next()??, t.next()??, t.next()??, t.next()??))
}

#[cfg(any(test, not(windows)))]
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(stamp: &str) -> Option<i64> {
        let (y, mo, d, h, mi, s) = fields(stamp)?;
        Some(days_from_civil(y as i64, mo as i64, d as i64) * 86400 + h as i64 * 3600 + mi as i64 * 60 + s as i64)
    }

    const LOG: &str = "\
[2025-07-12 11:41:55] SSGL: change [2358050] LCT 762470754->0
[2025-07-12 12:00:00] AppID 2358050 adding PID 5792 as a tracked process \"C:\\Games\\a.exe\"
[2025-07-12 12:00:02] AppID 2358050 adding PID 5084 as a tracked process \"C:\\Games\\b.exe\"
[2025-07-12 12:26:02] AppID 2358050 no longer tracking PID 5792, exit code 0
[2025-07-12 12:26:02] AppID 2358050 no longer tracking PID 5084, exit code -1073740940
[2025-07-12 12:26:02] Remove 2358050 from running list
[2025-07-12 13:00:00] AppID 218 adding PID 1 as a tracked process
[2025-07-12 13:00:30] Remove 218 from running list
[2025-07-17 19:18:13] AppID 3199170 adding PID 8624 as a tracked process
[2025-07-17 19:41:54] Remove 3199170 from running list
[2025-07-18 10:00:00] AppID 9 adding PID 2 as a tracked process
";

    #[test]
    fn reads_sessions_from_start_to_removal() {
        let got = parse(LOG, utc);
        assert_eq!(got.len(), 2, "a 30 s launch and one still running are left out");
        assert_eq!(got[0].game, "steam:2358050");
        assert_eq!(got[0].end - got[0].start, 26 * 60 + 2);
        assert_eq!(got[1].game, "steam:3199170");
        assert_eq!(got[1].start, utc("2025-07-17 19:18:13").unwrap());
    }

    #[test]
    fn known_play_isnt_added_twice() {
        let mut log = vec![Session { game: "steam:2358050".into(), start: utc("2025-07-12 12:00:05").unwrap(), end: utc("2025-07-12 12:26:00").unwrap() }];
        let added = merge(&mut log, parse(LOG, utc));
        assert_eq!(added, 1);
        assert_eq!(log.len(), 2);
        assert!(log[0].start <= log[1].start);
        assert_eq!(merge(&mut log, parse(LOG, utc)), 0);
    }

    #[test]
    fn leaves_out_what_isnt_play() {
        let mut log = vec![
            Session { game: "steam:1".into(), start: 0, end: 3600 },
            Session { game: "steam:2".into(), start: 0, end: 3600 },
            Session { game: "steam:1".into(), start: 0, end: 30 * 3600 },
            Session { game: "xbox:x".into(), start: 0, end: 600 },
        ];
        let apps = HashMap::from([(1, AppInfo { name: "Game".into(), kind: "game".into(), at: 0 }), (2, AppInfo { name: "Wallpaper".into(), kind: "application".into(), at: 0 })]);
        assert!(prune(&mut log, &apps));
        assert_eq!(log.iter().map(|s| s.game.as_str()).collect::<Vec<_>>(), ["steam:1", "xbox:x"]);
        assert!(!prune(&mut log, &apps));
        assert_eq!(steam_ids(&log), [1]);
    }

    #[test]
    fn counts_days_like_unix() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 11, 17), 20774);
    }
}
