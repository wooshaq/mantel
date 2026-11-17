import { motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import { useScrollFocus } from "../hooks";
import { locale, t } from "../i18n";
import { Layer, useInput } from "../input";
import type { Game, Session } from "../types";
import { playtime } from "../util";
import { useHints } from "./Hints";

const PAGE = 6;

/** A local calendar day, as "2026-11-17". */
export function dayKey(unix: number): string {
  const d = new Date(unix * 1000);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

/** Minutes for a duration in seconds: anything played counts as at least a minute. */
export const minutesOf = (seconds: number) => Math.max(1, Math.round(seconds / 60));

interface Day {
  key: string;
  /** When the day's first session started. */
  start: number;
  seconds: number;
  /** Days without play before the next day shown (the one above it in the list). */
  gapAfter: number;
}

/** A game's sessions folded into days, newest first. */
function daysOf(sessions: Session[], game: string): Day[] {
  const byDay = new Map<string, Day>();
  for (const s of sessions) {
    if (s.game !== game || s.end <= s.start) continue;
    const key = dayKey(s.start);
    const day = byDay.get(key) ?? { key, start: s.start, seconds: 0, gapAfter: 0 };
    day.start = Math.min(day.start, s.start);
    day.seconds += s.end - s.start;
    byDay.set(key, day);
  }
  const days = [...byDay.values()].sort((a, b) => b.start - a.start);
  const midnight = (unix: number) => new Date(new Date(unix * 1000).toDateString()).getTime();
  for (let i = 1; i < days.length; i++) {
    days[i].gapAfter = Math.round((midnight(days[i - 1].start) - midnight(days[i].start)) / 86_400_000) - 1;
  }
  return days;
}

/**
 * Every day the game was played, newest first, like a feed: how long, the breaks between, and
 * the longest day. Built from the sessions Mantel records.
 */
export function PlayHistory({ game, onClose }: { game: Game; onClose: () => void }) {
  const [sessions, setSessions] = useState<Session[] | null>(null);
  const [index, setIndex] = useState(0);
  const box = useRef<HTMLDivElement>(null);
  useScrollFocus(box, index);

  useEffect(() => {
    api.getSessions().then(setSessions, () => setSessions([]));
  }, []);
  const days = useMemo(() => (sessions ? daysOf(sessions, game.id) : []), [sessions, game.id]);
  const longest = days.reduce((best, d) => (d.seconds > (best?.seconds ?? 0) ? d : best), undefined as Day | undefined);
  const total = days.reduce((sum, d) => sum + d.seconds, 0);
  const count = sessions?.filter((s) => s.game === game.id).length ?? 0;
  const first = days[days.length - 1];

  useInput(
    (a) => {
      switch (a) {
        case "up":
          return index > 0 ? setIndex(index - 1) : "edge";
        case "down":
          return index < days.length - 1 ? setIndex(index + 1) : "edge";
        case "lb":
        case "lt":
          return index > 0 ? setIndex(Math.max(0, index - PAGE)) : "edge";
        case "rb":
        case "rt":
          return index < days.length - 1 ? setIndex(Math.min(days.length - 1, index + PAGE)) : "edge";
        case "b":
          return onClose();
        default:
          return "silent";
      }
    },
    Layer.overlay,
  );
  useHints([["dpad", "Browse"], ["b", "Back"]], Layer.overlay);

  const date = (unix: number) => new Date(unix * 1000).toLocaleDateString(locale(), { day: "numeric", month: "short" });
  const time = (unix: number) => new Date(unix * 1000).toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" });

  return (
    <motion.div className="ach history" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
      <div className="ach-head">
        <div className="eyebrow">{t("Play history")}</div>
        <h2 className="ach-title">{game.name}</h2>
        <div className="ach-summary">
          {first ? (
            <>
              <span>{playtime(minutesOf(total))}</span>
              <span>{count === 1 ? t("1 session") : t("{n} sessions", { n: count })}</span>
              <span>{t("since {date}", { date: date(first.start) })}</span>
            </>
          ) : (
            <span>&nbsp;</span>
          )}
        </div>
      </div>
      <div className="ach-list history-list" ref={box}>
        {sessions && !days.length && <div className="ach-note">{t("Mantel writes down every session from now on. Play it and it shows up here.")}</div>}
        {days.map((d, n) => (
          <div key={d.key} className="history-entry">
            {d.gapAfter >= 2 && <div className="history-gap">{t("{n} days without playing", { n: d.gapAfter })}</div>}
            <div className={`history-day ${n === index ? "is-focused" : ""}`} onClick={() => setIndex(n)}>
              <div className="history-when">
                <b>{date(d.start)}</b>
                <span>{time(d.start)}</span>
              </div>
              <span className={`history-dot ${n === 0 ? "is-latest" : ""}`} />
              <div className="history-what">
                {t("Played {time}", { time: playtime(minutesOf(d.seconds)) })}
                {longest === d && days.length > 1 && <span className="history-badge">{t("Longest day")}</span>}
              </div>
            </div>
          </div>
        ))}
      </div>
    </motion.div>
  );
}
