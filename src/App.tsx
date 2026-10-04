import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  formatPendingSummary,
  isPendingEmpty,
  type CollectionChangeset,
} from "./lib/pending";
import {
  localizeError,
  parseLocalePreference,
  resolveLocale,
  t,
  type LocalePreference,
} from "./i18n";
import "./App.css";

type SteamInfo = {
  root: string;
  users: string[];
  steamRunning: boolean;
};

type SessionSummary = {
  mode: string;
  gameCount: number;
  collectionCount: number;
  storePath: string;
};

type Game = { app_id: number; name: string };
type Collection = { id: string; name: string; app_ids: number[] };

type AutoCatSummary = {
  matchedGames: number;
  skippedMissingMeta: number;
  createCount: number;
  addMemberOps: number;
};

type EnrichReport = {
  fromCache: number;
  fetched: number;
  failed: number;
  alreadyPresent: number;
  skippedBudget: number;
};

type AppConfig = {
  last_user_id3?: string | null;
  steam_path_override?: string | null;
  locale?: string | null;
};

function App() {
  const [error, setError] = useState<string | null>(null);
  const [steam, setSteam] = useState<SteamInfo | null>(null);
  const [session, setSession] = useState<SessionSummary | null>(null);
  const [games, setGames] = useState<Game[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [pending, setPending] = useState<CollectionChangeset | null>(null);
  const [autocat, setAutocat] = useState<AutoCatSummary | null>(null);
  const [exportMsg, setExportMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [selectedUser, setSelectedUser] = useState<string>("");
  const [selectedGames, setSelectedGames] = useState<number[]>([]);
  const [targetCollection, setTargetCollection] = useState<string>("");
  const [newCollectionName, setNewCollectionName] = useState("");
  const [gameQuery, setGameQuery] = useState("");
  const [metaCoverage, setMetaCoverage] = useState<number | null>(null);
  const [enrichReport, setEnrichReport] = useState<string | null>(null);
  const [localePref, setLocalePref] = useState<LocalePreference>("system");

  const locale = useMemo(
    () => resolveLocale(localePref, navigator.language),
    [localePref],
  );
  const tr = useCallback(
    (key: Parameters<typeof t>[1], params?: Record<string, string | number>) =>
      t(locale, key, params),
    [locale],
  );

  const run = useCallback(
    async (label: string, fn: () => Promise<void>) => {
      setBusy(true);
      setError(null);
      try {
        await fn();
      } catch (e) {
        const raw = `${label}: ${String(e)}`;
        setError(localizeError(locale, raw));
      } finally {
        setBusy(false);
      }
    },
    [locale],
  );

  const refreshLists = useCallback(async () => {
    const [g, c, p] = await Promise.all([
      invoke<Game[]>("list_games"),
      invoke<Collection[]>("list_collections"),
      invoke<CollectionChangeset>("pending_changeset"),
    ]);
    setGames(g);
    setCollections(c);
    setPending(p);
    setTargetCollection((prev) => {
      if (prev && c.some((x) => x.id === prev)) return prev;
      return c[0]?.id ?? "";
    });
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const cfg = await invoke<AppConfig>("get_config");
        if (cfg.last_user_id3) {
          setSelectedUser(cfg.last_user_id3);
        }
        setLocalePref(parseLocalePreference(cfg.locale));
      } catch {
        // ignore missing config
      }
    })();
  }, []);

  const toggleGame = (appId: number) => {
    setSelectedGames((prev) =>
      prev.includes(appId) ? prev.filter((id) => id !== appId) : [...prev, appId],
    );
  };

  const filteredGames = games.filter((g) => {
    const q = gameQuery.trim().toLowerCase();
    if (!q) return true;
    return (
      g.name.toLowerCase().includes(q) || String(g.app_id).includes(q)
    );
  });

  const refreshMetaCoverage = async () => {
    try {
      setMetaCoverage(await invoke<number>("metadata_coverage"));
    } catch {
      setMetaCoverage(null);
    }
  };

  const onLocaleChange = (value: string) => {
    const pref = parseLocalePreference(value);
    setLocalePref(pref);
    void run("save_locale", async () => {
      await invoke("save_locale", { locale: pref });
    });
  };

  return (
    <main className="app">
      <header className="hero">
        <h1>Shelfmark</h1>
        <p className="tagline">{tr("tagline")}</p>
      </header>

      <section className="panel">
        <h2>{tr("settings")}</h2>
        <div className="row">
          <label className="field">
            <span>{tr("language")}</span>
            <select
              value={localePref}
              disabled={busy}
              onChange={(e) => onLocaleChange(e.target.value)}
            >
              <option value="system">{tr("langSystem")}</option>
              <option value="en">{tr("langEn")}</option>
              <option value="zh">{tr("langZh")}</option>
            </select>
          </label>
        </div>
      </section>

      {error && <div className="banner error">{error}</div>}
      {exportMsg && <div className="banner ok">{exportMsg}</div>}

      <section className="panel">
        <h2>{tr("steam")}</h2>
        <button
          disabled={busy}
          onClick={() =>
            run("detect_steam", async () => {
              const info = await invoke<SteamInfo>("detect_steam");
              setSteam(info);
              setSelectedUser((prev) => {
                if (prev && info.users.includes(prev)) return prev;
                return info.users[0] ?? "";
              });
            })
          }
        >
          {tr("detectSteam")}
        </button>
        {steam && (
          <dl className="kv">
            <dt>{tr("root")}</dt>
            <dd>{steam.root}</dd>
            <dt>{tr("users")}</dt>
            <dd>{steam.users.join(", ") || tr("none")}</dd>
            <dt>{tr("running")}</dt>
            <dd>{steam.steamRunning ? tr("runningYes") : tr("runningNo")}</dd>
          </dl>
        )}
      </section>

      <section className="panel">
        <h2>{tr("session")}</h2>
        <div className="row">
          <button
            disabled={busy}
            onClick={() =>
              run("open_demo_session", async () => {
                const s = await invoke<SessionSummary>("open_demo_session");
                setSession(s);
                setAutocat(null);
                setExportMsg(null);
                setSelectedGames([]);
                setEnrichReport(null);
                await refreshLists();
                await refreshMetaCoverage();
              })
            }
          >
            {tr("openDemo")}
          </button>
          <select
            value={selectedUser}
            disabled={!steam || busy}
            onChange={(e) => setSelectedUser(e.target.value)}
          >
            {(steam?.users ?? []).map((u) => (
              <option key={u} value={u}>
                {u}
              </option>
            ))}
          </select>
          <button
            disabled={busy || !selectedUser}
            onClick={() =>
              run("open_live_session", async () => {
                await invoke("save_last_user", { userId3: selectedUser });
                const s = await invoke<SessionSummary>("open_live_session", {
                  userId3: selectedUser,
                });
                setSession(s);
                setAutocat(null);
                setExportMsg(null);
                setSelectedGames([]);
                setEnrichReport(null);
                await refreshLists();
                await refreshMetaCoverage();
              })
            }
          >
            {tr("openLive")}
          </button>
        </div>
        {session && (
          <dl className="kv">
            <dt>{tr("mode")}</dt>
            <dd>{session.mode}</dd>
            <dt>{tr("gamesCollections")}</dt>
            <dd>
              {games.length} / {collections.length}
            </dd>
            <dt>{tr("store")}</dt>
            <dd className="path">{session.storePath}</dd>
          </dl>
        )}
      </section>

      {session && (
        <>
          <section className="panel">
            <h2>{tr("manualEdit")}</h2>
            <div className="row">
              <input
                value={newCollectionName}
                placeholder={tr("newCollectionPlaceholder")}
                onChange={(e) => setNewCollectionName(e.target.value)}
              />
              <button
                disabled={busy || !newCollectionName.trim()}
                onClick={() =>
                  run("create_collection", async () => {
                    setPending(
                      await invoke<CollectionChangeset>("create_collection", {
                        name: newCollectionName.trim(),
                      }),
                    );
                    setNewCollectionName("");
                    await refreshLists();
                  })
                }
              >
                {tr("createCollection")}
              </button>
            </div>
            <div className="row" style={{ marginTop: "0.75rem" }}>
              <select
                value={targetCollection}
                onChange={(e) => setTargetCollection(e.target.value)}
                disabled={!collections.length}
              >
                {collections.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name} ({c.app_ids.length})
                  </option>
                ))}
              </select>
              <button
                disabled={busy || !targetCollection || selectedGames.length === 0}
                onClick={() =>
                  run("add_games_to_collection", async () => {
                    setPending(
                      await invoke<CollectionChangeset>("add_games_to_collection", {
                        collectionId: targetCollection,
                        appIds: selectedGames,
                      }),
                    );
                    await refreshLists();
                  })
                }
              >
                {tr("addSelected")}
              </button>
              <button
                disabled={busy || !targetCollection || selectedGames.length === 0}
                onClick={() =>
                  run("remove_games_from_collection", async () => {
                    setPending(
                      await invoke<CollectionChangeset>(
                        "remove_games_from_collection",
                        {
                          collectionId: targetCollection,
                          appIds: selectedGames,
                        },
                      ),
                    );
                    await refreshLists();
                  })
                }
              >
                {tr("removeSelected")}
              </button>
            </div>
            <p className="muted">
              {tr("selectedGamesHint", { count: selectedGames.length })}
            </p>
          </section>

          <section className="panel split">
            <div>
              <h2>{tr("games")}</h2>
              <input
                value={gameQuery}
                placeholder={tr("searchPlaceholder")}
                onChange={(e) => setGameQuery(e.target.value)}
                style={{ marginBottom: "0.6rem", width: "100%" }}
              />
              <p className="muted">
                {tr("gamesShowing", {
                  shown: filteredGames.length,
                  total: games.length,
                })}
                {metaCoverage != null
                  ? tr("metaCount", { count: metaCoverage })
                  : ""}
              </p>
              <ul className="checklist">
                {filteredGames.map((g) => (
                  <li key={g.app_id}>
                    <label>
                      <input
                        type="checkbox"
                        checked={selectedGames.includes(g.app_id)}
                        onChange={() => toggleGame(g.app_id)}
                      />{" "}
                      {g.name} <span className="muted">({g.app_id})</span>
                    </label>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h2>{tr("collections")}</h2>
              <ul>
                {collections.map((c) => (
                  <li key={c.id}>
                    <button
                      className="linkish"
                      disabled={busy}
                      onClick={() => setTargetCollection(c.id)}
                    >
                      {c.name}
                    </button>{" "}
                    <span className="muted">
                      {tr("appsCount", { count: c.app_ids.length })}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          </section>

          <section className="panel">
            <h2>{tr("metadataAutocat")}</h2>
            <div className="row">
              <button
                disabled={busy}
                onClick={() =>
                  run("enrich_metadata", async () => {
                    const r = await invoke<EnrichReport>("enrich_metadata", {
                      maxFetch: 40,
                    });
                    setEnrichReport(
                      tr("enrichReport", {
                        fromCache: r.fromCache,
                        fetched: r.fetched,
                        failed: r.failed,
                        skippedBudget: r.skippedBudget,
                      }),
                    );
                    await refreshMetaCoverage();
                  })
                }
              >
                {tr("enrichMetadata")}
              </button>
              <button
                disabled={busy}
                onClick={() =>
                  run("run_autocat_genre", async () => {
                    setAutocat(await invoke<AutoCatSummary>("run_autocat_genre"));
                  })
                }
              >
                {tr("dryRunGenre")}
              </button>
              <button
                disabled={busy || !autocat}
                onClick={() =>
                  run("queue_last_autocat", async () => {
                    setPending(
                      await invoke<CollectionChangeset>("queue_last_autocat"),
                    );
                    await refreshLists();
                  })
                }
              >
                {tr("queuePending")}
              </button>
            </div>
            {enrichReport && <p className="muted">{enrichReport}</p>}
            {autocat && (
              <p className="muted">
                {tr("autocatReport", {
                  matched: autocat.matchedGames,
                  skipped: autocat.skippedMissingMeta,
                  creates: autocat.createCount,
                  addOps: autocat.addMemberOps,
                })}
              </p>
            )}
          </section>

          <section className="panel">
            <h2>{tr("pending")}</h2>
            {pending ? (
              <>
                <p>
                  {formatPendingSummary(pending, (key, params) =>
                    tr(key, params),
                  )}
                </p>
                <pre className="code">{JSON.stringify(pending, null, 2)}</pre>
              </>
            ) : (
              <p className="muted">{tr("noPending")}</p>
            )}
            <button
              disabled={busy || !pending || isPendingEmpty(pending)}
              onClick={() =>
                run("export_session", async () => {
                  const msg = await invoke<string>("export_session");
                  setExportMsg(msg);
                  setAutocat(null);
                  setSelectedGames([]);
                  await refreshLists();
                })
              }
            >
              {tr("export")}
            </button>
          </section>
        </>
      )}
    </main>
  );
}

export default App;
