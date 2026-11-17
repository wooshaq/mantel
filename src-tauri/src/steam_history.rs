//! Past Steam sessions, read from the log the Steam client keeps of the games it runs
//! (`logs/gameprocess_log.txt`, and the older part in `gameprocess_log.previous.txt`). Each game
//! start and stop is there with its local date and time, as far back as Steam keeps the files.
//! Mantel only reads them.

use std::collections::HashMap;
use std::path::Path;

use crate::store::Session;

/// Sessions shorter than this are launchers that closed at once, not play.
const MIN_SECS: i64 = 60;

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
                    open.entry(id).or_insert(at);
                }
            }
        } else if let Some(tail) = event.strip_prefix("Remove ") {
            // "Remove 1091500 from running list"
            let Some(id) = tail.strip_suffix(" from running list").and_then(|s| s.trim().parse::<u32>().ok()) else { continue };
            if let Some(start) = open.remove(&id) {
                if at - start >= MIN_SECS {
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
    fn counts_days_like_unix() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 11, 17), 20774);
    }
}
