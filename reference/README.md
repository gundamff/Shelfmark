# Reference sources (read-only)

## Depressurizer upstream

Local clone (not committed; GPL-3.0):

```powershell
git clone --depth 1 https://github.com/julianxhokaxhiu/Depressurizer.git reference/depressurizer
```

**Allowed:** read for Steam path layout, collection key shapes, AutoCat *behavior*, test ideas.  
**Forbidden:** copy C# (or other) source into Shelfmark. Shelfmark is MIT greenfield; clean-room reimplementation only.

### High-value entry points

| Topic | Path under `reference/depressurizer/` |
|-------|----------------------------------------|
| Path constants | `src/Depressurizer.Core/Helpers/Constants.cs` |
| Collections LevelDB | `src/Depressurizer.Core/Models/SteamLevelDB.cs` |
| Collections JSON | `src/Depressurizer.Core/Models/SteamJsonDB.cs` |
| Merge / write collections | `src/Depressurizer.Core/Helpers/SteamJsonCollectionHelper.cs` |
| Import library + collections | `src/Depressurizer.Core/Models/GameList.cs` |
| AutoCat types | `src/Depressurizer/AutoCats/`, `src/Depressurizer.Core/AutoCats/` |
| Collection save interface | `src/Depressurizer.Core/Interfaces/ISteamCollectionSaveManager.cs` |

### Collection storage notes (as of upstream)

- LevelDB dir (Windows): `%LocalAppData%\Steam\htmlcache\Local Storage\leveldb`
- Key prefix pattern: `_https://steamloopback.host` + NUL + SOH + `U{steamID3}-cloud-storage-namespace-1`
- Keys of interest: `user-collections.*` (plus `hidden` / `favorite`)
- JSON fallback: `{Steam}\userdata\{id3}\config\cloudstorage\cloud-storage-namespace-1.json`
