import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Locale, MessageKey } from "../i18n";

type I18nStr = { en: string; zh: string };
type RulePack = {
  id: string;
  name: I18nStr;
  description: I18nStr;
  recommended: boolean;
  needs_metadata: boolean;
  needs_achievements: boolean;
  default_conflict: "skip" | "merge" | "replace";
};

type AutoCatSummary = {
  matchedGames: number;
  skippedMissingMeta: number;
  createCount: number;
  addMemberOps: number;
};

type Props = {
  open: boolean;
  locale: Locale;
  metaCoverage: number | null;
  gameCount: number;
  onClose: () => void;
  onQueued: () => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

function label(s: I18nStr, locale: Locale): string {
  return locale === "zh" ? s.zh || s.en : s.en || s.zh;
}

export function AutocatWizard({
  open,
  locale,
  metaCoverage,
  gameCount,
  onClose,
  onQueued,
  t,
}: Props) {
  const [packs, setPacks] = useState<RulePack[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [preview, setPreview] = useState<AutoCatSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    void invoke<RulePack[]>("list_rule_packs")
      .then((list) => {
        setPacks(list);
        const rec = list.find((p) => p.recommended) ?? list[0];
        setSelected(rec?.id ?? "");
      })
      .catch((e) => setError(String(e)));
  }, [open]);

  if (!open) return null;

  const pack = packs.find((p) => p.id === selected);
  const metaLow =
    !!pack?.needs_metadata &&
    (metaCoverage == null || (gameCount > 0 && metaCoverage / gameCount < 0.2));

  return (
    <div className="modal-mask" role="presentation" onClick={onClose}>
      <div
        className="modal-card"
        style={{ width: "min(560px, 100%)", textAlign: "left" }}
        role="dialog"
        onClick={(e) => e.stopPropagation()}
      >
        <h2>{t("dryRunGenre").replace("Genre", "AutoCat")}</h2>
        <p className="muted">{t("tagline")}</p>
        {error && <div className="banner error">{error}</div>}
        <label className="field">
          <span>Rule pack</span>
          <select
            value={selected}
            onChange={(e) => {
              setSelected(e.target.value);
              setPreview(null);
            }}
          >
            {packs.map((p) => (
              <option
                key={p.id}
                value={p.id}
                disabled={p.needs_achievements}
              >
                {label(p.name, locale)}
                {p.recommended ? " ★" : ""}
                {p.needs_achievements ? " (needs achievements)" : ""}
              </option>
            ))}
          </select>
        </label>
        {pack && (
          <p className="muted">{label(pack.description, locale)}</p>
        )}
        {metaLow && (
          <p className="muted">{t("compatWarnOutside")} — metadata low; enrich in Settings.</p>
        )}
        <div className="row" style={{ marginTop: "0.75rem" }}>
          <button
            type="button"
            disabled={busy || !selected}
            onClick={() => {
              setBusy(true);
              setError(null);
              void invoke<AutoCatSummary>("preview_rule_pack", { packId: selected })
                .then(setPreview)
                .catch((e) => setError(String(e)))
                .finally(() => setBusy(false));
            }}
          >
            Preview
          </button>
          <button
            type="button"
            disabled={busy || !preview}
            onClick={() => {
              setBusy(true);
              void invoke("queue_last_autocat")
                .then(() => {
                  onQueued();
                  onClose();
                })
                .catch((e) => setError(String(e)))
                .finally(() => setBusy(false));
            }}
          >
            {t("queuePending")}
          </button>
          <button type="button" onClick={onClose}>
            {t("aboutClose")}
          </button>
        </div>
        {preview && (
          <p className="muted" style={{ marginTop: "0.75rem" }}>
            {t("autocatReport", {
              matched: preview.matchedGames,
              skipped: preview.skippedMissingMeta,
              creates: preview.createCount,
              addOps: preview.addMemberOps,
            })}
          </p>
        )}
      </div>
    </div>
  );
}
