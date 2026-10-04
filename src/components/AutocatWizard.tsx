import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Locale, MessageKey } from "../i18n";

type I18nStr = { en: string; zh: string };
type Conflict = "skip" | "merge" | "replace";
type RulePack = {
  id: string;
  name: I18nStr;
  description: I18nStr;
  recommended: boolean;
  needs_metadata: boolean;
  needs_achievements: boolean;
  default_conflict: Conflict;
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
  selectedUser: string;
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
  selectedUser,
  onClose,
  onQueued,
  t,
}: Props) {
  const [packs, setPacks] = useState<RulePack[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [conflict, setConflict] = useState<Conflict>("merge");
  const [preview, setPreview] = useState<AutoCatSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [achNote, setAchNote] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    void invoke<RulePack[]>("list_rule_packs")
      .then((list) => {
        setPacks(list);
        const rec = list.find((p) => p.recommended) ?? list[0];
        setSelected(rec?.id ?? "");
        setConflict(rec?.default_conflict ?? "merge");
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
        <h2>AutoCat</h2>
        <p className="muted">{t("tagline")}</p>
        {error && <div className="banner error">{error}</div>}
        <label className="field">
          <span>Rule pack</span>
          <select
            value={selected}
            onChange={(e) => {
              const id = e.target.value;
              setSelected(id);
              setPreview(null);
              const p = packs.find((x) => x.id === id);
              if (p) setConflict(p.default_conflict);
            }}
          >
            {packs.map((p) => (
              <option key={p.id} value={p.id}>
                {label(p.name, locale)}
                {p.recommended ? " ★" : ""}
                {p.needs_achievements ? " 🏆" : ""}
              </option>
            ))}
          </select>
        </label>
        {pack && <p className="muted">{label(pack.description, locale)}</p>}
        <label className="field" style={{ marginTop: "0.75rem" }}>
          <span>
            {locale === "zh"
              ? "覆盖策略"
              : "Conflict policy"}
          </span>
          <select
            value={conflict}
            onChange={(e) => {
              setConflict(e.target.value as Conflict);
              setPreview(null);
            }}
          >
            <option value="skip">
              {locale === "zh" ? "跳过同名" : "Skip same-named"}
            </option>
            <option value="merge">
              {locale === "zh" ? "合并追加" : "Merge / append"}
            </option>
            <option value="replace">
              {locale === "zh" ? "替换成员" : "Replace members"}
            </option>
          </select>
        </label>
        {metaLow && (
          <p className="muted">
            {locale === "zh"
              ? "元数据覆盖偏低，请先到设置拉取。"
              : "Metadata coverage is low — enrich in Settings first."}
          </p>
        )}
        {pack?.needs_achievements && (
          <div className="row" style={{ marginTop: "0.5rem" }}>
            <button
              type="button"
              disabled={busy || !selectedUser}
              onClick={() => {
                setBusy(true);
                setError(null);
                void invoke<{ fetched: number; failed: number; cached: number }>(
                  "fetch_session_achievements",
                  { userId3: selectedUser, maxApps: 40 },
                )
                  .then((r) =>
                    setAchNote(
                      locale === "zh"
                        ? `成就：新拉取 ${r.fetched}，缓存 ${r.cached}，失败 ${r.failed}`
                        : `Achievements: fetched ${r.fetched}, cached ${r.cached}, failed ${r.failed}`,
                    ),
                  )
                  .catch((e) => setError(String(e)))
                  .finally(() => setBusy(false));
              }}
            >
              {locale === "zh" ? "拉取成就（40）" : "Fetch achievements (40)"}
            </button>
          </div>
        )}
        {achNote && <p className="muted">{achNote}</p>}
        <div className="row" style={{ marginTop: "0.75rem" }}>
          <button
            type="button"
            disabled={busy || !selected}
            onClick={() => {
              setBusy(true);
              setError(null);
              void invoke<AutoCatSummary>("preview_rule_pack", {
                packId: selected,
                conflict,
              })
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
