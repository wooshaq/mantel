//! Interface text. English is the source and the lookup key, as in the web build; the other
//! languages are its catalogs (locales/*.json, generated from src/locales/*.ts).

use std::collections::HashMap;
use std::sync::RwLock;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    Text(String),
    Plural { one: Option<String>, few: Option<String>, many: Option<String>, other: String },
}

type Catalog = HashMap<String, Entry>;

const CATALOGS: &[(&str, &str)] = &[
    ("pl", include_str!("../locales/pl.json")),
    ("de", include_str!("../locales/de.json")),
    ("es", include_str!("../locales/es.json")),
    ("fr", include_str!("../locales/fr.json")),
    ("ru", include_str!("../locales/ru.json")),
];

/// Settings value "" follows Windows. Labels are in their own language, as consoles do.
pub const LANGUAGES: &[(&str, &str)] = &[("", "System"), ("de", "Deutsch"), ("en", "English"), ("es", "Español"), ("fr", "Français"), ("pl", "Polski"), ("ru", "Русский")];

struct Current {
    lang: String,
    catalog: Option<Catalog>,
}

static CURRENT: RwLock<Current> = RwLock::new(Current { lang: String::new(), catalog: None });

/// The language Windows is in, when Mantel has it.
pub fn system_language() -> Option<String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;
        let mut buf = [0u16; 85];
        let n = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        if n > 1 {
            let name = String::from_utf16_lossy(&buf[..(n - 1) as usize]);
            let code = name.split('-').next().unwrap_or("").to_lowercase();
            return (code == "en" || CATALOGS.iter().any(|(l, _)| *l == code)).then_some(code);
        }
        None
    }
    #[cfg(not(windows))]
    {
        let env = std::env::var("MANTEL_LANG").or_else(|_| std::env::var("LANG")).unwrap_or_default();
        let code = env.split(['_', '.', '-']).next().unwrap_or("").to_lowercase();
        (code == "en" || CATALOGS.iter().any(|(l, _)| *l == code)).then_some(code)
    }
}

/// Switches to a language id, or "" for Windows' own. Returns the one in use.
pub fn set_language(id: &str) -> String {
    let lang = if id.is_empty() { system_language().unwrap_or_else(|| "en".into()) } else { id.to_string() };
    let catalog = CATALOGS.iter().find(|(l, _)| *l == lang).and_then(|(_, json)| serde_json::from_str::<Catalog>(json).ok());
    let lang = if catalog.is_some() || lang == "en" { lang } else { "en".into() };
    *CURRENT.write().unwrap() = Current { lang: lang.clone(), catalog };
    lang
}

pub fn language() -> String {
    CURRENT.read().unwrap().lang.clone()
}

/// Steam's name for the language, for store pages.
pub fn steam_language() -> &'static str {
    match language().as_str() {
        "fr" => "french",
        "de" => "german",
        "es" => "spanish",
        "pl" => "polish",
        "ru" => "russian",
        _ => "english",
    }
}

/// The locale tag for Microsoft's stores, like "pl-pl".
pub fn store_locale() -> &'static str {
    match language().as_str() {
        "fr" => "fr-fr",
        "de" => "de-de",
        "es" => "es-es",
        "pl" => "pl-pl",
        "ru" => "ru-ru",
        _ => "en-us",
    }
}

/// CLDR plural category for whole numbers in the current language.
fn category(lang: &str, n: i64) -> &'static str {
    let n = n.abs();
    let (m10, m100) = (n % 10, n % 100);
    match lang {
        "pl" => {
            if n == 1 {
                "one"
            } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
                "few"
            } else {
                "many"
            }
        }
        "ru" => {
            if m10 == 1 && m100 != 11 {
                "one"
            } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
                "few"
            } else {
                "many"
            }
        }
        "fr" => if n <= 1 { "one" } else { "other" },
        _ => if n == 1 { "one" } else { "other" },
    }
}

/// `text` in the interface language, with `{name}` placeholders filled from `vars`. A plural
/// entry picks its form by the `n` (or first numeric) variable.
pub fn t(text: &str, vars: &[(&str, String)]) -> String {
    let cur = CURRENT.read().unwrap();
    let mut s = match cur.catalog.as_ref().and_then(|c| c.get(text)) {
        Some(Entry::Text(s)) => s.clone(),
        Some(Entry::Plural { one, few, many, other }) => {
            let count = vars.iter().find(|(k, _)| *k == "n").or_else(|| vars.iter().find(|(_, v)| v.parse::<i64>().is_ok())).and_then(|(_, v)| v.parse::<i64>().ok());
            let form = match count.map(|n| category(&cur.lang, n)) {
                Some("one") => one.as_ref(),
                Some("few") => few.as_ref(),
                Some("many") => many.as_ref(),
                _ => None,
            };
            form.unwrap_or(other).clone()
        }
        None => text.to_string(),
    };
    for (k, v) in vars {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

/// `t` without variables.
pub fn tr(text: &str) -> String {
    t(text, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polish_plurals() {
        set_language("pl");
        assert_eq!(t("{n} games", &[("n", "1".into())]), "1 gra");
        assert_eq!(t("{n} games", &[("n", "3".into())]), "3 gry");
        assert_eq!(t("{n} games", &[("n", "5".into())]), "5 gier");
        assert_eq!(t("{n} games", &[("n", "22".into())]), "22 gry");
        assert_eq!(t("{n} games", &[("n", "12".into())]), "12 gier");
        assert_eq!(tr("Play"), "Graj");
        assert_eq!(tr("Not in any catalog"), "Not in any catalog");
        set_language("en");
        assert_eq!(t("{n} games", &[("n", "5".into())]), "5 games");
    }
}
