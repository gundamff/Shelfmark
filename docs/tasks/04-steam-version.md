# Task 04 — Steam version probe shows "尚未检测版本"

Repo: D:\gits\Depressurizer. Scope: version detection only.

## Symptom
The banner always shows `尚未检测版本 — 请先「检测 Steam」` even though Steam is installed at
`C:\Program Files (x86)\Steam\Steam.exe` and has a FileVersion.

## Required
1. Reproduce the probe on this machine (Steam at `C:\Program Files (x86)\Steam`).
2. Fix it, or if it cannot be read, make the banner state the real reason (not detected vs unsupported),
   including the path that was checked.
3. Add a unit test for the parsing step (fixture string, no real Steam dependency).

## Acceptance
- `cargo test -p steam-core` green
- one commit: `fix(steam): report version detection state accurately`
- Do not push. Do not touch src-tauri/tauri.conf.json.
