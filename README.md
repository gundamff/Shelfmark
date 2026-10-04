# Shelfmark

Windows-first Steam library organizer (spiritual successor to Depressurizer).  
**MIT** greenfield — do not copy GPL code from upstream Depressurizer.

## Status

- `steam-core`: platform detect, collections (JSON/LevelDB stores), backup, safe export, AutoCat dry-run
- Desktop UI: Tauri 2 + React
  - **Demo session**: fixtures → `%AppData%/Shelfmark/demo`
  - **Live session**: reads/writes `{Steam}/userdata/<id>/config/cloudstorage/cloud-storage-namespace-1.json`
  - Export **refuses while Steam is running**; backups under `%AppData%/Shelfmark/backups`
  - Live AutoCat metadata enrichment still TBD (Genre dry-run will mostly skip)

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
