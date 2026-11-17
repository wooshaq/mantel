//! Optional store accounts: what never reaches the disk, like games owned but not
//! installed and playtime from other devices.
//!
//! Steam: the player signs in on Steam's own page in a Mantel window (email Steam Guard
//! works, no API key, no mobile authenticator). The session lives in the WebView's own
//! cookie jar. When Mantel needs data, a store page asks Steam for the short-lived Web API
//! token the store itself uses and hands it over by navigating to a bridge URL that Mantel
//! intercepts. Reading cookies directly is avoided on purpose: on Windows it runs a
//! nested message loop on the UI thread, which froze the sign-in window.

use std::time::Duration;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::host::{AppHandle, Manager};

use crate::library::steam_cdn_art;
use crate::model::{now, Achievement, Achievements, Game, Launch};
use crate::store::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedSteam {
    pub appid: u32,
    pub name: String,
    pub minutes: u64,
    pub last_played: i64,
    #[serde(default)]
    pub achievements: Option<Achievements>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mantel/0.1 (game launcher)")
        .build()
        .unwrap_or_default()
}

/// The Web API token is a JWT: its subject is the SteamID64, plus an expiry.
fn parse_token(jwt: &str) -> Option<(u64, i64)> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    let claims: Value = serde_json::from_slice(&bytes).ok()?;
    Some((claims.get("sub")?.as_str()?.parse().ok()?, claims.get("exp")?.as_i64()?))
}

fn save_token(app: &AppHandle, token: Option<&str>) {
    let state = app.state::<AppState>();
    let mut user = state.user.lock().unwrap();
    user.accounts.steam_signed_in = token.is_some();
    user.accounts.steam_token = token.unwrap_or_default().to_string();
    state.save_user(&user);
}

/// Signing in to Steam needs a browser window; the native build brings it back in a later stage.
pub async fn steam_sign_in(_app: &AppHandle) -> Result<(), String> {
    Err("Steam sign-in isn't in this build yet".into())
}

/// The saved Web API token while it's valid. Renewing it needs Steam's own page in a browser
/// window, which the native build doesn't have yet: an expired token means signed out.
async fn live_token(app: &AppHandle) -> Result<String, String> {
    let (signed_in, saved) = {
        let state = app.state::<AppState>();
        let user = state.user.lock().unwrap();
        (user.accounts.steam_signed_in, user.accounts.steam_token.clone())
    };
    if !signed_in {
        return Err("Not signed in to Steam".into());
    }
    if parse_token(&saved).is_some_and(|(_, exp)| exp > now() + 300) {
        return Ok(saved);
    }
    Err("Steam session expired: sign in again".into())
}

/// Every game on the signed-in account, with lifetime playtime across all devices.
pub async fn steam_owned(app: &AppHandle) -> Result<Vec<OwnedSteam>, String> {
    let token = live_token(app).await?;
    let (steam_id, _) = parse_token(&token).ok_or("Unreadable Steam token")?;
    let res = client()
        .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/")
        .query(&[
            ("access_token", token.as_str()),
            ("steamid", &steam_id.to_string()),
            ("include_appinfo", "1"),
            ("include_played_free_games", "1"),
        ])
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Steam: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("Steam answered with an error ({})", res.status().as_u16()));
    }
    let body: Value = res.json().await.map_err(|e| format!("Unexpected answer from Steam: {e}"))?;
    let games = body.pointer("/response/games").and_then(Value::as_array).cloned().unwrap_or_default();
    Ok(games
        .iter()
        .filter_map(|g| {
            Some(OwnedSteam {
                appid: g.get("appid")?.as_u64()? as u32,
                name: g.get("name")?.as_str()?.to_string(),
                minutes: g.get("playtime_forever").and_then(Value::as_u64).unwrap_or(0),
                last_played: g.get("rtime_last_played").and_then(Value::as_i64).unwrap_or(0),
                achievements: None,
            })
        })
        .collect())
}

/// Fills in achievement progress for the games the player has started. Games without
/// achievements come back with a total of 0 and stay empty.
pub async fn steam_achievements(app: &AppHandle, owned: &mut [OwnedSteam]) -> Result<(), String> {
    let token = live_token(app).await?;
    let (steam_id, _) = parse_token(&token).ok_or("Unreadable Steam token")?;
    let ids: Vec<u32> = owned.iter().filter(|o| o.minutes > 0).map(|o| o.appid).collect();
    let http = client();
    let batches = ids.chunks(100).map(|chunk| {
        let mut form = vec![("access_token".to_string(), token.clone()), ("steamid".into(), steam_id.to_string())];
        form.extend(chunk.iter().enumerate().map(|(i, id)| (format!("appids[{i}]"), id.to_string())));
        // POST only: a GET gets 405 with this token.
        let req = http.post("https://api.steampowered.com/IPlayerService/GetAchievementsProgress/v1/").form(&form);
        async move {
            let v: Value = req.send().await.and_then(reqwest::Response::error_for_status).map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
            Ok::<_, String>(v.pointer("/response/achievement_progress").and_then(Value::as_array).cloned().unwrap_or_default())
        }
    });
    let mut progress = std::collections::HashMap::new();
    let mut failed = None;
    for batch in futures::future::join_all(batches).await {
        match batch {
            Ok(list) => progress.extend(list.iter().filter_map(|p| Some((p.get("appid")?.as_u64()? as u32, steam_progress(p)?)))),
            Err(e) => failed = Some(e),
        }
    }
    if progress.is_empty() {
        if let Some(e) = failed {
            return Err(format!("Steam achievements: {e}"));
        }
    }
    for o in owned.iter_mut() {
        o.achievements = progress.get(&o.appid).cloned();
    }
    Ok(())
}

/// Every achievement of a Steam game with the player's progress, unlocked ones first. The
/// store token can't read the player's achievements directly, but it can read the game's full
/// list and the player's top achievements, which with a high enough limit are all of them.
pub async fn steam_achievement_list(app: &AppHandle, appid: u32) -> Result<Vec<Achievement>, String> {
    let token = live_token(app).await?;
    let (steam_id, _) = parse_token(&token).ok_or("Unreadable Steam token")?;
    let http = client();
    let get = |url: &str, query: Vec<(&str, String)>| {
        let req = http.get(url).query(&[("access_token", token.as_str()), ("language", "english")]).query(&query);
        async move {
            req.send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|e| format!("Couldn't get achievements from Steam: {e}"))?
                .json::<Value>()
                .await
                .map_err(|e| format!("Unexpected answer from Steam: {e}"))
        }
    };
    let (schema, mine) = futures::join!(
        get("https://api.steampowered.com/IPlayerService/GetGameAchievements/v1/", vec![("appid", appid.to_string())]),
        get(
            "https://api.steampowered.com/IPlayerService/GetTopAchievementsForGames/v1/",
            vec![("steamid", steam_id.to_string()), ("max_achievements", "1000".into()), ("appids[0]", appid.to_string())],
        ),
    );
    let (schema, mine) = (schema?, mine?);
    let unlocked: Vec<&str> = mine
        .pointer("/response/games/0/achievements")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x.get("icon")?.as_str()).collect())
        .unwrap_or_default();
    let mut list: Vec<Achievement> = schema
        .pointer("/response/achievements")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| steam_achievement(appid, x, &unlocked)).collect())
        .unwrap_or_default();
    list.sort_by_key(|a| !a.unlocked);
    Ok(list)
}

fn steam_achievement(appid: u32, a: &Value, unlocked: &[&str]) -> Option<Achievement> {
    let icon = a.get("icon").and_then(Value::as_str).unwrap_or_default();
    let got = !icon.is_empty() && unlocked.contains(&icon);
    let image = if got { icon } else { a.get("icon_gray").and_then(Value::as_str).unwrap_or(icon) };
    Some(Achievement {
        name: a.get("localized_name")?.as_str()?.to_string(),
        description: a.get("localized_desc").and_then(Value::as_str).unwrap_or_default().to_string(),
        icon: (!image.is_empty()).then(|| format!("https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{appid}/{image}")),
        unlocked: got,
        unlocked_at: None,
        rarity: a.get("player_percent_unlocked").and_then(Value::as_str).and_then(|p| p.parse().ok()),
        gamerscore: None,
        secret: !got && a.get("hidden").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn steam_progress(p: &Value) -> Option<Achievements> {
    let total = p.get("total")?.as_u64()? as u32;
    let unlocked = p.get("unlocked").and_then(Value::as_u64).unwrap_or(0) as u32;
    (total > 0).then(|| Achievements {
        unlocked,
        total,
        percent: (100.0 * unlocked as f64 / total as f64).round() as u8,
        gamerscore: None,
        gamerscore_total: None,
    })
}

/// Forgets the account here.
pub async fn steam_sign_out(app: &AppHandle) {
    save_token(app, None);
}

// ---- Xbox ---------------------------------------------------------------------------
//
// Microsoft sign-in in a Mantel window, then the usual Xbox Live hops: user token, then an
// XSTS token that the title and stats services accept.

/// Mantel's own app registration (public client, personal Microsoft accounts).
const XBOX_CLIENT_ID: &str = "21eaa04f-e328-4cb4-a047-f0f9175f3690";
/// The same string at every step: Live rejects a token request whose scopes differ in
/// any way from what the player consented to.
const XBOX_SCOPE: &str = "Xboxlive.signin Xboxlive.offline_access";
// The Live endpoints, not login.microsoftonline.com: personal-account app registrations
// count as "first party" there and can't be granted Xbox Live (AADSTS65002).
const LIVE_TOKEN: &str = "https://login.live.com/oauth20_token.srf";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct XboxSession {
    #[serde(with = "crate::store::secret")]
    pub token: String,
    pub uhs: String,
    pub xuid: String,
    pub gamertag: String,
    pub not_after: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XboxTitle {
    pub pfn: String,
    pub name: String,
    pub minutes: u64,
    pub last_played: i64,
    pub image: Option<String>,
    /// Store id like 9PCR3Z2MSM4T; the Xbox app opens its game page by it.
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub achievements: Option<Achievements>,
    /// Xbox Live's numeric title id, which the achievements service goes by.
    #[serde(default)]
    pub title_id: String,
}

fn ms_error(v: &Value) -> String {
    v.get("error_description")
        .or_else(|| v.get("error"))
        .and_then(Value::as_str)
        .map(|e| e.lines().next().unwrap_or(e).to_string())
        .unwrap_or_else(|| "Microsoft sign-in failed".into())
}

/// Signing in to Microsoft needs a browser window; the native build brings it back in a later stage.
pub async fn xbox_sign_in(_app: &AppHandle) -> Result<String, String> {
    Err("Microsoft sign-in isn't in this build yet".into())
}

/// Microsoft access token → Xbox Live user token → XSTS token for xboxlive.com.
async fn xbox_live(access_token: &str) -> Result<XboxSession, String> {
    let http = client();
    let user: Value = http
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("x-xbl-contract-version", "1")
        .json(&serde_json::json!({
            "Properties": { "AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com", "RpsTicket": format!("d={access_token}") },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Xbox Live: {e}"))?
        .json()
        .await
        .map_err(|_| "Xbox Live refused the Microsoft sign-in".to_string())?;
    let user_token = user.get("Token").and_then(Value::as_str).ok_or("Xbox Live refused the Microsoft sign-in")?;

    let res = http
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("x-xbl-contract-version", "1")
        .json(&serde_json::json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [user_token] },
            "RelyingParty": "http://xboxlive.com",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Xbox Live: {e}"))?;
    if res.status().as_u16() == 401 {
        // XErr codes: no Xbox profile yet, child account, region…
        let body: Value = res.json().await.unwrap_or_default();
        return Err(match body.get("XErr").and_then(Value::as_u64) {
            Some(2148916233) => "This Microsoft account has no Xbox profile yet. Create one at xbox.com first.".into(),
            Some(2148916238) => "Child accounts need a parent's permission for Xbox Live.".into(),
            _ => "Xbox Live refused the sign-in".into(),
        });
    }
    let xsts: Value = res.json().await.map_err(|e| e.to_string())?;
    let claims = xsts.pointer("/DisplayClaims/xui/0").ok_or("Xbox Live sent no profile")?;
    let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    Ok(XboxSession {
        token: s(&xsts, "Token"),
        uhs: s(claims, "uhs"),
        xuid: s(claims, "xid"),
        gamertag: s(claims, "gtg"),
        not_after: parse_iso(&s(&xsts, "NotAfter")).unwrap_or_else(|| now() + 3600),
    })
}

fn save_xbox(app: &AppHandle, refresh: &str, session: &XboxSession) {
    let state = app.state::<AppState>();
    let mut user = state.user.lock().unwrap();
    if !refresh.is_empty() {
        user.accounts.xbox_refresh = refresh.to_string();
    }
    user.accounts.xbox_gamertag = session.gamertag.clone();
    user.accounts.xbox_session = session.clone();
    state.save_user(&user);
}

/// A live XSTS session, refreshed through the Microsoft refresh token when it expires.
async fn xbox_session(app: &AppHandle) -> Result<XboxSession, String> {
    let (refresh, session) = {
        let state = app.state::<AppState>();
        let user = state.user.lock().unwrap();
        (user.accounts.xbox_refresh.clone(), user.accounts.xbox_session.clone())
    };
    if refresh.is_empty() {
        return Err("Not signed in to Xbox".into());
    }
    if session.not_after > now() + 300 && !session.token.is_empty() {
        return Ok(session);
    }
    let v: Value = client()
        .post(LIVE_TOKEN)
        .form(&[("grant_type", "refresh_token"), ("client_id", XBOX_CLIENT_ID), ("refresh_token", &refresh), ("scope", XBOX_SCOPE)])
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Microsoft: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let access = v.get("access_token").and_then(Value::as_str).ok_or_else(|| format!("Xbox sign-in expired: {}", ms_error(&v)))?;
    let next_refresh = v.get("refresh_token").and_then(Value::as_str).unwrap_or(&refresh).to_string();
    let session = xbox_live(access).await?;
    save_xbox(app, &next_refresh, &session);
    crate::log::info("xbox: session refreshed");
    Ok(session)
}

/// PC games on the account with last-played times and minutes played.
pub async fn xbox_titles(app: &AppHandle) -> Result<Vec<XboxTitle>, String> {
    let session = xbox_session(app).await?;
    let auth = format!("XBL3.0 x={};{}", session.uhs, session.token);
    let http = client();
    let history: Value = http
        .get(format!("https://titlehub.xboxlive.com/users/xuid({})/titles/titlehistory/decoration/detail,image,achievement", session.xuid))
        .header("Authorization", &auth)
        .header("x-xbl-contract-version", "2")
        .header("Accept-Language", "en-US")
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Xbox Live: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Unexpected answer from Xbox Live: {e}"))?;
    let titles: Vec<&Value> = history
        .get("titles")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter(|t| t.get("type").and_then(Value::as_str) == Some("Game"))
                .filter(|t| t.get("pfn").and_then(Value::as_str).is_some_and(|p| !p.is_empty()))
                .filter(|t| t.get("devices").and_then(Value::as_array).is_some_and(|d| d.iter().any(|x| x.as_str() == Some("PC"))))
                .collect()
        })
        .unwrap_or_default();

    let ids: Vec<&str> = titles.iter().filter_map(|t| t.get("titleId").and_then(Value::as_str)).collect();
    let minutes = if ids.is_empty() { Default::default() } else { xbox_minutes(&http, &auth, &session.xuid, &ids).await };

    Ok(titles
        .iter()
        .filter_map(|t| {
            let id = t.get("titleId").and_then(Value::as_str).unwrap_or_default();
            Some(XboxTitle {
                pfn: t.get("pfn")?.as_str()?.to_string(),
                name: t.get("name")?.as_str()?.replace("(PC)", "").trim().to_string(),
                minutes: minutes.get(id).copied().unwrap_or(0),
                last_played: t.pointer("/titleHistory/lastTimePlayed").and_then(Value::as_str).and_then(parse_iso).unwrap_or(0),
                image: t.get("displayImage").and_then(Value::as_str).map(String::from),
                product_id: t
                    .pointer("/detail/availabilities")
                    .and_then(Value::as_array)
                    .and_then(|a| a.iter().find_map(|x| x.get("ProductId")?.as_str().filter(|p| !p.is_empty())))
                    .map(String::from),
                achievements: t.get("achievement").and_then(xbox_achievements),
                title_id: id.to_string(),
            })
        })
        .collect())
}

/// Titlehub's `achievement` decoration. Newer titles often report no achievement total,
/// only gamerscore, so the percentage comes from Xbox Live itself.
fn xbox_achievements(a: &Value) -> Option<Achievements> {
    let n = |k: &str| a.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
    let (unlocked, total, score, score_total) = (n("currentAchievements"), n("totalAchievements"), n("currentGamerscore"), n("totalGamerscore"));
    if total == 0 && score_total == 0 {
        return None;
    }
    let percent = a.get("progressPercentage").and_then(Value::as_f64).unwrap_or_else(|| {
        if score_total > 0 { 100.0 * score as f64 / score_total as f64 } else { 100.0 * unlocked as f64 / total as f64 }
    });
    Some(Achievements {
        unlocked,
        total,
        percent: percent.round().clamp(0.0, 100.0) as u8,
        gamerscore: (score_total > 0).then_some(score),
        gamerscore_total: (score_total > 0).then_some(score_total),
    })
}

/// Titlehub leaves the achievement count out for most newer titles. Counts learned before
/// carry over, and installed games, the ones Home shows, get theirs from their list.
pub async fn xbox_counts(app: &AppHandle, titles: &mut [XboxTitle], known: &[XboxTitle], installed: &[String]) {
    for t in titles.iter_mut() {
        let Some(a) = t.achievements.as_mut().filter(|a| a.total == 0) else { continue };
        let total = known.iter().find(|k| k.title_id == t.title_id).and_then(|k| k.achievements.as_ref()).map(|k| k.total);
        if let Some(total) = total.filter(|&n| n > 0) {
            a.count(a.unlocked, total);
        }
    }
    let todo: Vec<String> = titles
        .iter()
        .filter(|t| installed.iter().any(|pfn| pfn.eq_ignore_ascii_case(&t.pfn)) && t.achievements.as_ref().is_some_and(|a| a.total == 0))
        .map(|t| t.title_id.clone())
        .collect();
    let lists = futures::future::join_all(todo.iter().map(|id| xbox_achievement_list(app, id))).await;
    for (id, list) in todo.iter().zip(lists) {
        match list {
            Ok(list) if !list.is_empty() => {
                if let Some(a) = titles.iter_mut().find(|t| &t.title_id == id).and_then(|t| t.achievements.as_mut()) {
                    a.count(list.iter().filter(|x| x.unlocked).count() as u32, list.len() as u32);
                }
            }
            Ok(_) => {}
            Err(e) => crate::log::warn(&format!("xbox achievements for {id}: {e}")),
        }
    }
}

/// Every achievement of an Xbox title with the player's progress, unlocked ones first.
pub async fn xbox_achievement_list(app: &AppHandle, title_id: &str) -> Result<Vec<Achievement>, String> {
    let session = xbox_session(app).await?;
    let v: Value = client()
        .get(format!("https://achievements.xboxlive.com/users/xuid({})/achievements", session.xuid))
        .query(&[("titleId", title_id), ("maxItems", "1000")])
        .header("Authorization", format!("XBL3.0 x={};{}", session.uhs, session.token))
        .header("x-xbl-contract-version", "2")
        .header("Accept-Language", "en-US")
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("Couldn't get achievements from Xbox Live: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Unexpected answer from Xbox Live: {e}"))?;
    let mut list: Vec<Achievement> = v.get("achievements").and_then(Value::as_array).map(|a| a.iter().filter_map(xbox_achievement).collect()).unwrap_or_default();
    list.sort_by_key(|a| (!a.unlocked, -a.unlocked_at.unwrap_or(0)));
    Ok(list)
}

fn xbox_achievement(a: &Value) -> Option<Achievement> {
    let unlocked = a.get("progressState")?.as_str()? == "Achieved";
    let text = |k: &str| a.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    Some(Achievement {
        name: a.get("name")?.as_str()?.to_string(),
        description: text(if unlocked { "description" } else { "lockedDescription" }),
        icon: a
            .get("mediaAssets")
            .and_then(Value::as_array)
            .and_then(|m| m.iter().find(|x| x.get("type").and_then(Value::as_str) == Some("Icon")))
            .and_then(|x| x.get("url")?.as_str())
            .map(String::from),
        unlocked,
        // Locked ones carry year 1 here.
        unlocked_at: a.pointer("/progression/timeUnlocked").and_then(Value::as_str).and_then(parse_iso).filter(|&t| unlocked && t > 0),
        rarity: a.pointer("/rarity/currentPercentage").and_then(Value::as_f64).map(|r| r as f32),
        gamerscore: a
            .get("rewards")
            .and_then(Value::as_array)
            .and_then(|r| r.iter().find(|x| x.get("type").and_then(Value::as_str) == Some("Gamerscore")))
            .and_then(|x| x.get("value")?.as_str()?.parse().ok()),
        secret: !unlocked && a.get("isSecret").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// Minutes played per title id. Xbox Live answers slower the more titles one request names
/// (246 took 17 s, past the client's timeout), so they go in batches, all at once.
async fn xbox_minutes(http: &reqwest::Client, auth: &str, xuid: &str, ids: &[&str]) -> std::collections::HashMap<String, u64> {
    let batches = ids.chunks(50).map(|chunk| xbox_minutes_batch(http, auth, xuid, chunk));
    let mut minutes = std::collections::HashMap::new();
    for batch in futures::future::join_all(batches).await {
        match batch {
            Ok(found) => minutes.extend(found),
            Err(e) => crate::log::warn(&format!("xbox playtime: {e}")),
        }
    }
    minutes
}

async fn xbox_minutes_batch(http: &reqwest::Client, auth: &str, xuid: &str, ids: &[&str]) -> Result<Vec<(String, u64)>, String> {
    let body = serde_json::json!({
        "arrangebyfield": "xuid",
        "xuids": [xuid],
        "stats": ids.iter().map(|id| serde_json::json!({ "name": "MinutesPlayed", "titleid": id })).collect::<Vec<_>>(),
    });
    let v: Value = http
        .post("https://userstats.xboxlive.com/batch")
        .header("Authorization", auth)
        .header("x-xbl-contract-version", "2")
        .json(&body)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(v.pointer("/statlistscollection/0/stats")
        .and_then(Value::as_array)
        .map(|stats| {
            stats
                .iter()
                .filter_map(|s| {
                    let id = s.get("titleid")?.as_str()?.to_string();
                    let value = s.get("value")?.as_str()?.parse().ok()?;
                    Some((id, value))
                })
                .collect()
        })
        .unwrap_or_default())
}

/// Title history links its art over plain http, which the page's content policy refuses, and at
/// full size, a megabyte or more per cover.
fn cover_url(image: &str) -> String {
    let url = image.strip_prefix("http://").map_or_else(|| image.to_string(), |rest| format!("https://{rest}"));
    if url.contains("store-images.") && !url.contains('?') {
        format!("{url}?w=600")
    } else {
        url
    }
}

/// Installed games gain account playtime; the rest open their page in the Xbox app to
/// install, or the Microsoft Store when Live sent no store id.
pub fn merge_xbox(games: &mut Vec<Game>, titles: &[XboxTitle]) {
    for t in titles {
        let last = (t.last_played > 0).then_some(t.last_played);
        if let Some(g) = games.iter_mut().find(|g| g.source == "xbox" && g.source_id.eq_ignore_ascii_case(&t.pfn)) {
            g.playtime_minutes = g.playtime_minutes.max(t.minutes);
            g.last_played = g.last_played.max(last);
            g.achievements = t.achievements.clone();
            continue;
        }
        let uri = match &t.product_id {
            Some(id) => format!("msxbox://game/?productId={id}"),
            None => format!("ms-windows-store://pdp/?PFN={}", t.pfn),
        };
        let mut g = Game::new("xbox", t.pfn.clone(), t.name.clone(), Launch::Uri { uri });
        g.installed = false;
        g.playtime_minutes = t.minutes;
        g.last_played = last;
        g.art.cover.extend(t.image.as_deref().map(cover_url));
        g.achievements = t.achievements.clone();
        games.push(g);
    }
}

/// `2026-10-28T12:34:56.1234567Z` → unix seconds (UTC only, which is all Xbox Live sends).
fn parse_iso(s: &str) -> Option<i64> {
    let (date, time) = s.split_once('T')?;
    let mut d = date.split('-').map(|x| x.parse::<i64>());
    let (y, m, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
    let mut t = time.trim_end_matches('Z').split(':');
    let (h, min) = (t.next()?.parse::<i64>().ok()?, t.next()?.parse::<i64>().ok()?);
    let sec = t.next().and_then(|x| x.split('.').next()?.parse::<i64>().ok()).unwrap_or(0);
    // Days from civil (Howard Hinnant).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146097 + doe - 719468) * 86400 + h * 3600 + min * 60 + sec)
}

/// Steam came out in 2003. For games last played before it kept track it answers with a date
/// next to 1970, which only means it doesn't know.
const STEAM_BEGAN: i64 = 1_063_000_000;

/// Folds account data into a scan: better playtime for installed games, and entries
/// for the rest that open Steam's install dialog.
pub fn merge_steam(games: &mut Vec<Game>, owned: &[OwnedSteam]) {
    for o in owned {
        let id = format!("steam:{}", o.appid);
        let last = (o.last_played > STEAM_BEGAN).then_some(o.last_played);
        if let Some(g) = games.iter_mut().find(|g| g.id == id) {
            g.playtime_minutes = g.playtime_minutes.max(o.minutes);
            g.last_played = g.last_played.max(last);
            g.achievements = o.achievements.clone();
            continue;
        }
        let mut g = Game::new("steam", o.appid.to_string(), o.name.clone(), Launch::Uri { uri: format!("steam://install/{}", o.appid) });
        g.installed = false;
        g.steam_app_id = Some(o.appid);
        g.art = steam_cdn_art(o.appid);
        g.playtime_minutes = o.minutes;
        g.last_played = last;
        g.achievements = o.achievements.clone();
        games.push(g);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_for_covers_over_https_at_their_size() {
        assert_eq!(cover_url("http://store-images.s-microsoft.com/image/apps.1.2"), "https://store-images.s-microsoft.com/image/apps.1.2?w=600");
        assert_eq!(cover_url("https://store-images.microsoft.com/image/apps.1.2"), "https://store-images.microsoft.com/image/apps.1.2?w=600");
        assert_eq!(cover_url("http://images-eds.xboxlive.com/image?url=abc"), "https://images-eds.xboxlive.com/image?url=abc");
    }

    #[test]
    fn takes_steams_dates_near_1970_as_unknown() {
        let owned = |last_played| OwnedSteam { appid: 400, name: "Portal".into(), minutes: 22, last_played, achievements: None };
        let mut games = Vec::new();
        merge_steam(&mut games, &[owned(86400)]);
        assert_eq!(games[0].last_played, None);
        let mut games = Vec::new();
        merge_steam(&mut games, &[owned(1_700_000_000)]);
        assert_eq!(games[0].last_played, Some(1_700_000_000));
    }

    #[test]
    fn parses_xbox_dates() {
        assert_eq!(parse_iso("1970-01-02T00:00:00Z"), Some(86400));
        assert_eq!(parse_iso("2026-10-28T12:34:56.1234567Z"), Some(1793190896));
    }

    #[test]
    fn reads_xbox_achievements() {
        let a = |v: Value| xbox_achievements(&v);
        let full = a(serde_json::json!({"currentAchievements": 12, "totalAchievements": 50, "currentGamerscore": 230, "totalGamerscore": 1000, "progressPercentage": 23}));
        assert_eq!(full, Some(Achievements { unlocked: 12, total: 50, percent: 23, gamerscore: Some(230), gamerscore_total: Some(1000) }));
        let no_total = a(serde_json::json!({"currentAchievements": 3, "totalAchievements": 0, "currentGamerscore": 45, "totalGamerscore": 1000}));
        assert_eq!(no_total.map(|x| (x.total, x.percent)), Some((0, 5)));
        assert_eq!(a(serde_json::json!({"currentAchievements": 0, "totalAchievements": 0, "currentGamerscore": 0, "totalGamerscore": 0})), None);
    }

    #[test]
    fn reads_steam_progress() {
        let p = steam_progress(&serde_json::json!({"appid": 730, "unlocked": 1, "total": 3, "percentage": 33.3}));
        assert_eq!(p.map(|a| (a.unlocked, a.total, a.percent)), Some((1, 3, 33)));
        assert_eq!(steam_progress(&serde_json::json!({"appid": 1, "unlocked": 0, "total": 0})), None);
    }

    #[test]
    fn reads_achievement_lists() {
        let x = xbox_achievement(&serde_json::json!({
            "name": "Emergence Begins", "description": "Unlock during Act 1.", "lockedDescription": "Hidden",
            "isSecret": true, "progressState": "Achieved",
            "progression": {"timeUnlocked": "1970-01-02T00:00:00.0000000Z"},
            "mediaAssets": [{"type": "Icon", "url": "https://img/x"}],
            "rewards": [{"type": "Gamerscore", "value": "15"}], "rarity": null
        }))
        .unwrap();
        assert_eq!((x.unlocked, x.unlocked_at, x.gamerscore, x.secret, x.icon.as_deref()), (true, Some(86400), Some(15), false, Some("https://img/x")));
        let locked = xbox_achievement(&serde_json::json!({
            "name": "Later", "lockedDescription": "Keep going", "isSecret": true, "progressState": "NotStarted",
            "progression": {"timeUnlocked": "0001-01-01T00:00:00.0000000Z"}
        }))
        .unwrap();
        assert_eq!((locked.unlocked, locked.unlocked_at, locked.secret, locked.description.as_str()), (false, None, true, "Keep going"));

        let s = serde_json::json!({"localized_name": "Timber!!", "localized_desc": "Chop", "icon": "a.jpg", "icon_gray": "b.jpg", "hidden": false, "player_percent_unlocked": "87.0"});
        let got = steam_achievement(105600, &s, &["a.jpg"]).unwrap();
        assert!(got.unlocked && got.icon.unwrap().ends_with("/105600/a.jpg"));
        let not = steam_achievement(105600, &s, &[]).unwrap();
        assert!(!not.unlocked && not.icon.unwrap().ends_with("/b.jpg") && not.rarity == Some(87.0));
    }

    #[test]
    fn reads_web_api_token() {
        let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"sub":"76561198000000000","exp":2000000000}"#);
        assert_eq!(parse_token(&format!("h.{claims}.s")), Some((76561198000000000, 2000000000)));
    }
}
