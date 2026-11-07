# Task 05 — appinfo.vdf v41 parsing returns 0 names on the real file

Repo: D:\gits\Depressurizer. Scope: `crates/steam-core/src/library/appinfo.rs` only.

## Measured failure (real machine, not synthetic)
Real file: `C:\Program Files (x86)\Steam\appcache\appinfo.vdf`
- size: 5,155,805 bytes
- magic: 0x07564429 (MAGIC_V41)
- bytes 4..8: 0x00000001 (universe)
- bytes 8..16: u64 = 5,091,396 -> string table offset (file tail after it is 64,409 bytes of NUL-separated strings, so this offset is plausible)
- entries start at offset 16

Calling `load_names_from_appinfo(root, &[])` returns **0 entries**, and with an explicit want-list it also returns 0.
The current V41 branch therefore fails on the real file and the error is swallowed by
`parse_appinfo_metadata(&bytes, want).unwrap_or_default()`.

## First-entry dump at offset 16 (little endian)
```
appid      = 5
size       = 85
infoState  = 1
updated    = 1717623589
picsToken  = 0
changeNum  = 1732508295
```
The current code advances with `offset = payload_off + (size - 60)` and also assumes a 60-byte header,
which does not match this file: advancing 4 + size lands on garbage (next u32 reads 0x08080000).

## Required
1. Implement the actual v41 entry layout so the real file parses. Determine the correct stride and field
   offsets from the file itself (scan for known app ids such as 730, 570, 2288340 and verify the stride is
   consistent across the whole file).
2. Names must come from the string table at `string_table_offset`: each entry's binary VDF payload has
   `appinfo.common.name` stored as an offset into that table.
3. **Do not swallow parse errors silently.** If parsing fails on a file larger than 1 MB, return the error
   so the caller can show "names unavailable" instead of silently rendering `App <id>` for every app.
4. Keep the existing v40 path and the existing synthetic tests working.

## Verification you must do yourself
Add a temporary probe (delete it before committing) that calls `load_names_from_appinfo` on the real file
and prints the number of names plus a few samples. It must yield several thousand names and include
`730 => Counter-Strike 2` or similar. Paste the probe output in your final message.

## Acceptance
- `cargo test -p steam-core` green
- one commit: `fix(appinfo): parse v41 string table so real appinfo yields names`
- Do not push. Do not touch src-tauri/tauri.conf.json. Do not leave the temporary probe file in the tree.
