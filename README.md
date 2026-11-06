# Shelfmark

Windows-first Steam library organizer (spiritual successor to Depressurizer).  
**MIT** greenfield — do not copy GPL code from upstream Depressurizer.

## Status

- `steam-core`: platform detect (+ Steam.exe FileVersion), collections (JSON store + Steam `cloud-storage-namespace-1` catalog read/write, dynamic `filterSpec` entries skipped), backup, safe export, AutoCat with conflict policies, **JSON rule packs** (genre, tags, year, flags, devpub, user score, achievements)
- Desktop UI: Tauri 2 + React — **Settings | Collections** shell, About dialog, Steam support line
  - **Demo / live** sessions; live open is Windows-only, export refuses while Steam is running and backs up before writing
  - Live loader unions `libraryfolders.vdf` + `appmanifest_*.acf` with collection ids and a heuristic `localconfig.vdf` app scan
  - Metadata enrich with progress (Steam Store API + local `appinfo.vdf` parser)
  - AutoCat wizard loads `rule-packs/*.json` (+ `%AppData%/Shelfmark/rule-packs/`), previews and queues changes
  - Achievement AutoCat packs (`achievements-0/100/partial`) are available after setting a Steam Web API key and fetching ratios in the wizard; each fetch is capped (40 apps by default) and the ratio cache lasts only for the current session
- Supported Steam layout is shown in the UI (Windows · `cloud-storage-namespace-1.json`, FileVersion 10.x)

## Develop

```powershell
cd D:\gits\Depressurizer
$env:HTTP_PROXY = "http://127.0.0.1:7890"
$env:HTTPS_PROXY = "http://127.0.0.1:7890"

cargo test -p steam-core
npm install
npm test
npm run build
npm run tauri dev
```

## Reference

```powershell
git clone --depth 1 https://github.com/julianxhokaxhiu/Depressurizer.git reference/depressurizer
```

See `reference/README.md`.
