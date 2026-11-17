//! Sample games for working on the interface without a real library (MANTEL_DEMO=1).

use crate::model::{now, Achievements, Game, Launch};

pub fn games() -> Vec<Game> {
    let names = [
        ("Hades II", "steam", 2717, Some(1), Some((11, 33)), Some(57)),
        ("Cyberpunk 2077", "steam", 6240, Some(0), Some((14, 44)), Some(70)),
        ("Clair Obscur: Expedition 33", "steam", 1890, Some(3), Some((20, 51)), Some(55)),
        ("Baldur's Gate 3", "steam", 9900, Some(12), Some((31, 54)), Some(150)),
        ("Forza Horizon 5", "xbox", 1300, Some(20), None, Some(110)),
        ("Black Myth: Wukong", "steam", 0, None, None, Some(130)),
        ("Stardew Valley", "steam", 7010, Some(40), Some((40, 49)), Some(1)),
        ("Control", "epic", 600, Some(90), None, Some(42)),
        ("Indiana Jones and the Great Circle", "xbox", 450, Some(200), None, Some(120)),
        ("Elden Ring", "steam", 15000, Some(400), Some((42, 42)), Some(60)),
        ("The Witcher 3", "gog", 22000, Some(600), None, Some(50)),
        ("Assassin's Creed Shadows", "ubisoft", 0, None, None, Some(115)),
    ];
    let hero = std::env::var("MANTEL_DEMO_HERO").ok();
    names
        .iter()
        .enumerate()
        .map(|(i, (name, source, minutes, days, ach, gb))| {
            let mut g = Game::new(source, format!("demo{i}"), name.to_string(), Launch::Uri { uri: String::new() });
            g.playtime_minutes = *minutes;
            g.last_played = days.map(|d| now() - d * 86_400 - 3_600);
            g.added_at = Some(now() - i as i64 * 1000);
            g.size_bytes = gb.map(|n| n * 1024 * 1024 * 1024);
            g.favorite = i == 0 || i == 3;
            g.achievements = ach.map(|(u, t)| Achievements { unlocked: u, total: t, percent: (100 * u / t) as u8, gamerscore: None, gamerscore_total: None });
            if i == 0 {
                g.art.hero.extend(hero.clone());
            }
            g
        })
        .collect()
}
