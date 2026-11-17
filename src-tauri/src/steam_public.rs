//! Steam achievements from the player's public profile, without signing in: Steam's Web API
//! answers anyone for a profile whose game details are public. The SteamID comes from the
//! settings, or from the account Steam last signed in with on this PC.

use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{now, Achievement, Achievements};

const API: &str = "https://api.steampowered.com/IPlayerService";
/// How long a game's progress is trusted before Steam is asked again.
const FRESH_SECS: i64 = 6 * 3600;
/// The list of a game's achievements changes rarely: patches add some.
const SCHEMA_SECS: i64 = 7 * 24 * 3600;
/// Requests at a time, so a big library doesn't flood Steam.
const PARALLEL: usize = 4;
const STEAMID64_BASE: u64 = 76561197960265728;

/// A game's progress on the profile, as last seen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Progress {
    pub at: i64,
    /// None when the profile hides its games or the game has no achievements.
    pub achievements: Option<Achievements>,
    /// How many achievements the game has, and when that was last asked.
    pub total: u32,
    pub total_at: i64,
}

/// What the player's profile shows, for the settings page.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub steam_id: String,
    pub name: String,
    /// False when the profile or its game details are private.
    pub public: bool,
    /// Taken from Steam on this PC rather than typed in.
    pub detected: bool,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mantel/0.1 (game launcher)")
        .build()
        .unwrap_or_default()
}

/// A SteamID64 from what the player typed: the number, a profile link, or a link with a
/// custom name, which Steam's public profile page turns into the number.
pub async fn parse_id(input: &str) -> Result<u64, String> {
    let s = input.trim().trim_end_matches('/');
    if let Some(id) = number(s) {
        return Ok(id);
    }
    let vanity = s.rsplit_once("/id/").map(|(_, v)| v).or_else(|| (!s.contains('/') && !s.is_empty()).then_some(s));
    let Some(vanity) = vanity.filter(|v| v.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')) else {
        return Err("That doesn't look like a Steam profile".into());
    };
    let xml = client()
        .get(format!("https://steamcommunity.com/id/{vanity}/?xml=1"))
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Steam: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Couldn't reach Steam: {e}"))?;
    tag(&xml, "steamID64").and_then(|t| number(&t)).ok_or_else(|| "No Steam profile by that name".into())
}

fn number(s: &str) -> Option<u64> {
    let digits = s.rsplit_once("/profiles/").map(|(_, d)| d).unwrap_or(s);
    digits.parse::<u64>().ok().filter(|id| *id > STEAMID64_BASE)
}

/// The text of `<name>…</name>`, without a CDATA wrapper.
fn tag(xml: &str, name: &str) -> Option<String> {
    let start = xml.find(&format!("<{name}>"))? + name.len() + 2;
    let end = start + xml[start..].find(&format!("</{name}>"))?;
    let inner = xml[start..end].trim();
    Some(inner.strip_prefix("<![CDATA[").and_then(|s| s.strip_suffix("]]>")).unwrap_or(inner).to_string())
}

/// Name and privacy of a profile, from its public page.
pub async fn profile(id: u64, detected: bool) -> Profile {
    let xml = async { client().get(format!("https://steamcommunity.com/profiles/{id}/?xml=1")).send().await.ok()?.text().await.ok() }.await.unwrap_or_default();
    Profile {
        steam_id: id.to_string(),
        name: tag(&xml, "steamID").unwrap_or_default(),
        public: tag(&xml, "privacyState").is_some_and(|p| p == "public"),
        detected,
    }
}

async fn get(url: &str, query: &[(&str, String)]) -> Result<Value, String> {
    client()
        .get(url)
        .query(query)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("Couldn't get achievements from Steam: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("Unexpected answer from Steam: {e}"))
}

/// The player's unlocked achievements in a game, by Steam's key for each, with when. None
/// when Steam shows nothing: a private profile, or a game not on the account.
async fn unlocked(steam_id: u64, appid: u32) -> Result<Option<HashMap<u64, i64>>, String> {
    let v = get(&format!("{API}/GetUserAchievements/v1/"), &[("steamid", steam_id.to_string()), ("appid", appid.to_string())]).await?;
    Ok(parse_unlocked(&v))
}

fn parse_unlocked(v: &Value) -> Option<HashMap<u64, i64>> {
    let r = v.get("response")?;
    // A game the profile shows always comes with its schema's hash, even with nothing unlocked.
    r.get("schema_hash")?;
    Some(
        r.get("achievements")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter(|x| x.get("unlocked").and_then(Value::as_bool).unwrap_or(false))
                    .filter_map(|x| Some((x.get("internal_key")?.as_u64()?, x.get("unlock_time").and_then(Value::as_i64).unwrap_or(0))))
                    .collect()
            })
            .unwrap_or_default(),
    )
}

async fn schema(appid: u32, language: &str) -> Result<Vec<Value>, String> {
    let v = get(&format!("{API}/GetGameAchievements/v1/"), &[("appid", appid.to_string()), ("language", language.to_string())]).await?;
    Ok(v.pointer("/response/achievements").and_then(Value::as_array).cloned().unwrap_or_default())
}

fn counted(schema: &[Value]) -> u32 {
    schema.iter().filter(|a| !a.get("archived").and_then(Value::as_bool).unwrap_or(false)).count() as u32
}

/// Progress for each game, asking Steam only about games whose last answer has gone stale.
/// Returns the games it has new answers for.
pub async fn refresh(steam_id: u64, appids: Vec<u32>, known: &HashMap<u32, Progress>) -> HashMap<u32, Progress> {
    let t = now();
    let stale: Vec<(u32, Progress)> = appids
        .into_iter()
        .filter_map(|id| {
            let p = known.get(&id).cloned().unwrap_or_default();
            (t - p.at > FRESH_SECS).then_some((id, p))
        })
        .collect();
    futures::stream::iter(stale)
        .map(|(appid, mut p)| async move {
            let mine = match unlocked(steam_id, appid).await {
                Ok(m) => m,
                Err(e) => {
                    crate::log::warn(&format!("steam profile {appid}: {e}"));
                    return None;
                }
            };
            if mine.is_some() && (p.total == 0 || t - p.total_at > SCHEMA_SECS) {
                if let Ok(s) = schema(appid, "english").await {
                    p.total = counted(&s);
                    p.total_at = t;
                }
            }
            p.at = t;
            p.achievements = mine.and_then(|m| progress(m.len() as u32, p.total));
            Some((appid, p))
        })
        .buffer_unordered(PARALLEL)
        .filter_map(|x| async { x })
        .collect()
        .await
}

fn progress(unlocked: u32, total: u32) -> Option<Achievements> {
    (total > 0).then(|| {
        let mut a = Achievements::default();
        a.count(unlocked.min(total), total);
        a
    })
}

/// Every achievement of a game, unlocked ones first, newest first, with when. Err when the
/// profile shows nothing for the game.
pub async fn list(steam_id: u64, appid: u32, language: &str) -> Result<(Vec<Achievement>, Achievements), String> {
    let (mine, schema) = futures::join!(unlocked(steam_id, appid), schema(appid, language));
    let mine = mine?.ok_or("Your Steam profile doesn't show this game: make its game details public")?;
    let schema = schema?;
    let mut list: Vec<Achievement> = schema
        .iter()
        .filter_map(|a| {
            let key = a.get("internal_key")?.as_u64()?;
            let at = mine.get(&key).copied();
            if at.is_none() && a.get("archived").and_then(Value::as_bool).unwrap_or(false) {
                return None;
            }
            achievement(appid, a, at)
        })
        .collect();
    list.sort_by_key(|a| (!a.unlocked, std::cmp::Reverse(a.unlocked_at.unwrap_or(0))));
    let total = counted(&schema);
    let got = list.iter().filter(|a| a.unlocked).count() as u32;
    Ok((list, progress(got, total).unwrap_or_default()))
}

fn achievement(appid: u32, a: &Value, unlocked_at: Option<i64>) -> Option<Achievement> {
    let got = unlocked_at.is_some();
    let icon = a.get("icon").and_then(Value::as_str).unwrap_or_default();
    let image = if got { icon } else { a.get("icon_gray").and_then(Value::as_str).unwrap_or(icon) };
    Some(Achievement {
        name: a.get("localized_name")?.as_str()?.to_string(),
        description: a.get("localized_desc").and_then(Value::as_str).unwrap_or_default().to_string(),
        icon: (!image.is_empty()).then(|| format!("https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{appid}/{image}")),
        unlocked: got,
        // Steam gives 0 for achievements from before it kept the time.
        unlocked_at: unlocked_at.filter(|t| *t > 0),
        rarity: a.get("player_percent_unlocked").and_then(Value::as_str).and_then(|p| p.parse().ok()),
        gamerscore: None,
        // Hidden ones show their name and description too: Steam sends both.
        secret: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_ids_and_profile_links() {
        assert_eq!(number("76561198006409530"), Some(76561198006409530));
        assert_eq!(number("https://steamcommunity.com/profiles/76561198006409530"), Some(76561198006409530));
        assert_eq!(number("12345"), None);
    }

    #[test]
    fn reads_tags_with_and_without_cdata() {
        let xml = "<profile><steamID64>76561198006409530</steamID64><steamID><![CDATA[Robin]]></steamID></profile>";
        assert_eq!(tag(xml, "steamID64").as_deref(), Some("76561198006409530"));
        assert_eq!(tag(xml, "steamID").as_deref(), Some("Robin"));
    }

    #[test]
    fn a_private_profile_is_told_apart_from_nothing_unlocked() {
        assert_eq!(parse_unlocked(&json!({"response": {}})), None);
        assert_eq!(parse_unlocked(&json!({"response": {"schema_hash": 1, "groups": []}})), Some(HashMap::new()));
        let got = parse_unlocked(&json!({"response": {"schema_hash": 1, "achievements": [
            {"internal_key": 1, "unlocked": true, "unlock_time": 1303246011},
            {"internal_key": 2, "unlocked": false, "unlock_time": 0}
        ]}}))
        .unwrap();
        assert_eq!(got, HashMap::from([(1, 1303246011)]));
    }

    #[test]
    fn archived_achievements_count_only_when_unlocked() {
        let schema = vec![
            json!({"internal_key": 1, "localized_name": "A", "icon": "a.jpg", "icon_gray": "a_bw.jpg", "archived": false}),
            json!({"internal_key": 2, "localized_name": "B", "archived": true}),
        ];
        assert_eq!(counted(&schema), 1);
        let a = achievement(620, &schema[0], Some(5)).unwrap();
        assert!(a.unlocked && a.unlocked_at == Some(5) && a.icon.unwrap().ends_with("/620/a.jpg"));
        let b = achievement(620, &schema[0], None).unwrap();
        assert!(!b.unlocked && b.icon.unwrap().ends_with("a_bw.jpg"));
    }

    #[test]
    fn progress_needs_a_total() {
        assert!(progress(3, 0).is_none());
        let p = progress(42, 51).unwrap();
        assert_eq!((p.unlocked, p.total, p.percent), (42, 51, 82));
    }
}
