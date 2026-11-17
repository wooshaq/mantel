//! How long a game takes, from HowLongToBeat. It has no public API and its search turns away
//! anything that isn't a browser, so the player links each game once by pasting the address
//! of its page; Mantel then reads the times from that public page.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Times to beat a game, in seconds as HowLongToBeat gives them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Hltb {
    pub id: u32,
    pub name: String,
    /// Main story.
    pub main: u32,
    /// Main story and extras.
    pub plus: u32,
    /// Everything.
    pub full: u32,
    /// When it was read.
    pub at: i64,
}

/// How long the times are trusted: they move slowly as people submit.
pub const FRESH_SECS: i64 = 14 * 24 * 3600;

/// The game id in a page address ("https://howlongtobeat.com/game/10270") or a bare number.
pub fn parse_id(input: &str) -> Option<u32> {
    let s = input.trim().trim_end_matches('/');
    let tail = s.rsplit_once("/game/").map(|(_, t)| t).unwrap_or(s);
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok().filter(|n| *n > 0)
}

/// Reads a game's times from its page.
pub async fn fetch(id: u32) -> Result<Hltb, String> {
    let html = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Mantel/0.1 (game launcher)")
        .build()
        .unwrap_or_default()
        .get(format!("https://howlongtobeat.com/game/{id}"))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("Couldn't reach HowLongToBeat: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Couldn't reach HowLongToBeat: {e}"))?;
    parse_page(&html, id).ok_or_else(|| "HowLongToBeat has no times for that game".into())
}

fn parse_page(html: &str, id: u32) -> Option<Hltb> {
    const START: &str = "<script id=\"__NEXT_DATA__\" type=\"application/json\">";
    let from = html.find(START)? + START.len();
    let to = from + html[from..].find("</script>")?;
    let data: Value = serde_json::from_str(&html[from..to]).ok()?;
    let game = data.pointer("/props/pageProps/game/data/game/0")?;
    let secs = |key: &str| game.get(key).and_then(Value::as_u64).unwrap_or(0) as u32;
    let found = Hltb {
        id,
        name: game.get("game_name").and_then(Value::as_str).unwrap_or_default().to_string(),
        main: secs("comp_main"),
        plus: secs("comp_plus"),
        full: secs("comp_100"),
        at: crate::model::now(),
    };
    (found.main > 0 || found.plus > 0 || found.full > 0).then_some(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_links_and_numbers() {
        assert_eq!(parse_id("https://howlongtobeat.com/game/10270"), Some(10270));
        assert_eq!(parse_id("howlongtobeat.com/game/10270/"), Some(10270));
        assert_eq!(parse_id("https://howlongtobeat.com/game/10270?tab=x"), Some(10270));
        assert_eq!(parse_id("10270"), Some(10270));
        assert_eq!(parse_id("https://howlongtobeat.com/"), None);
    }

    #[test]
    fn reads_the_times_from_the_page() {
        let html = r#"<html><script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{"game":{"data":{"game":[{"game_name":"Witcher","comp_main":185989,"comp_plus":373618,"comp_100":631298}]}}}}}</script></html>"#;
        let h = parse_page(html, 10270).unwrap();
        assert_eq!((h.id, h.name.as_str(), h.main, h.plus, h.full), (10270, "Witcher", 185989, 373618, 631298));
        assert!(parse_page("<html></html>", 1).is_none());
    }
}

/// Runs in the HowLongToBeat window: the site changes pages without loading them, so a move to
/// a game's page is passed on as a navigation Mantel can see.
const WATCH_GAME_PAGE: &str = "(function(){\
var done=false;\
function check(){if(done)return;var m=location.pathname.match(/^\\/game\\/(\\d+)/);if(m){done=true;location.replace('https://mantel.invalid/hltb#'+m[1]);}}\
['pushState','replaceState'].forEach(function(k){var o=history[k];history[k]=function(){var r=o.apply(this,arguments);setTimeout(check,0);return r;};});\
addEventListener('popstate',check);check();})();";

/// Opens HowLongToBeat's own search for a game in a window over Mantel. The player picks the
/// game there, and the id of the page they open comes back; Mantel reads nothing else.
pub async fn pick(app: &tauri::AppHandle, query: &str) -> Result<u32, String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
    const LABEL: &str = "hltb";
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.set_focus();
        return Err("HowLongToBeat is already open".into());
    }
    let url = tauri::Url::parse_with_params("https://howlongtobeat.com/", &[("q", query)]).map_err(|e| e.to_string())?;
    let (tx, rx) = std::sync::mpsc::channel::<Option<u32>>();
    let tx_nav = tx.clone();
    let mut builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(url));
    // Same WebView2 options as the main window: environments with different flags can't coexist.
    if let Some(args) = app.config().app.windows.first().and_then(|w| w.additional_browser_args.clone()) {
        builder = builder.additional_browser_args(&args);
    }
    let w = builder
        .title("HowLongToBeat")
        .inner_size(1100.0, 800.0)
        .center()
        .always_on_top(true)
        .initialization_script(WATCH_GAME_PAGE)
        .on_navigation(move |url| {
            let id = match url.host_str() {
                Some("mantel.invalid") => url.fragment().and_then(parse_id),
                Some(h) if h.ends_with("howlongtobeat.com") && url.path().starts_with("/game/") => parse_id(url.path()),
                _ => return true,
            };
            match id {
                Some(id) => {
                    let _ = tx_nav.send(Some(id));
                    false
                }
                None => url.host_str() != Some("mantel.invalid"),
            }
        })
        .build()
        .map_err(|e| e.to_string())?;
    w.on_window_event(move |e| {
        if matches!(e, WindowEvent::Destroyed) {
            let _ = tx.send(None);
        }
    });
    let got = tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(std::time::Duration::from_secs(15 * 60)).ok().flatten())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.close();
    }
    got.ok_or_else(|| "closed".into())
}
