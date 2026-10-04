# Shelfmark

Windows-first Steam library organizer (spiritual successor to Depressurizer).  
**MIT** greenfield — do not copy GPL code from upstream Depressurizer.

## Status

`steam-core` is the pure Rust business library (no UI yet).

## Develop

```powershell
cd D:\gits\Depressurizer\.worktrees\shelfmark-core-collections
$env:HTTP_PROXY = "http://127.0.0.1:7890"
$env:HTTPS_PROXY = "http://127.0.0.1:7890"
cargo test -p steam-core
```

Windows Steam is detected via registry, then `C:\Program Files (x86)\Steam`.

### Manual safety checklist (live write — later UI plan)

1. Import / load is read-only in unit tests (fixtures).
2. Before any real export: **fully exit Steam**.
3. Export creates a timestamped backup under the app backup dir, then writes.
4. If Steam is running, export must hard-refuse.

## Reference

Read-only upstream clone for format research:

```powershell
git clone --depth 1 https://github.com/julianxhokaxhiu/Depressurizer.git reference/depressurizer
```

See `reference/README.md`. GPL sources must not be copied into this tree.
