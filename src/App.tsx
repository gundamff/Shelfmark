import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AboutDialog } from "./components/AboutDialog";
import { AppShell, type AppTab } from "./components/AppShell";
import {
  localizeError,
  parseLocalePreference,
  resolveLocale,
  t,
  type LocalePreference,
} from "./i18n";
import type { CollectionChangeset } from "./lib/pending";
import type { SteamCompatView } from "./lib/steamCompat";
import { CollectionsPage } from "./pages/CollectionsPage";
import {
  SettingsPage,
  type SessionSummary,
  type SteamInfo,
} from "./pages/SettingsPage";
import "./App.css";

type Game = { app_id: number; name: string };
type Collection = { id: string; name: string; app_ids: number[] };

type AutoCatSummary = {
  matchedGames: number;
  skippedMissingMeta: number;
  createCount: number;
  addMemberOps: number;
};

type AppConfig = {
  last_user_id3?: string | null;
  steam_path_override?: string | null;
  locale?: string | null;
  steam_web_api_key?: string | null;
};

function App() {
  const [tab, setTab] = useState<AppTab>("settings");
  const [aboutOpen, setAboutOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [steam, setSteam] = useState<SteamInfo | null>(null);
  const [compat, setCompat] = useState<SteamCompatView | null>(null);
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
  const [enrichRunning, setEnrichRunning] = useState(false);
  const [enrichDetail, setEnrichDetail] = useState<string | null>(null);
  const [localePref, setLocalePref] = useState<LocalePreference>("system");
  const [apiKey, setApiKey] = useState("");

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
        setError(localizeError(locale, `${label}: ${String(e)}`));
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

  const refreshMetaCoverage = useCallback(async () => {
    try {
      setMetaCoverage(await invoke<number>("metadata_coverage"));
    } catch {
      setMetaCoverage(null);
    }
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const cfg = await invoke<AppConfig>("get_config");
        if (cfg.last_user_id3) setSelectedUser(cfg.last_user_id3);
        setLocalePref(parseLocalePreference(cfg.locale));
        if (cfg.steam_web_api_key) setApiKey(cfg.steam_web_api_key);
      } catch {
        // ignore
      }
      try {
        const c = await invoke<{
          supportedLabel: string;
          supportedNote: string;
          detectedVersion: string | null;
          withinSupport: boolean | null;
        }>("steam_compat", { steamRoot: null });
        setCompat({
          supportedSteam: c.supportedLabel,
          supportedSteamNote: c.supportedNote,
          detectedSteamVersion: c.detectedVersion,
          steamWithinSupport: c.withinSupport,
        });
      } catch {
        // ignore
      }
    })();
  }, []);

  useEffect(() => {
    if (!enrichRunning) return;
    const id = window.setInterval(() => {
      void (async () => {
        try {
          const s = await invoke<{
            running: boolean;
            total: number;
            done: number;
            fromLocal: number;
            fromStore: number;
            failed: number;
            remaining: number;
            stopped: boolean;
          }>("enrich_status");
          setEnrichDetail(
            `${s.done}/${s.total} · local ${s.fromLocal} · store ${s.fromStore} · fail ${s.failed} · left ${s.remaining}`,
          );
          if (!s.running) {
            setEnrichRunning(false);
            await refreshMetaCoverage();
          }
        } catch {
          setEnrichRunning(false);
        }
      })();
    }, 500);
    return () => window.clearInterval(id);
  }, [enrichRunning, refreshMetaCoverage]);

  const sessionChip = session
    ? tr("sessionChip", {
        mode: session.mode,
        user: selectedUser || "—",
      })
    : null;

  return (
    <main className="app">
      <AppShell
        tab={tab}
        onTab={setTab}
        onAbout={() => setAboutOpen(true)}
        sessionChip={sessionChip}
        compat={compat}
        t={tr}
      >
        {error && <div className="banner error">{error}</div>}
        {exportMsg && <div className="banner ok">{exportMsg}</div>}

        {tab === "settings" && (
          <SettingsPage
            busy={busy}
            localePref={localePref}
            onLocaleChange={(value) => {
              const pref = parseLocalePreference(value);
              setLocalePref(pref);
              void run("save_locale", async () => {
                await invoke("save_locale", { locale: pref });
              });
            }}
            steam={steam}
            selectedUser={selectedUser}
            onSelectUser={setSelectedUser}
            session={session}
            gameCount={games.length}
            collectionCount={collections.length}
            metaCoverage={metaCoverage}
            enrichReport={enrichReport}
            apiKey={apiKey}
            onApiKeyChange={setApiKey}
            onDetectSteam={() =>
              run("detect_steam", async () => {
                const info = await invoke<SteamInfo>("detect_steam");
                setSteam(info);
                setCompat({
                  supportedSteam: info.supportedSteam,
                  supportedSteamNote: info.supportedSteamNote,
                  detectedSteamVersion: info.detectedSteamVersion,
                  steamWithinSupport: info.steamWithinSupport,
                });
                setSelectedUser((prev) => {
                  if (prev && info.users.includes(prev)) return prev;
                  return info.users[0] ?? "";
                });
              })
            }
            onOpenDemo={() =>
              run("open_demo_session", async () => {
                const s = await invoke<SessionSummary>("open_demo_session");
                setSession(s);
                setAutocat(null);
                setExportMsg(null);
                setSelectedGames([]);
                setEnrichReport(null);
                await refreshLists();
                await refreshMetaCoverage();
                setTab("collections");
              })
            }
            onOpenLive={() =>
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
                setTab("collections");
              })
            }
            onEnrich={() =>
              run("enrich_start", async () => {
                await invoke("enrich_start", { maxFetch: 40 });
                setEnrichRunning(true);
                setEnrichReport(null);
              })
            }
            onEnrichStop={() => {
              void invoke("enrich_stop");
            }}
            enrichRunning={enrichRunning}
            enrichDetail={enrichDetail}
            onSaveApiKey={() =>
              run("save_api_key", async () => {
                await invoke("save_api_key", { apiKey: apiKey.trim() });
                setExportMsg(tr("apiKeySaved"));
              })
            }
            t={tr}
          />
        )}

        {tab === "collections" && (
          <CollectionsPage
            busy={busy}
            hasSession={!!session}
            steamRunning={!!steam?.steamRunning}
            games={games}
            collections={collections}
            pending={pending}
            autocat={autocat}
            selectedGames={selectedGames}
            targetCollection={targetCollection}
            newCollectionName={newCollectionName}
            gameQuery={gameQuery}
            metaCoverage={metaCoverage}
            onGameQuery={setGameQuery}
            onToggleGame={(appId) =>
              setSelectedGames((prev) =>
                prev.includes(appId)
                  ? prev.filter((id) => id !== appId)
                  : [...prev, appId],
              )
            }
            onNewName={setNewCollectionName}
            onTargetCollection={setTargetCollection}
            onCreate={() =>
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
            onAdd={() =>
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
            onRemove={() =>
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
            onAutocat={() =>
              run("run_autocat_genre", async () => {
                setAutocat(await invoke<AutoCatSummary>("run_autocat_genre"));
              })
            }
            onQueueAutocat={() =>
              run("queue_last_autocat", async () => {
                setPending(
                  await invoke<CollectionChangeset>("queue_last_autocat"),
                );
                await refreshLists();
              })
            }
            onExport={() =>
              run("export_session", async () => {
                const msg = await invoke<string>("export_session");
                setExportMsg(msg);
                setAutocat(null);
                setSelectedGames([]);
                await refreshLists();
              })
            }
            t={tr}
          />
        )}
      </AppShell>

      <AboutDialog
        open={aboutOpen}
        onClose={() => setAboutOpen(false)}
        compat={compat}
        t={tr}
      />
    </main>
  );
}

export default App;
