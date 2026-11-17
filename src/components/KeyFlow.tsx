import { motion } from "motion/react";
import { useState } from "react";
import { api } from "../api";
import { errorText, t } from "../i18n";
import { Layer, useInput } from "../input";
import { ease } from "../motion";
import { toast } from "../store";
import { useHints } from "./Hints";
import { TextPrompt } from "./TextPrompt";

type Step = "open" | "paste" | "type" | "disconnect";

const ICONS: Record<Step, React.ReactNode> = {
  open: <path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" />,
  paste: <><rect x="6" y="4" width="12" height="17" rx="2" /><path d="M9 4V3h6v1M9 10h6M9 14h6" /></>,
  type: <><rect x="2.5" y="6" width="19" height="12" rx="2" /><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10" /></>,
  disconnect: <path d="M15 4h4v16h-4M10 8l-4 4 4 4M6 12h10" />,
};

/** What the flow asks for: an API key, or a link to a profile. */
function wordsFor(kind: "key" | "profile" | "link", service: string, site: string) {
  if (kind === "link")
    return {
      empty: t("The clipboard is empty. Copy the game's link first."),
      opened: t("Opened {site} in the browser. Find the game, copy its link, then come back.", { site }),
      checking: t("Checking the link…"),
      paste: t("Paste the game's link from clipboard"),
      type: t("Type the game's link"),
      prompt: t("{service} link", { service }),
    };
  return kind === "key"
    ? {
        empty: t("The clipboard is empty. Copy the key first."),
        opened: t("Opened {site} in the browser. Copy the key there, then come back.", { site }),
        checking: t("Checking the key…"),
        paste: t("Paste key from clipboard"),
        type: t("Type the key"),
        prompt: t("{service} key", { service }),
      }
    : {
        empty: t("The clipboard is empty. Copy the profile link first."),
        opened: t("Opened {site} in the browser. Copy your profile link there, then come back.", { site }),
        checking: t("Checking the profile…"),
        paste: t("Paste profile link from clipboard"),
        type: t("Type the profile link or ID"),
        prompt: t("Steam profile link or ID"),
      };
}

/**
 * Connecting a service by API key, built for a gamepad: open the provider's page in the
 * browser, copy the key there, come back and paste. Typing stays as a fallback.
 */
export function KeyFlow({
  service,
  title,
  intro,
  howTo,
  url,
  site,
  status,
  connected,
  onSubmit,
  onDisconnect,
  onClose,
  kind = "key",
  disconnectLabel,
  onOpen,
  openLabel,
}: {
  service: string;
  title: string;
  intro: string;
  howTo: string[];
  url: string;
  site: string;
  status?: string;
  connected: boolean;
  onSubmit: (key: string) => Promise<void>;
  onDisconnect?: () => Promise<void>;
  onClose: () => void;
  /** A key by default; "profile" asks for a profile link, "link" for a page's address. */
  kind?: "key" | "profile" | "link";
  disconnectLabel?: string;
  /** Does the first step in Mantel instead of opening the site in the browser. */
  onOpen?: () => Promise<void>;
  openLabel?: string;
}) {
  const words = wordsFor(kind, service, site);
  const steps: Step[] = ["open", "paste", "type", ...(connected && onDisconnect ? (["disconnect"] as Step[]) : [])];
  const [index, setIndex] = useState(connected ? 1 : 0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [typing, setTyping] = useState(false);

  const submit = async (key: string) => {
    if (!key.trim()) return setError(words.empty);
    setBusy(true);
    setError(null);
    try {
      await onSubmit(key.trim());
      if (kind === "key") toast(t("{service} connected", { service }), "good");
      onClose();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const run = async (step: Step) => {
    if (busy) return;
    if (step === "open" && onOpen) {
      setError(null);
      setBusy(true);
      await onOpen().catch((e) => String(e) !== "closed" && setError(errorText(e)));
      setBusy(false);
    } else if (step === "open") {
      await api.openUrl(url).catch((e) => setError(errorText(e)));
      toast(words.opened);
    } else if (step === "paste") {
      submit(await api.readClipboard().catch(() => ""));
    } else if (step === "type") {
      setTyping(true);
    } else if (step === "disconnect" && onDisconnect) {
      await onDisconnect();
      if (!disconnectLabel) toast(t("{service} disconnected", { service }));
      onClose();
    }
  };

  const label: Record<Step, string> = {
    open: openLabel ?? t("Open {site}", { site }),
    paste: busy ? words.checking : words.paste,
    type: words.type,
    disconnect: disconnectLabel ?? t("Disconnect {service}", { service }),
  };

  useInput(
    (a) => {
      if (busy) return "silent";
      if (a === "b") return onClose();
      if (a === "up") return index > 0 ? setIndex(index - 1) : "edge";
      if (a === "down") return index < steps.length - 1 ? setIndex(index + 1) : "edge";
      if (a === "a") return void run(steps[index]);
      return "silent";
    },
    Layer.modal,
    !typing,
  );
  useHints([["a", "Select"], ["b", "Close"]], Layer.modal, !typing);

  return (
    <motion.div className="keyflow" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
      <motion.div className="keyflow-card" initial={{ opacity: 0, y: 24, scale: 0.98 }} animate={{ opacity: 1, y: 0, scale: 1 }} transition={{ duration: 0.5, ease }}>
        <div className="eyebrow">{service}</div>
        <h2 className="keyflow-title">{title}</h2>
        <p className="keyflow-intro">{intro}</p>
        <ol className="keyflow-howto">
          {howTo.map((h) => (
            <li key={h}>{h}</li>
          ))}
        </ol>
        {status && <div className={`keyflow-status ${connected ? "ok" : ""}`}>{status}</div>}
        <div className="keyflow-steps">
          {steps.map((s, n) => (
            <button
              key={s}
              className={`keyflow-step ${n === index ? "is-focused" : ""} ${s === "disconnect" ? "danger" : ""}`}
              onClick={() => {
                setIndex(n);
                run(s);
              }}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                {ICONS[s]}
              </svg>
              {label[s]}
              {busy && s === "paste" && <span className="spinner" />}
            </button>
          ))}
        </div>
        {error && <div className="keyflow-error">{error}</div>}
      </motion.div>
      {typing && (
        <TextPrompt
          title={words.prompt}
          hint={t("Type or paste with Ctrl+V.")}
          initial=""
          onDone={(v) => {
            setTyping(false);
            if (v) submit(v);
          }}
        />
      )}
    </motion.div>
  );
}
