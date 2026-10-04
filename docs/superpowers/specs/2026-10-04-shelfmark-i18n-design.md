# Shelfmark i18n (zh/en) Design

**Status:** approved  
**Scope:** Frontend UI strings + common user-visible error mapping (option B). Game titles, paths, and raw pending JSON stay untranslated.

## Goals

- Support **English** and **Simplified Chinese**.
- **Settings** panel to choose language: Follow system / English / 中文.
- Default **Follow system**: `navigator.language` starting with `zh` → Chinese, else English.
- Persist preference in existing `%AppData%/Shelfmark/config.json` as `locale`.

## Non-goals

- Full backend error catalog translation.
- i18next or other heavy libraries.
- Translating Steam game names or store API content.

## Architecture

```
AppConfig.locale: "system" | "en" | "zh" | null(=system)
        │
        ▼
resolveLocale(pref, navigator.language) → "en" | "zh"
        │
        ▼
dictionaries (en.ts / zh.ts) + t(key, params?)
        │
        ├── UI labels / buttons / settings
        └── localizeError(raw) for known invoke failures
```

- Keep `steam-core` free of UI strings; only add optional `locale` field on `AppConfig`.
- Tauri: `save_locale` command (load-modify-save pattern like `save_last_user`).
- Frontend: `src/i18n/` with pure functions (unit-tested) and a small React context or props for `t` / `locale` / `setLocale`.

## Error mapping (B)

Match substrings / known English fragments from Rust, e.g.:

- `no session open`
- `Steam is running` / export refuse
- `Steam catalog not found`
- `run AutoCat first`
- Windows-only session/export messages

Unmatched: show original text (optionally prefixed with localized "Error").

## Settings UI

- Collapsible **Settings** panel near the top (after hero).
- Language `<select>` with three options; change applies immediately and persists.

## Testing

- Unit tests: `resolveLocale`, `localizeError`, dictionary key parity (en/zh same keys).
- Existing `pending` summary tests updated to accept translator or stay locale-agnostic counts.
