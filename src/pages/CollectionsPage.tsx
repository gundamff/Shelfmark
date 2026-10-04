import {
  formatPendingSummary,
  isPendingEmpty,
  type CollectionChangeset,
} from "../lib/pending";
import type { MessageKey } from "../i18n";

type Game = { app_id: number; name: string };
type Collection = { id: string; name: string; app_ids: number[] };

type AutoCatSummary = {
  matchedGames: number;
  skippedMissingMeta: number;
  createCount: number;
  addMemberOps: number;
};

type Props = {
  busy: boolean;
  hasSession: boolean;
  steamRunning: boolean;
  games: Game[];
  collections: Collection[];
  pending: CollectionChangeset | null;
  autocat: AutoCatSummary | null;
  selectedGames: number[];
  targetCollection: string;
  newCollectionName: string;
  gameQuery: string;
  metaCoverage: number | null;
  onGameQuery: (q: string) => void;
  onToggleGame: (id: number) => void;
  onNewName: (n: string) => void;
  onTargetCollection: (id: string) => void;
  onCreate: () => void;
  onAdd: () => void;
  onRemove: () => void;
  onAutocat: () => void;
  onOpenWizard: () => void;
  onQueueAutocat: () => void;
  onExport: () => void;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

export function CollectionsPage({
  busy,
  hasSession,
  steamRunning,
  games,
  collections,
  pending,
  autocat,
  selectedGames,
  targetCollection,
  newCollectionName,
  gameQuery,
  metaCoverage,
  onGameQuery,
  onToggleGame,
  onNewName,
  onTargetCollection,
  onCreate,
  onAdd,
  onRemove,
  onAutocat,
  onOpenWizard,
  onQueueAutocat,
  onExport,
  t,
}: Props) {
  if (!hasSession) {
    return (
      <section className="panel">
        <h2>{t("navCollections")}</h2>
        <p className="muted">{t("noSessionHint")}</p>
      </section>
    );
  }

  const filteredGames = games.filter((g) => {
    const q = gameQuery.trim().toLowerCase();
    if (!q) return true;
    return g.name.toLowerCase().includes(q) || String(g.app_id).includes(q);
  });

  return (
    <>
      <section className="panel">
        <h2>{t("manualEdit")}</h2>
        <div className="row">
          <input
            value={newCollectionName}
            placeholder={t("newCollectionPlaceholder")}
            onChange={(e) => onNewName(e.target.value)}
          />
          <button
            type="button"
            disabled={busy || !newCollectionName.trim()}
            onClick={onCreate}
          >
            {t("createCollection")}
          </button>
        </div>
        <div className="row" style={{ marginTop: "0.75rem" }}>
          <select
            value={targetCollection}
            onChange={(e) => onTargetCollection(e.target.value)}
            disabled={!collections.length}
          >
            {collections.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name} ({c.app_ids.length})
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={busy || !targetCollection || selectedGames.length === 0}
            onClick={onAdd}
          >
            {t("addSelected")}
          </button>
          <button
            type="button"
            disabled={busy || !targetCollection || selectedGames.length === 0}
            onClick={onRemove}
          >
            {t("removeSelected")}
          </button>
        </div>
        <p className="muted">
          {t("selectedGamesHint", { count: selectedGames.length })}
        </p>
      </section>

      <section className="panel split">
        <div>
          <h2>{t("games")}</h2>
          <input
            value={gameQuery}
            placeholder={t("searchPlaceholder")}
            onChange={(e) => onGameQuery(e.target.value)}
            style={{ marginBottom: "0.6rem", width: "100%" }}
          />
          <p className="muted">
            {t("gamesShowing", {
              shown: filteredGames.length,
              total: games.length,
            })}
            {metaCoverage != null ? t("metaCount", { count: metaCoverage }) : ""}
          </p>
          <ul className="checklist">
            {filteredGames.map((g) => (
              <li key={g.app_id}>
                <label>
                  <input
                    type="checkbox"
                    checked={selectedGames.includes(g.app_id)}
                    onChange={() => onToggleGame(g.app_id)}
                  />{" "}
                  {g.name} <span className="muted">({g.app_id})</span>
                </label>
              </li>
            ))}
          </ul>
        </div>
        <div>
          <h2>{t("collections")}</h2>
          <ul>
            {collections.map((c) => (
              <li key={c.id}>
                <button
                  type="button"
                  className="linkish"
                  disabled={busy}
                  onClick={() => onTargetCollection(c.id)}
                >
                  {c.name}
                </button>{" "}
                <span className="muted">
                  {t("appsCount", { count: c.app_ids.length })}
                </span>
              </li>
            ))}
          </ul>
        </div>
      </section>

      <section className="panel">
        <h2>{t("metadataAutocat")}</h2>
        <div className="row">
          <button type="button" disabled={busy} onClick={onOpenWizard}>
            AutoCat…
          </button>
          <button type="button" disabled={busy} onClick={onAutocat}>
            {t("dryRunGenre")}
          </button>
          <button
            type="button"
            disabled={busy || !autocat}
            onClick={onQueueAutocat}
          >
            {t("queuePending")}
          </button>
        </div>
        {autocat && (
          <p className="muted">
            {t("autocatReport", {
              matched: autocat.matchedGames,
              skipped: autocat.skippedMissingMeta,
              creates: autocat.createCount,
              addOps: autocat.addMemberOps,
            })}
          </p>
        )}
      </section>

      <section className="panel pending-bar">
        <h2>{t("pending")}</h2>
        {pending ? (
          <>
            <p>
              {formatPendingSummary(pending, (key, params) => t(key, params))}
            </p>
            <pre className="code">{JSON.stringify(pending, null, 2)}</pre>
          </>
        ) : (
          <p className="muted">{t("noPending")}</p>
        )}
        <button
          type="button"
          disabled={
            busy ||
            steamRunning ||
            !pending ||
            isPendingEmpty(pending)
          }
          onClick={onExport}
        >
          {t("export")}
        </button>
        {steamRunning && (
          <p className="muted">{t("errSteamRunning")}</p>
        )}
      </section>
    </>
  );
}
