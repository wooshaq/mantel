import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { Glyph } from "../components/Glyph";
import { useHints } from "../components/Hints";
import { dayKey, minutesOf } from "../components/PlayHistory";
import { locale, t } from "../i18n";
import { Layer, useInput } from "../input";
import { gameById, useStore } from "../store";
import type { Session, UnlockTimes } from "../types";
import { playtime } from "../util";

/** Colours for the month's most played games, in order; the rest share one grey. */
const COLORS = ["#5bb8d0", "#ef8a4b", "#59c98c", "#e7c24a", "#b18cf0"];
const OTHER = "rgba(255, 255, 255, 0.28)";

interface DayStat {
  seconds: number;
  games: Map<string, number>;
}

const midnight = (ms: number) => {
  const d = new Date(ms);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
};
const keyOf = (ms: number) => dayKey(ms / 1000);
/** The local day a key names: "2026-11-15" read as a date would be midnight UTC. */
const fromKey = (key: string) => {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, m - 1, d);
};
/** Adds days on the calendar, so a change to or from summer time doesn't skip a day. */
const addDays = (ms: number, n: number) => {
  const d = new Date(ms);
  d.setDate(d.getDate() + n);
  return d.getTime();
};

/** Playtime per local day and game, sessions split at midnight. */
function byDay(sessions: Session[]): Map<string, DayStat> {
  const days = new Map<string, DayStat>();
  for (const s of sessions) {
    let from = s.start * 1000;
    const end = s.end * 1000;
    while (from < end) {
      const next = Math.min(end, addDays(midnight(from), 1));
      const key = keyOf(from);
      const day = days.get(key) ?? { seconds: 0, games: new Map() };
      const secs = (next - from) / 1000;
      day.seconds += secs;
      day.games.set(s.game, (day.games.get(s.game) ?? 0) + secs);
      days.set(key, day);
      from = next;
    }
  }
  return days;
}

function unlocksByDay(list: UnlockTimes[]): Map<string, number> {
  const days = new Map<string, number>();
  for (const g of list) for (const at of g.times) if (at > 0) days.set(dayKey(at), (days.get(dayKey(at)) ?? 0) + 1);
  return days;
}

/** How bright a day is: by how long it was played. */
function level(seconds: number): number {
  if (seconds <= 0) return 0;
  if (seconds < 30 * 60) return 1;
  if (seconds < 90 * 60) return 2;
  if (seconds < 3 * 3600) return 3;
  return 4;
}

/** Monday on or before a date. */
const weekStart = (ms: number) => {
  const d = new Date(midnight(ms));
  return addDays(d.getTime(), -((d.getDay() + 6) % 7));
};

/**
 * The year at a glance: a calendar whose colour is the time played each day and whose number
 * is the achievements unlocked that day, and under it the focused month, day by day, split by
 * game. The pad walks the days; LT and RT go a month at a time.
 */
export function Stats({ active }: { active: boolean }) {
  const games = useStore((s) => s.games);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [unlocks, setUnlocks] = useState<UnlockTimes[]>([]);
  const [names, setNames] = useState<Record<string, string>>({});
  const [focus, setFocus] = useState(() => midnight(Date.now()));

  // Read again whenever the tab comes up: a game may have ended since.
  useEffect(() => {
    if (!active) return;
    api.getSessions().then(setSessions, () => {});
    api.achievementTimes().then(setUnlocks, () => {});
    api.sessionNames().then(setNames, () => {});
  }, [active, games]);

  const days = useMemo(() => byDay(sessions), [sessions]);
  const achievements = useMemo(() => unlocksByDay(unlocks), [unlocks]);
  const today = midnight(Date.now());
  const year = new Date(focus).getFullYear();

  const calendar = useMemo(() => {
    const first = weekStart(new Date(year, 0, 1).getTime());
    const last = new Date(year, 11, 31).getTime();
    const weeks: number[][] = [];
    for (let w = first; w <= last; w = addDays(w, 7)) weeks.push(Array.from({ length: 7 }, (_, d) => addDays(w, d)));
    // Where each month's label goes: the first week holding its 1st.
    const months = Array.from({ length: 12 }, (_, m) => ({ m, week: weeks.findIndex((wk) => wk.some((d) => new Date(d).getMonth() === m && new Date(d).getDate() === 1 && new Date(d).getFullYear() === year)) }));
    return { weeks, months };
  }, [year]);

  const summary = useMemo(() => {
    let total = 0;
    let best = { key: "", seconds: 0 };
    let streak = 0;
    let run = 0;
    let unlocked = 0;
    for (let d = new Date(year, 0, 1).getTime(); new Date(d).getFullYear() === year && d <= today; d = addDays(d, 1)) {
      const key = keyOf(d);
      const s = days.get(key)?.seconds ?? 0;
      total += s;
      if (s > best.seconds) best = { key, seconds: s };
      run = s > 0 ? run + 1 : 0;
      streak = Math.max(streak, run);
      unlocked += achievements.get(key) ?? 0;
    }
    return { total, best, streak, unlocked };
  }, [days, achievements, year, today]);

  const month = new Date(focus).getMonth();
  const chart = useMemo(() => {
    const count = new Date(year, month + 1, 0).getDate();
    const list = Array.from({ length: count }, (_, i) => {
      const ms = new Date(year, month, i + 1).getTime();
      return { ms, key: keyOf(ms), stat: days.get(keyOf(ms)) };
    });
    const totals = new Map<string, number>();
    for (const d of list) d.stat?.games.forEach((s, g) => totals.set(g, (totals.get(g) ?? 0) + s));
    const ranked = [...totals].sort((a, b) => b[1] - a[1]).map(([g]) => g);
    const colored = ranked.slice(0, COLORS.length);
    const peak = Math.max(0, ...list.map((d) => d.stat?.seconds ?? 0));
    const top = Math.max(2, Math.ceil(peak / 3600 / 2) * 2);
    const seconds = list.reduce((sum, d) => sum + (d.stat?.seconds ?? 0), 0);
    const played = list.filter((d) => d.stat).length;
    const unlocked = list.reduce((sum, d) => sum + (achievements.get(d.key) ?? 0), 0);
    return { list, colored, others: ranked.length > colored.length, top, seconds, played, unlocked };
  }, [days, achievements, year, month]);

  const move = (n: number) => {
    const next = addDays(focus, n);
    if (next > today && n > 0 && focus >= today) return "edge" as const;
    setFocus(Math.min(next, today));
  };
  const moveMonth = (n: number) => {
    const d = new Date(focus);
    const target = new Date(d.getFullYear(), d.getMonth() + n, 1);
    if (target.getTime() > today) return "edge" as const;
    const lastDay = new Date(target.getFullYear(), target.getMonth() + 1, 0).getDate();
    setFocus(Math.min(today, new Date(target.getFullYear(), target.getMonth(), Math.min(d.getDate(), lastDay)).getTime()));
  };

  useInput(
    (a) => {
      switch (a) {
        case "left":
          return move(-7);
        case "right":
          return move(7);
        case "up":
          return (new Date(focus).getDay() + 6) % 7 === 0 ? "edge" : move(-1);
        case "down":
          return (new Date(focus).getDay() + 6) % 7 === 6 ? "edge" : move(1);
        case "lt":
          return moveMonth(-1);
        case "rt":
          return moveMonth(1);
        default:
          return false;
      }
    },
    Layer.view,
    active,
  );
  useHints([["dpad", "Days"], [["lt", "rt"], "Month"], ["menu", "Control center"]], Layer.view, active);

  const focusStat = days.get(keyOf(focus));
  const focusUnlocks = achievements.get(keyOf(focus)) ?? 0;
  // A Steam game no longer installed is named by the store; one it doesn't know, by its number.
  const nameOf = (id: string) => gameById(id)?.name ?? names[id] ?? (id.startsWith("steam:") ? t("Steam game {id}", { id: id.slice(6) }) : id);
  const colorOf = (id: string) => {
    const n = chart.colored.indexOf(id);
    return n >= 0 ? COLORS[n] : OTHER;
  };
  const monthName = new Date(year, month, 1).toLocaleDateString(locale(), { month: "long" });
  const weekdays = [0, 2, 4, 6].map((d) => new Date(2024, 0, 1 + d).toLocaleDateString(locale(), { weekday: "short" }));

  return (
    <main className="stats">
      <div className="stats-head">
        <h1 className="stats-year">{year}</h1>
        <div className="stats-sum">
          <div className="stat">
            <div className="stat-value">{Math.round(summary.total / 3600)}<span className="stat-unit">{t("h")}</span></div>
            <div className="stat-label">{t("This year")}</div>
          </div>
          <div className="stat">
            <div className="stat-value">{summary.streak}<span className="stat-unit">{summary.streak === 1 ? t("day") : t("days")}</span></div>
            <div className="stat-label">{t("Longest streak")}</div>
          </div>
          <div className="stat">
            <div className="stat-value">{summary.best.seconds ? playtime(minutesOf(summary.best.seconds)) : "—"}</div>
            <div className="stat-label">
              {summary.best.key ? t("Record · {date}", { date: fromKey(summary.best.key).toLocaleDateString(locale(), { day: "numeric", month: "short" }) }) : t("Record")}
            </div>
          </div>
          <div className="stat">
            <div className="stat-value">{summary.unlocked}</div>
            <div className="stat-label">{t("Achievements")}</div>
          </div>
        </div>
      </div>

      <section className="stats-card stats-calendar" aria-label={t("Calendar")} style={{ "--weeks": calendar.weeks.length } as React.CSSProperties}>
        <div className="cal-months">
          {calendar.months.map(({ m, week }) => (
            <span key={m} className={m === month ? "is-current" : ""} style={{ gridColumn: week + 1 }}>
              {new Date(year, m, 1).toLocaleDateString(locale(), { month: "short" })}
            </span>
          ))}
        </div>
        <div className="cal-body">
          <div className="cal-weekdays">
            {weekdays.map((w, n) => <span key={w} style={{ gridRow: n * 2 + 1 }}>{w}</span>)}
          </div>
          <div className="cal-grid">
            {calendar.weeks.map((week, w) => (
              <div key={w} className="cal-week">
                {week.map((d) => {
                  const key = keyOf(d);
                  const out = new Date(d).getFullYear() !== year || d > today;
                  const lvl = out ? 0 : level(days.get(key)?.seconds ?? 0);
                  const count = out ? 0 : achievements.get(key) ?? 0;
                  return (
                    <span key={key} className={`cal-day l${lvl} ${out ? "is-out" : ""} ${d === focus ? "is-focused" : ""}`} onClick={() => !out && setFocus(d)}>
                      {count > 0 ? count : ""}
                    </span>
                  );
                })}
              </div>
            ))}
          </div>
        </div>
        <div className="cal-foot">
          <span className="cal-legend">
            {t("Colour: time played")}
            {[0, 1, 2, 3, 4].map((l) => <span key={l} className={`cal-day l${l}`} />)}
            <span className="cal-legend-count">{t("Number: achievements unlocked")}</span>
          </span>
          <span className="cal-focus">
            <b>{new Date(focus).toLocaleDateString(locale(), { weekday: "long", day: "numeric", month: "long" })}</b>
            {" · "}
            {focusStat ? playtime(minutesOf(focusStat.seconds)) : t("Not played")}
            {focusUnlocks > 0 && <span className="cal-focus-ach"> · ★ {focusUnlocks}</span>}
          </span>
        </div>
      </section>

      <section className="stats-card stats-month" aria-label={monthName}>
        <div className="month-head">
          <div className="month-title">
            <Glyph b="lt" />
            <span className="month-name">{monthName}</span>
            <Glyph b="rt" />
            <span className="month-sum">
              {playtime(minutesOf(chart.seconds))} · {chart.played === 1 ? t("1 day played") : t("{n} days played", { n: chart.played })}
              {chart.unlocked > 0 && <> · {chart.unlocked === 1 ? t("1 achievement") : t("{n} achievements", { n: chart.unlocked })}</>}
            </span>
          </div>
          <div className="month-legend">
            {chart.colored.map((g) => (
              <span key={g}><i style={{ background: colorOf(g) }} />{nameOf(g)}</span>
            ))}
            {chart.others && <span><i style={{ background: OTHER }} />{t("Other games")}</span>}
          </div>
        </div>
        <div className="month-plot">
          {Array.from({ length: chart.top / 2 + 1 }, (_, n) => n * 2).map((h) => (
            <div key={h} className="month-line" style={{ bottom: `${(100 * h) / chart.top}%` }}>
              <span>{t("{h} h", { h })}</span>
            </div>
          ))}
          <div className="month-bars">
            {chart.list.map((d) => {
              const count = achievements.get(d.key) ?? 0;
              const parts = d.stat ? [...d.stat.games].sort((a, b) => colorRank(a[0]) - colorRank(b[0])) : [];
              return (
                <div key={d.key} className={`month-day ${d.ms === focus ? "is-focused" : ""} ${d.ms > today ? "is-future" : ""}`} onClick={() => d.ms <= today && setFocus(d.ms)}>
                  {count > 0 && <span className="month-ach">★ {count}</span>}
                  {parts.map(([g, s]) => (
                    <span key={g} className="month-part" style={{ height: `${(100 * s) / 3600 / chart.top}%`, background: colorOf(g) }} />
                  ))}
                </div>
              );
            })}
          </div>
        </div>
        <div className="month-days">
          {chart.list.map((d, n) => (
            <span key={d.key} className={d.ms === focus ? "is-focused" : d.ms === today ? "is-today" : d.ms > today ? "is-future" : ""}>{n + 1}</span>
          ))}
        </div>
      </section>
    </main>
  );

  function colorRank(id: string) {
    const n = chart.colored.indexOf(id);
    return n >= 0 ? n : COLORS.length;
  }
}
