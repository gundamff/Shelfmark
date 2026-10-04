import { useCallback, useState } from "react";
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
  }, []);

  return (
    <main className="app">
      <header className="hero">
        <h1>Shelfmark</h1>
        <p className="tagline">
          Windows Steam library organizer — demo session writes AppData only, not
          live Steam LevelDB yet.
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
              if (info.users.length > 0) {
                setSelectedUser((prev) => prev || info.users[0]);
              }
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
                await refreshLists();
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
                const s = await invoke<SessionSummary>("open_live_session", {
                  userId3: selectedUser,
                });
                setSession(s);
                setAutocat(null);
                setExportMsg(null);
                await refreshLists();
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
              {session.gameCount} / {session.collectionCount}
            </dd>
            <dt>Store</dt>
            <dd className="path">{session.storePath}</dd>
          </dl>
        )}
      </section>

      {session && (
        <>
          <section className="panel split">
            <div>
              <h2>Games</h2>
              <ul>
                {games.map((g) => (
                  <li key={g.app_id}>
                    {g.name} <span className="muted">({g.app_id})</span>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h2>Collections</h2>
              <ul>
                {collections.map((c) => (
                  <li key={c.id}>
                    {c.name}{" "}
                    <span className="muted">{c.app_ids.length} apps</span>
                  </li>
                ))}
              </ul>
            </div>
          </section>

          <section className="panel">
            <h2>AutoCat (Genre)</h2>
            <div className="row">
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
                    setPending(await invoke<CollectionChangeset>("queue_last_autocat"));
                  })
                }
              >
                Queue into pending
              </button>
            </div>
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
