# Shelfmark

Windows-first Steam library organizer (spiritual successor to Depressurizer).  
**MIT** greenfield — do not copy GPL code from upstream Depressurizer.

## Status

- `steam-core`: platform detect (+ Steam.exe FileVersion), collections, backup, safe export, AutoCat, **JSON rule packs**
- Desktop UI: Tauri 2 + React — **Settings | Collections** shell, About dialog, Steam support line
  - **Demo / live** sessions; export refuses while Steam running
  - Metadata enrich with progress (Store API; local appinfo parser stubbed)
  - AutoCat wizard loads `rule-packs/*.json` (+ `%AppData%/Shelfmark/rule-packs/`)
  - Steam Web API key in config for achievement probes (full achievement AutoCat packs WIP)
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
