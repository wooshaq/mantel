//! The window and everything between it and the backend: turning games into cards, loading
//! their pictures, launching, favorites, and the events the tracker sends back.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel};

use crate::host::{AppHandle, Manager};
use crate::i18n::{t, tr};
use crate::images::{self, Fit};
use crate::model::{now, Game};
use crate::store::AppState;
use crate::{AppWindow, Card, Focused, Tr};

thread_local! {
    /// The games the window shows, as the backend last gave them.
    static GAMES: RefCell<Vec<Game>> = const { RefCell::new(Vec::new()) };
    /// The game launched from Mantel, while its overlay is up.
    static LAUNCHING: RefCell<Option<String>> = const { RefCell::new(None) };
}

const SHELF_SIZE: usize = 24;

pub fn source_label(source: &str) -> String {
    match source {
        "steam" => "Steam".into(),
        "epic" => "Epic Games".into(),
        "gog" => "GOG".into(),
        "xbox" => "Xbox".into(),
        "ubisoft" => "Ubisoft".into(),
        _ => tr("Custom"),
    }
}

/// Hours with one decimal for the big number, or minutes under an hour.
fn played(minutes: u64) -> (String, String) {
    if minutes == 0 {
        return ("0".into(), tr("h"));
    }
    if minutes < 60 {
        return (minutes.to_string(), tr("min"));
    }
    let h = (minutes as f64 / 6.0).round() / 10.0;
    let text = if h >= 100.0 { format!("{}", h.round() as u64) } else { format!("{h:.1}") };
    let text = if crate::i18n::language() == "en" { text } else { text.replace('.', ",") };
    (text, tr("h"))
}

fn last_played(ts: Option<i64>, played: bool) -> String {
    let Some(ts) = ts else { return if played { "—".into() } else { tr("Never") } };
    let days = (now() - ts) / 86_400;
    let today = now() / 86_400 == ts / 86_400;
    if today {
        tr("Today")
    } else if days <= 1 {
        tr("Yesterday")
    } else if days < 7 {
        t("{days} days ago", &[("days", days.to_string())])
    } else if days < 30 {
        let w = days / 7;
        if w == 1 { tr("Last week") } else { t("{w} weeks ago", &[("w", w.to_string())]) }
    } else {
        // Day.month.year: the same in every language Mantel has but English, close enough there.
        let (y, m, d) = civil(ts / 86_400);
        format!("{d}.{m:02}.{y}")
    }
}

/// Days since 1970 to a calendar date (Howard Hinnant's algorithm).
fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn size(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes.filter(|b| *b > 0) else { return "—".into() };
    let gb = bytes as f64 / 1024f64.powi(3);
    let n = if gb >= 100.0 { format!("{gb:.0}") } else { format!("{gb:.1}") };
    let n = if crate::i18n::language() == "en" { n } else { n.replace('.', ",") };
    if gb >= 1.0 {
        t("{n} GB", &[("n", n)])
    } else {
        t("{n} MB", &[("n", (bytes / 1024 / 1024).to_string())])
    }
}

fn hue(name: &str) -> f32 {
    let h = name.bytes().fold(2_166_136_261u32, |h, b| (h ^ b as u32).wrapping_mul(16_777_619));
    (h % 360) as f32
}

fn running_ids(app: &AppHandle) -> Vec<String> {
    app.state::<AppState>().running.lock().unwrap().iter().filter(|r| r.detected).map(|r| r.id.clone()).collect()
}

fn card(g: &Game, running: &[String]) -> Card {
    let cover = g.art.cover.first().and_then(|src| images::ready(src, Fit::Cover)).flatten();
    Card {
        id: g.id.clone().into(),
        name: g.name.clone().into(),
        has_cover: cover.is_some(),
        cover: cover.map(Image::from_rgba8).unwrap_or_default(),
        hue: hue(&g.name),
        source: g.source.clone().into(),
        favorite: g.favorite,
        running: running.contains(&g.id),
    }
}

/// What Mantel just ran, then recently played, favorites, and the newest installs.
fn shelf(games: &[Game]) -> Vec<&Game> {
    let visible: Vec<&Game> = games.iter().filter(|g| !g.hidden && g.installed && !g.is_app()).collect();
    let mut recent: Vec<&Game> = visible.iter().copied().filter(|g| g.last_played.is_some()).collect();
    recent.sort_by_key(|g| std::cmp::Reverse(g.last_played));
    let favs = visible.iter().copied().filter(|g| g.favorite);
    let mut fresh = visible.clone();
    fresh.sort_by_key(|g| std::cmp::Reverse(g.added_at));
    let mut out: Vec<&Game> = Vec::new();
    for g in recent.into_iter().chain(favs).chain(fresh) {
        if !out.iter().any(|o| o.id == g.id) {
            out.push(g);
        }
        if out.len() == SHELF_SIZE {
            break;
        }
    }
    out
}

fn library(games: &[Game]) -> Vec<&Game> {
    let mut list: Vec<&Game> = games.iter().filter(|g| !g.hidden && g.installed && !g.is_app()).collect();
    list.sort_by_key(|g| g.name.to_lowercase());
    list
}

/// Puts a freshly loaded cover into every card of that game, on the UI thread.
fn cover_arrived(weak: slint::Weak<AppWindow>, id: String, px: images::Pixels) {
    let _ = slint::invoke_from_event_loop(move || {
        let Some(w) = weak.upgrade() else { return };
        let image = Image::from_rgba8(px);
        for model in [w.get_shelf(), w.get_library()] {
            for i in 0..model.row_count() {
                if let Some(mut c) = model.row_data(i) {
                    if c.id == id.as_str() {
                        c.cover = image.clone();
                        c.has_cover = true;
                        model.set_row_data(i, c);
                    }
                }
            }
        }
        if w.get_launch_card().id == id.as_str() {
            let mut c = w.get_launch_card();
            c.cover = image;
            c.has_cover = true;
            w.set_launch_card(c);
        }
    });
}

/// Shows `games`, keeping focus on the same game where it's still there.
pub fn show_games(app: &AppHandle, w: &AppWindow, games: Vec<Game>) {
    let running = running_ids(app);
    let keep_shelf = w.get_shelf().row_data(w.get_shelf_index() as usize).map(|c| c.id.to_string());
    let keep_lib = w.get_library().row_data(w.get_library_index() as usize).map(|c| c.id.to_string());
    let shelf_games = shelf(&games);
    let lib_games = library(&games);
    let shelf_cards: Vec<Card> = shelf_games.iter().map(|g| card(g, &running)).collect();
    let lib_cards: Vec<Card> = lib_games.iter().map(|g| card(g, &running)).collect();
    for g in shelf_games.iter().chain(lib_games.iter()) {
        if !g.art.cover.is_empty() && images::ready(&g.art.cover[0], Fit::Cover).is_none() {
            let (weak, id) = (w.as_weak(), g.id.clone());
            images::load(g.art.cover.clone(), Fit::Cover, move |_, px| cover_arrived(weak, id, px));
        }
    }
    let si = keep_shelf.and_then(|id| shelf_cards.iter().position(|c| c.id == id.as_str())).unwrap_or(0);
    let li = keep_lib.and_then(|id| lib_cards.iter().position(|c| c.id == id.as_str())).unwrap_or(0);
    w.set_library_total(lib_cards.len() as i32);
    w.set_shelf(ModelRc::new(Rc::new(VecModel::from(shelf_cards))));
    w.set_library(ModelRc::new(Rc::new(VecModel::from(lib_cards))));
    w.set_shelf_index(si as i32);
    w.set_library_index(li as i32);
    GAMES.with(|g| *g.borrow_mut() = games);
    let id = if w.get_tab() == 0 { w.get_shelf().row_data(si) } else { w.get_library().row_data(li) }.map(|c| c.id.to_string());
    if let Some(id) = id {
        focus(app, w, &id);
    }
}

/// Fills the Home header for game `id`, and starts loading its background and logo.
pub fn focus(app: &AppHandle, w: &AppWindow, id: &str) {
    let Some(g) = GAMES.with(|gs| gs.borrow().iter().find(|g| g.id == id).cloned()) else { return };
    let running = running_ids(app).contains(&g.id);
    let (value, unit) = played(g.playtime_minutes);
    let ach = g.achievements.as_ref().filter(|a| a.total > 0);
    let prev = w.get_focus_info();
    let same = prev.id == id;
    let hero = g.art.hero.iter().find_map(|s| images::ready(s, Fit::Hero).flatten());
    let logo = g.art.logo.iter().find_map(|s| images::ready(s, Fit::Logo).flatten());
    w.set_focus_info(Focused {
        id: g.id.clone().into(),
        name: g.name.clone().into(),
        source: g.source.clone().into(),
        source_label: source_label(&g.source).to_uppercase().into(),
        has_hero: hero.is_some() || (same && prev.has_hero),
        hero: hero.map(Image::from_rgba8).unwrap_or(if same { prev.hero.clone() } else { Image::default() }),
        has_logo: logo.is_some() || (same && prev.has_logo),
        logo: logo.map(Image::from_rgba8).unwrap_or(if same { prev.logo.clone() } else { Image::default() }),
        played_value: value.into(),
        played_unit: unit.into(),
        last: last_played(g.last_played, g.playtime_minutes > 0).into(),
        ach_value: ach.map(|a| a.unlocked.to_string()).unwrap_or_default().into(),
        ach_total: ach.map(|a| a.total.to_string()).unwrap_or_default().into(),
        ach_ratio: ach.map(|a| a.unlocked as f32 / a.total as f32).unwrap_or(0.0),
        has_ach: ach.is_some(),
        size: size(g.size_bytes).into(),
        favorite: g.favorite,
        running,
    });
    // Backgrounds are big: only the focused game's stays decoded.
    let weak = w.as_weak();
    if !g.art.hero.is_empty() {
        let (weak, gid) = (weak.clone(), g.id.clone());
        images::load(g.art.hero.clone(), Fit::Hero, move |_, px| {
            let _ = slint::invoke_from_event_loop(move || {
                let Some(w) = weak.upgrade() else { return };
                let mut f = w.get_focus_info();
                if f.id == gid.as_str() {
                    f.hero = Image::from_rgba8(px);
                    f.has_hero = true;
                    w.set_focus_info(f);
                }
            });
        });
    }
    if !g.art.logo.is_empty() {
        let gid = g.id.clone();
        images::load(g.art.logo.clone(), Fit::Logo, move |_, px| {
            let _ = slint::invoke_from_event_loop(move || {
                let Some(w) = weak.upgrade() else { return };
                let mut f = w.get_focus_info();
                if f.id == gid.as_str() {
                    f.logo = Image::from_rgba8(px);
                    f.has_logo = true;
                    w.set_focus_info(f);
                }
            });
        });
    }
}

fn toast(w: &AppWindow, text: String) {
    w.set_toast(text.clone().into());
    let weak = w.as_weak();
    slint::Timer::single_shot(Duration::from_millis(3200), move || {
        if let Some(w) = weak.upgrade() {
            if w.get_toast() == text.as_str() {
                w.set_toast(SharedString::new());
            }
        }
    });
}

/// Reloads the library from the backend (which is cheap: it's the cached list) and shows it.
fn refresh(app: &AppHandle, weak: slint::Weak<AppWindow>) {
    let app2 = app.clone();
    crate::runtime().spawn(async move {
        let games = crate::core::snapshot(app2.state::<AppState>());
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(w) = weak.upgrade() {
                show_games(&app2, &w, games);
            }
        });
    });
}

/// A full scan of every store, then art and sizes for what it found.
fn rescan(app: &AppHandle, weak: slint::Weak<AppWindow>, quiet: bool) {
    if let Some(w) = weak.upgrade() {
        if w.get_scanning() {
            return;
        }
        w.set_scanning(true);
    }
    let app = app.clone();
    crate::runtime().spawn(async move {
        let before = crate::core::snapshot(app.state::<AppState>()).len();
        let scanned = crate::core::get_library(app.clone(), true).await;
        let show = |games: Vec<Game>, done: bool, note: Option<String>| {
            let (app, weak) = (app.clone(), weak.clone());
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak.upgrade() {
                    show_games(&app, &w, games);
                    if done {
                        w.set_scanning(false);
                    }
                    if let Some(n) = note {
                        toast(&w, n);
                    }
                }
            });
        };
        match scanned {
            Ok(games) => {
                let found = games.len().saturating_sub(before);
                let note = (!quiet && found > 0).then(|| if found == 1 { tr("1 new game found") } else { t("{n} new games found", &[("n", found.to_string())]) });
                show(games, false, note);
            }
            Err(e) => show(crate::core::snapshot(app.state::<AppState>()), false, Some(t("Scan failed: {error}", &[("error", e)]))),
        }
        let state = app.state::<AppState>();
        if crate::core::resolve_art(state).await.unwrap_or(false) {
            show(crate::core::snapshot(state), false, None);
        }
        if crate::core::measure_sizes(state).await.unwrap_or(false) {
            show(crate::core::snapshot(state), false, None);
        }
        let _ = crate::core::resolve_steam_facts(state, crate::i18n::steam_language().into()).await;
        show(crate::core::snapshot(state), true, None);
    });
}

fn update_clock(w: &AppWindow) {
    let secs = now() + local_offset();
    let (h, m) = ((secs / 3600) % 24, (secs / 60) % 60);
    w.set_clock(format!("{h:02}:{m:02}").into());
    let p = crate::system::power();
    w.set_battery(if p.has_battery { p.percent.map(|n| format!("{n}%")).unwrap_or_default() } else { String::new() }.into());
}

/// Seconds east of UTC, from Windows' time zone (daylight saving included).
fn local_offset() -> i64 {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
        let mut tz: TIME_ZONE_INFORMATION = std::mem::zeroed();
        let r = GetTimeZoneInformation(&mut tz);
        let bias = tz.Bias + if r == 2 { tz.DaylightBias } else { tz.StandardBias };
        return -(bias as i64) * 60;
    }
    #[allow(unreachable_code)]
    0
}

fn store_of(source: &str) -> String {
    match source {
        "steam" => "Steam".into(),
        "epic" => "Epic Games".into(),
        "gog" => "GOG Galaxy".into(),
        "xbox" => "Xbox".into(),
        "ubisoft" => "Ubisoft Connect".into(),
        _ => String::new(),
    }
}

pub fn run(app: AppHandle, demo: Option<Vec<Game>>) -> Result<(), slint::PlatformError> {
    let w = AppWindow::new()?;
    w.global::<Tr>().on_t(|text, _rev| tr(&text).into());
    w.global::<Tr>().on_upper(|text| text.to_uppercase().into());

    let weak = w.as_weak();
    // Events from the tracker and the launcher, on their threads: hop to the UI thread.
    {
        let weak = weak.clone();
        let app2 = app.clone();
        app.listen(move |event, value| {
            let (weak, app, event) = (weak.clone(), app2.clone(), event.to_string());
            let _ = slint::invoke_from_event_loop(move || {
                let Some(w) = weak.upgrade() else { return };
                on_event(&app, &w, &event, value);
            });
        });
    }

    w.on_focus_changed({
        let (app, weak) = (app.clone(), weak.clone());
        move |id| {
            if let Some(w) = weak.upgrade() {
                focus(&app, &w, &id);
            }
        }
    });
    w.on_launch({
        let (app, weak) = (app.clone(), weak.clone());
        move |id| {
            let Some(w) = weak.upgrade() else { return };
            let Some(g) = GAMES.with(|gs| gs.borrow().iter().find(|g| g.id == id.as_str()).cloned()) else { return };
            if running_ids(&app).contains(&g.id) {
                let (app, id) = (app.clone(), g.id.clone());
                crate::runtime().spawn(async move {
                    crate::core::switch_to_game(app, id).await;
                });
                return;
            }
            w.set_launch_card(card(&g, &[]));
            w.set_launch_store(t("via {store}", &[("store", store_of(&g.source))]).into());
            w.set_launch_phase("starting".into());
            LAUNCHING.with(|l| *l.borrow_mut() = Some(g.id.clone()));
            if let Err(e) = crate::core::launch_game(app.clone(), app.state::<AppState>(), g.id.clone()) {
                w.set_launch_phase("failed".into());
                w.set_launch_store(tr(&e).into());
            }
        }
    });
    w.on_dismiss_launch({
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                w.set_launch_phase(SharedString::new());
            }
            LAUNCHING.with(|l| *l.borrow_mut() = None);
        }
    });
    w.on_toggle_favorite({
        let (app, weak) = (app.clone(), weak.clone());
        move |id| {
            let Some(w) = weak.upgrade() else { return };
            let Some(g) = GAMES.with(|gs| gs.borrow().iter().find(|g| g.id == id.as_str()).cloned()) else { return };
            if crate::core::set_flag(app.state::<AppState>(), g.id.clone(), "favorite".into(), !g.favorite).is_ok() {
                toast(&w, if g.favorite { t("Removed {name} from favorites", &[("name", g.name.clone())]) } else { t("Added {name} to favorites", &[("name", g.name.clone())]) });
                refresh(&app, weak.clone());
            }
        }
    });
    w.on_rescan({
        let (app, weak) = (app.clone(), weak.clone());
        move || rescan(&app, weak.clone(), false)
    });
    w.on_toggle_fullscreen({
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                let full = w.window().is_fullscreen();
                w.window().set_fullscreen(!full);
            }
        }
    });
    w.on_quit(|| {
        let _ = slint::quit_event_loop();
    });

    // The pad drives Mantel only while Mantel's window is the one in front.
    {
        let weak = weak.clone();
        let app2 = app.clone();
        crate::pad::start(
            move |a| {
                let weak = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_act(a.into());
                    }
                });
            },
            move || in_front(&app2),
        );
    }

    let clock = slint::Timer::default();
    update_clock(&w);
    clock.start(slint::TimerMode::Repeated, Duration::from_secs(10), {
        let weak = weak.clone();
        move || {
            if let Some(w) = weak.upgrade() {
                update_clock(&w);
            }
        }
    });

    match demo {
        Some(games) => show_games(&app, &w, games),
        None => {
            show_games(&app, &w, crate::core::snapshot(app.state::<AppState>()));
            rescan(&app, weak.clone(), true);
        }
    }

    // The HWND exists once the window is shown; the tracker and the launcher need it.
    w.show()?;
    remember_hwnd(&app, &w);
    if std::env::var_os("MANTEL_WINDOWED").is_none() {
        w.window().set_fullscreen(true);
    }
    slint::run_event_loop()?;
    w.hide()
}

#[allow(unused_variables)]
fn remember_hwnd(app: &AppHandle, w: &AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = w.window().window_handle().window_handle() {
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            app.set_hwnd(h.hwnd.get());
        }
    }
}

/// Whether Mantel's own window is the one in front, so the pad belongs to it.
fn in_front(app: &AppHandle) -> bool {
    #[cfg(windows)]
    {
        let fg = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() } as isize;
        return fg != 0 && fg == app.hwnd();
    }
    #[allow(unreachable_code)]
    {
        let _ = app;
        true
    }
}

fn on_event(app: &AppHandle, w: &AppWindow, event: &str, value: serde_json::Value) {
    match event {
        "game-started" => {
            let id = value.as_str().unwrap_or_default().to_string();
            if LAUNCHING.with(|l| l.borrow().as_deref() == Some(id.as_str())) {
                w.set_launch_phase("playing".into());
                let weak = w.as_weak();
                slint::Timer::single_shot(Duration::from_secs(4), move || {
                    if let Some(w) = weak.upgrade() {
                        if w.get_launch_phase() == "playing" {
                            w.set_launch_phase(SharedString::new());
                        }
                    }
                    LAUNCHING.with(|l| *l.borrow_mut() = None);
                });
            }
            refresh(app, w.as_weak());
        }
        "game-stopped" => {
            let id = value.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let seconds = value.get("seconds").and_then(|v| v.as_u64()).unwrap_or(0);
            let name = GAMES.with(|gs| gs.borrow().iter().find(|g| g.id == id).map(|g| (g.name.clone(), g.is_app())));
            if let Some((name, false)) = name {
                if seconds > 0 {
                    let min = (seconds + 30) / 60;
                    toast(w, if min > 0 { t("{name}: +{min} min played", &[("name", name), ("min", min.to_string())]) } else { t("{name} closed", &[("name", name)]) });
                }
            }
            refresh(app, w.as_weak());
        }
        "game-launch-timeout" => {
            let id = value.as_str().unwrap_or_default().to_string();
            if LAUNCHING.with(|l| l.borrow().as_deref() == Some(id.as_str())) {
                w.set_launch_phase("handed".into());
            }
        }
        _ => {}
    }
}
