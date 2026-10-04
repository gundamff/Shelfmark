import type { LocalePreference, MessageKey } from "../i18n";

export type SteamInfo = {
  root: string;
  users: string[];
  steamRunning: boolean;
  supportedSteam: string;
  supportedSteamNote: string;
  detectedSteamVersion: string | null;
  steamWithinSupport: boolean | null;
};

export type SessionSummary = {
  mode: string;
  gameCount: number;
  collectionCount: number;
  storePath: string;
};

type Props = {
  busy: boolean;
  localePref: LocalePreference;
  onLocaleChange: (value: string) => void;
  steam: SteamInfo | null;
  selectedUser: string;
  onSelectUser: (id: string) => void;
  session: SessionSummary | null;
  gameCount: number;
  collectionCount: number;
  metaCoverage: number | null;
  enrichReport: string | null;
  apiKey: string;
  onApiKeyChange: (v: string) => void;
  onDetectSteam: () => void;
  onOpenDemo: () => void;
  onOpenLive: () => void;
  onEnrich: () => void;
  onEnrichStop: () => void;
  enrichRunning: boolean;
  enrichDetail: string | null;
  onSaveApiKey: () => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

export function SettingsPage({
  busy,
  localePref,
  onLocaleChange,
  steam,
  selectedUser,
  onSelectUser,
  session,
  gameCount,
  collectionCount,
  metaCoverage,
  enrichReport,
  apiKey,
  onApiKeyChange,
  onDetectSteam,
  onOpenDemo,
  onOpenLive,
  onEnrich,
  onEnrichStop,
  enrichRunning,
  enrichDetail,
  onSaveApiKey,
  t,
}: Props) {
  return (
    <>
      <section className="panel">
        <h2>{t("language")}</h2>
        <label className="field">
          <select
            value={localePref}
            disabled={busy}
            onChange={(e) => onLocaleChange(e.target.value)}
          >
            <option value="system">{t("langSystem")}</option>
            <option value="en">{t("langEn")}</option>
            <option value="zh">{t("langZh")}</option>
          </select>
        </label>
      </section>

      <section className="panel">
        <h2>{t("steam")}</h2>
        <button type="button" disabled={busy} onClick={onDetectSteam}>
          {t("detectSteam")}
        </button>
        {steam && (
          <dl className="kv">
            <dt>{t("root")}</dt>
            <dd>{steam.root}</dd>
            <dt>{t("users")}</dt>
            <dd>{steam.users.join(", ") || t("none")}</dd>
            <dt>{t("running")}</dt>
            <dd>{steam.steamRunning ? t("runningYes") : t("runningNo")}</dd>
          </dl>
        )}
      </section>

      <section className="panel">
        <h2>{t("session")}</h2>
        <div className="row">
          <button type="button" disabled={busy} onClick={onOpenDemo}>
            {t("openDemo")}
          </button>
          <select
            value={selectedUser}
            disabled={!steam || busy}
            onChange={(e) => onSelectUser(e.target.value)}
          >
            {(steam?.users ?? []).map((u) => (
              <option key={u} value={u}>
                {u}
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={busy || !selectedUser}
            onClick={onOpenLive}
          >
            {t("openLive")}
          </button>
        </div>
        {session && (
          <dl className="kv">
            <dt>{t("mode")}</dt>
            <dd>{session.mode}</dd>
            <dt>{t("gamesCollections")}</dt>
            <dd>
              {gameCount} / {collectionCount}
            </dd>
            <dt>{t("store")}</dt>
            <dd className="path">{session.storePath}</dd>
          </dl>
        )}
      </section>

      <section className="panel">
        <h2>{t("apiKey")}</h2>
        <div className="row">
          <input
            type="password"
            autoComplete="off"
            value={apiKey}
            placeholder={t("apiKeyPlaceholder")}
            onChange={(e) => onApiKeyChange(e.target.value)}
            style={{ flex: 1, minWidth: "14rem" }}
          />
          <button type="button" disabled={busy || !apiKey.trim()} onClick={onSaveApiKey}>
            {t("saveApiKey")}
          </button>
        </div>
      </section>

      <section className="panel">
        <h2>{t("metadataAutocat")}</h2>
        <p className="muted">
          {metaCoverage != null
            ? t("metaCount", { count: metaCoverage }).replace(/^ · /, "")
            : t("noSessionHint")}
        </p>
        {session && gameCount > 0 && (
          <div
            className="progress"
            aria-valuenow={metaCoverage ?? 0}
            aria-valuemin={0}
            aria-valuemax={gameCount}
          >
            <div
              className="progress-fill"
              style={{
                width: `${Math.min(100, ((metaCoverage ?? 0) / gameCount) * 100)}%`,
              }}
            />
          </div>
        )}
        <div className="row" style={{ marginTop: "0.75rem" }}>
          <button
            type="button"
            disabled={busy || !session || enrichRunning}
            onClick={onEnrich}
          >
            {t("enrichMetadata")}
          </button>
          <button
            type="button"
            disabled={!enrichRunning}
            onClick={onEnrichStop}
          >
            Stop
          </button>
        </div>
        {enrichDetail && <p className="muted">{enrichDetail}</p>}
        {enrichReport && <p className="muted">{enrichReport}</p>}
      </section>
    </>
  );
}
