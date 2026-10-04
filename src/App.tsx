import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  formatPendingSummary,
  isPendingEmpty,
  type CollectionChangeset,
} from "./lib/pending";
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

  const run = useCallback(async (label: string, fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(`${label}: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  }, []);

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

  return (
    <main className="app">
      <header className="hero">
        <h1>Shelfmark</h1>
        <p className="tagline">
          Organize Steam collections. Demo writes AppData; live writes your Steam
          cloud-storage catalog after you exit Steam.
        </p>
      </header>

      {error && <div className="banner error">{error}</div>}
      {exportMsg && <div className="banner ok">{exportMsg}</div>}

      <section className="panel">
        <h2>Steam</h2>
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
          Detect Steam
        </button>
        {steam && (
          <dl className="kv">
            <dt>Root</dt>
            <dd>{steam.root}</dd>
            <dt>Users</dt>
            <dd>{steam.users.join(", ") || "(none)"}</dd>
            <dt>Running</dt>
            <dd>{steam.steamRunning ? "yes — export will refuse" : "no"}</dd>
          </dl>
        )}
      </section>

      <section className="panel">
        <h2>Session</h2>
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
            Open demo session
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
            Open live session
          </button>
        </div>
        {session && (
          <dl className="kv">
            <dt>Mode</dt>
            <dd>{session.mode}</dd>
            <dt>Games / collections</dt>
            <dd>
              {games.length} / {collections.length}
            </dd>
            <dt>Store</dt>
            <dd className="path">{session.storePath}</dd>
          </dl>
        )}
      </section>

      {session && (
        <>
          <section className="panel">
            <h2>Manual edit</h2>
            <div className="row">
              <input
                value={newCollectionName}
                placeholder="New collection name"
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
                Create collection
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
                Add selected
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
                Remove selected
              </button>
            </div>
            <p className="muted">
              Selected {selectedGames.length} game(s). Edits go to pending until
              export.
            </p>
          </section>

          <section className="panel split">
            <div>
              <h2>Games</h2>
              <input
                value={gameQuery}
                placeholder="Search name or app id"
                onChange={(e) => setGameQuery(e.target.value)}
                style={{ marginBottom: "0.6rem", width: "100%" }}
              />
              <p className="muted">
                Showing {filteredGames.length} / {games.length}
                {metaCoverage != null ? ` · meta ${metaCoverage}` : ""}
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
              <h2>Collections</h2>
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
                    <span className="muted">{c.app_ids.length} apps</span>
                  </li>
                ))}
              </ul>
            </div>
          </section>

          <section className="panel">
            <h2>Metadata + AutoCat</h2>
            <div className="row">
              <button
                disabled={busy}
                onClick={() =>
                  run("enrich_metadata", async () => {
                    const r = await invoke<EnrichReport>("enrich_metadata", {
                      maxFetch: 40,
                    });
                    setEnrichReport(
                      `cache ${r.fromCache}, fetched ${r.fetched}, failed ${r.failed}, budget-skip ${r.skippedBudget}`,
                    );
                    await refreshMetaCoverage();
                  })
                }
              >
                Enrich metadata (40/batch)
              </button>
              <button
                disabled={busy}
                onClick={() =>
                  run("run_autocat_genre", async () => {
                    setAutocat(await invoke<AutoCatSummary>("run_autocat_genre"));
                  })
                }
              >
                Dry-run Genre
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
                Queue into pending
              </button>
            </div>
            {enrichReport && <p className="muted">Enrich: {enrichReport}</p>}
            {autocat && (
              <p className="muted">
                matched {autocat.matchedGames}, skipped {autocat.skippedMissingMeta},
                creates {autocat.createCount}, add-ops {autocat.addMemberOps}
              </p>
            )}
          </section>

          <section className="panel">
            <h2>Pending</h2>
            {pending ? (
              <>
                <p>{formatPendingSummary(pending)}</p>
                <pre className="code">{JSON.stringify(pending, null, 2)}</pre>
              </>
            ) : (
              <p className="muted">No pending loaded</p>
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
              Export (refuses if Steam running)
            </button>
          </section>
        </>
      )}
    </main>
  );
}

export default App;
