import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { SourceIcon } from "../components/Glyph";
import { useHints } from "../components/Hints";
import { AddGame } from "../components/AddGame";
import { KeyFlow } from "../components/KeyFlow";
import { useScrollFocus } from "../hooks";
import { Layer, useInput } from "../input";
import { errorText, LANGUAGES, t } from "../i18n";
import { loadCatalog, openSetup, refreshAchievements, rescan, saveSettings, setHidden, toast, useStore } from "../store";
import { sound } from "../sound";
import type { Settings as S, Source, SteamProfile } from "../types";

type Row =
  | { kind: "group"; label: string }
  | { kind: "toggle"; id: string; label: string; sub?: string; value: boolean; icon?: Source; set: (v: boolean) => void }
  | { kind: "button"; id: string; label: string; sub?: string; value?: string; icon?: Icon; soon?: boolean; run: () => void };

type Icon = Source | "art";
type Flow = "steamid" | "sgdb" | "add" | null;
type Category = "library" | "interface" | "system";

const CATEGORIES: [Category, string, React.ReactNode][] = [
  ["library", "Library", <><rect x="3.5" y="3.5" width="7" height="7" rx="1.6" /><rect x="13.5" y="3.5" width="7" height="7" rx="1.6" /><rect x="3.5" y="13.5" width="7" height="7" rx="1.6" /><rect x="13.5" y="13.5" width="7" height="7" rx="1.6" /></>],
  ["interface", "Interface", <><rect x="3" y="4.5" width="18" height="12.5" rx="2.2" /><path d="M9 20.5h6M12 17v3.5" /></>],
  ["system", "System", <><rect x="6" y="6" width="12" height="12" rx="2.2" /><path d="M9.5 2.8v3.2M14.5 2.8v3.2M9.5 18v3.2M14.5 18v3.2M2.8 9.5H6M2.8 14.5H6M18 9.5h3.2M18 14.5h3.2" /></>],
];

function RowIcon({ icon }: { icon: Icon }) {
  if (icon === "art") {
    return (
      <svg className="source-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round">
        <rect x="3.5" y="3.5" width="17" height="17" rx="3" />
        <circle cx="9" cy="9" r="1.8" />
        <path d="m20 15-4.5-4.5L6 20" />
      </svg>
    );
  }
  return <SourceIcon source={icon} />;
}

const SOURCES: [Exclude<Source, "custom">, string][] = [
  ["steam", "Steam"],
  ["epic", "Epic Games"],
  ["gog", "GOG Galaxy"],
  ["xbox", "Xbox / Game Pass"],
  ["ubisoft", "Ubisoft Connect"],
];

/** Where the player was: a new language remounts the interface, and they should stay put. */
const place = { category: "library" as Category, zone: "categories" as "categories" | "rows", index: 0 };

export function Settings({ active }: { active: boolean }) {
  const settings = useStore((s) => s.settings);
  const games = useStore((s) => s.games);
  const scanning = useStore((s) => s.scanning);
  const version = useStore((s) => s.version);
  const [category, setCategory] = useState<Category>(place.category);
  const [zone, setZone] = useState<"categories" | "rows">(place.zone);
  const [index, setIndex] = useState(place.index);
  useEffect(() => void Object.assign(place, { category, zone, index }), [category, zone, index]);
  const [autostart, setAutostart] = useState(false);
  const [flow, setFlow] = useState<Flow>(null);
  const [profile, setProfile] = useState<SteamProfile | null>(null);
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api.getAutostart().then(setAutostart).catch(() => {});
    api.steamProfile().then(setProfile).catch(() => {});
  }, []);

  useEffect(() => {
    list.current?.scrollTo({ top: 0 });
  }, [category]);
  // Scrolls only when the focused row nears an edge: the lists are short, rows stay put.
  useScrollFocus(list, `${category}${zone}${index}`);

  if (!settings) return null;
  const patch = (p: Partial<S>) => saveSettings(p);
  const hidden = games.filter((g) => g.hidden);

  const library: Row[] = [
    {
      kind: "button",
      id: "rescan",
      label: scanning ? t("Scanning…") : t("Refresh library"),
      sub: t("Look for new installs across all launchers"),
      run: () => rescan(),
    },
    {
      kind: "button",
      id: "add",
      label: t("Add a game manually"),
      sub: t("Any .exe, shortcut or .url: games outside a store, emulators, Battle.net, EA app"),
      run: () => setFlow("add"),
    },
    { kind: "group", label: t("Stores on this PC") },
    ...SOURCES.map(
      ([id, label]): Row => ({
        kind: "toggle",
        id: `src-${id}`,
        label,
        sub: t("Show games installed with {store}", { store: label }),
        icon: id,
        value: settings.sources[id],
        set: (v) => {
          patch({ sources: { ...settings.sources, [id]: v } }).then(() => rescan(true));
        },
      }),
    ),
    { kind: "group", label: t("Achievements") },
    {
      kind: "button",
      id: "acc-steam-profile",
      icon: "steam",
      label: t("Steam achievements"),
      sub: profile
        ? profile.public
          ? t("From the profile of {name}, with when you unlocked each, no sign-in needed", { name: profile.name || profile.steamId })
          : t("The profile of {name} is private: make its game details public on Steam to see achievements", { name: profile.name || profile.steamId })
        : t("From your public Steam profile, with when you unlocked each, no sign-in needed"),
      value: profile ? (profile.detected ? t("From this PC") : t("Your profile")) : t("Add"),
      run: () => setFlow("steamid"),
    },
    { kind: "group", label: t("Artwork") },
    {
      kind: "button",
      id: "sgdb",
      icon: "art",
      label: "SteamGridDB",
      sub: t("Art for games Steam doesn't know, and more covers, backgrounds and logos to pick from on any game's page"),
      value: settings.steamgriddbKey ? t("Connected") : t("Connect"),
      run: () => setFlow("sgdb"),
    },
    ...(hidden.length
      ? [
          { kind: "group", label: t("Hidden games · {n}", { n: hidden.length }) } as Row,
          ...hidden.map(
            (g): Row => ({ kind: "button", id: `hidden-${g.id}`, label: g.name, value: t("Unhide"), run: () => setHidden(g, false) }),
          ),
        ]
      : []),
  ];

  const languageAt = Math.max(0, LANGUAGES.findIndex((l) => l.id === settings.language));
  const interfaceRows: Row[] = [
    {
      kind: "button",
      id: "language",
      label: t("Language"),
      sub: t("System follows the language of Windows"),
      value: t(LANGUAGES[languageAt].label),
      // Cycles, like a console's option row.
      run: () => patch({ language: LANGUAGES[(languageAt + 1) % LANGUAGES.length].id }),
    },
    {
      kind: "toggle",
      id: "trailers",
      label: t("Trailers"),
      sub: t("Play a muted trailer behind the game you stop on"),
      value: settings.trailers,
      set: (v) => patch({ trailers: v }),
    },
    {
      kind: "toggle",
      id: "gamepass",
      label: t("Game Pass catalog"),
      sub: t("Browse the PC Game Pass catalog and install from it"),
      value: settings.gamePass,
      set: (v) => patch({ gamePass: v }).then(loadCatalog),
    },
    {
      kind: "toggle",
      id: "sounds",
      label: t("Interface sounds"),
      value: settings.sounds,
      set: (v) => patch({ sounds: v }),
    },
  ];

  const system: Row[] = [
    {
      kind: "toggle",
      id: "autostart",
      label: t("Start with Windows"),
      sub: t("Boot straight into Mantel, like a console"),
      value: autostart,
      set: (v) => {
        setAutostart(v);
        api.setAutostart(v).catch((e) => toast(errorText(e), "warn"));
      },
    },
    {
      kind: "toggle",
      id: "minimize",
      label: t("Minimize while playing"),
      sub: t("Mantel steps aside during a game and comes back when it closes"),
      value: settings.minimizeOnLaunch,
      set: (v) => patch({ minimizeOnLaunch: v }),
    },
    {
      kind: "toggle",
      id: "tray",
      label: t("Icon in the notification area"),
      sub: t("Minimizing, closing or Esc pressed twice hides Mantel next to the clock; the icon brings it back"),
      value: settings.tray,
      set: (v) => patch({ tray: v }),
    },
    {
      kind: "toggle",
      id: "bigpicture",
      label: t("Steam games in Big Picture"),
      sub: t("Steam opens in Big Picture before the game and closes it after"),
      value: settings.bigPicture,
      set: (v) => patch({ bigPicture: v }),
    },
    {
      kind: "button",
      id: "hdr",
      label: t("HDR for games"),
      sub: t("Turn HDR on for a game on its page; it goes back off when the game ends"),
      value: settings.hdrMethod === "nvidia" ? "NVIDIA" : "Windows",
      run: () => patch({ hdrMethod: settings.hdrMethod === "nvidia" ? "windows" : "nvidia" }),
    },
    {
      kind: "button",
      id: "setup",
      label: t("Run setup again"),
      sub: t("Language, games and how the handheld starts, step by step"),
      run: openSetup,
    },
  ];

  const rows = { library, interface: interfaceRows, system }[category];
  const focusable = rows.map((r, n) => (r.kind === "group" ? -1 : n)).filter((n) => n >= 0);
  const i = focusable.includes(index) ? index : focusable[0];
  const pos = focusable.indexOf(i);
  const catPos = CATEGORIES.findIndex(([id]) => id === category);

  const pick = (c: Category) => {
    setCategory(c);
    setIndex(0);
  };
  const activate = (row: Row) => {
    if (row.kind === "toggle") {
      row.set(!row.value);
      sound.toggle();
    } else if (row.kind === "button") {
      sound.select();
      row.run();
    }
  };

  return (
    <SettingsInput
      active={active && !flow}
      zone={zone}
      onMove={(d) => {
        if (zone === "categories") {
          const next = CATEGORIES[catPos + d];
          if (!next) return "edge";
          pick(next[0]);
          return;
        }
        const next = focusable[pos + d];
        if (next === undefined) return "edge";
        setIndex(next);
      }}
      onEnter={() => setZone("rows")}
      onLeave={() => setZone("categories")}
      onActivate={() => activate(rows[i])}
    >
      <main className={`settings zone-${zone}`}>
        <h1 className="page-title">{t("Settings")}</h1>
        <div className="settings-body">
          <nav className="settings-cats">
            {CATEGORIES.map(([id, label, icon]) => (
              <button
                key={id}
                className={`settings-cat ${id === category ? "is-current" : ""} ${id === category && zone === "categories" ? "is-focused" : ""}`}
                onClick={() => {
                  pick(id);
                  setZone("categories");
                }}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  {icon}
                </svg>
                {t(label)}
              </button>
            ))}
          </nav>
          <div className="settings-list" ref={list}>
            {rows.map((row, n) =>
              row.kind === "group" ? (
                <div className="settings-group" key={`g${n}`}>{row.label}</div>
              ) : (
                <div
                  key={row.id}
                  className={`setting ${zone === "rows" && n === i ? "is-focused" : ""} ${row.kind === "button" && row.soon ? "is-soon" : ""}`}
                  onClick={() => {
                    setZone("rows");
                    setIndex(n);
                    activate(row);
                  }}
                >
                  {row.kind === "toggle" && row.icon && <SourceIcon source={row.icon} />}
                  {row.kind === "button" && row.icon && <RowIcon icon={row.icon} />}
                  <div className="setting-text">
                    <div className="setting-label">{row.label}</div>
                    {row.sub && <div className="setting-sub">{row.sub}</div>}
                  </div>
                  {row.kind === "toggle" ? (
                    <span className={`switch ${row.value ? "on" : ""}`}><span /></span>
                  ) : (
                    row.value && <span className="setting-value">{row.value}</span>
                  )}
                </div>
              ),
            )}
            {category === "system" && <div className="settings-foot">Mantel {version} · playmantel.com</div>}
          </div>
        </div>
        {flow === "add" && <AddGame onClose={() => setFlow(null)} />}
        {flow === "steamid" && (
          <KeyFlow
            kind="profile"
            service="Steam"
            title={t("Achievements from your profile")}
            intro={t("Mantel reads achievements from your public Steam profile. Without a link it uses the account Steam signed in with on this PC.")}
            howTo={[t("Open your profile on Steam and copy its address"), t("Its game details must be public: Edit Profile → Privacy Settings"), t("Come back here and paste it")]}
            url="https://steamcommunity.com/my/profile"
            site="steamcommunity.com"
            connected={!!profile && !profile.detected}
            status={profile ? `${profile.name || profile.steamId} · ${profile.public ? t("public") : t("private")}` : undefined}
            onSubmit={async (link) => {
              const p = await api.setSteamId(link);
              setProfile(p);
              await patch({ steamId: p?.detected ? "" : p?.steamId ?? "" });
              toast(t("Achievements now come from {name}", { name: p?.name || p?.steamId || "Steam" }), "good");
              void refreshAchievements();
            }}
            disconnectLabel={t("Use the account on this PC")}
            onDisconnect={async () => {
              const p = await api.setSteamId("");
              setProfile(p);
              await patch({ steamId: "" });
              void refreshAchievements();
            }}
            onClose={() => setFlow(null)}
          />
        )}
        {flow === "sgdb" && (
          <KeyFlow
            service="SteamGridDB"
            title={t("Better artwork")}
            intro={t("A free community library of covers, backgrounds and logos. Mantel uses it for games Steam doesn't have.")}
            howTo={[t("Open SteamGridDB and log in with Steam"), t("Preferences → API → Generate API key"), t("Copy it, come back here and paste it")]}
            url="https://www.steamgriddb.com/profile/preferences/api"
            site="steamgriddb.com"
            connected={!!settings.steamgriddbKey}
            status={settings.steamgriddbKey ? t("Connected · key ••••{last4}", { last4: settings.steamgriddbKey.slice(-4) }) : undefined}
            onSubmit={async (key) => {
              if (!/^[0-9a-f]{32}$/i.test(key)) throw t("That doesn't look like a SteamGridDB key (32 letters and digits).");
              await patch({ steamgriddbKey: key });
              rescan(true);
            }}
            onDisconnect={async () => {
              await patch({ steamgriddbKey: "" });
            }}
            onClose={() => setFlow(null)}
          />
        )}
      </main>
    </SettingsInput>
  );
}

/** Separate component so hooks run unconditionally regardless of the early return above. */
function SettingsInput({
  active,
  zone,
  onMove,
  onEnter,
  onLeave,
  onActivate,
  children,
}: {
  active: boolean;
  zone: "categories" | "rows";
  onMove: (d: number) => "edge" | void;
  onEnter: () => void;
  onLeave: () => void;
  onActivate: () => void;
  children: React.ReactNode;
}) {
  useInput(
    (a) => {
      if (a === "up") return onMove(-1);
      if (a === "down") return onMove(1);
      if (zone === "categories") {
        if (a === "a" || a === "right") return onEnter();
        if (a === "left") return "edge";
        return false;
      }
      if (a === "a") {
        onActivate();
        return "silent";
      }
      if (a === "left" || a === "b") return onLeave();
      if (a === "right") return "edge";
      return false;
    },
    Layer.view,
    active,
  );
  useHints(
    zone === "categories"
      ? [["a", "Open"], ["dpad", "Move"], ["menu", "Control center"]]
      : [["a", "Select"], ["b", "Back"], ["menu", "Control center"]],
    Layer.view,
    active,
  );
  return <>{children}</>;
}
