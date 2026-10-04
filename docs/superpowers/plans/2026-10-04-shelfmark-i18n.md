# Shelfmark i18n Implementation Plan

> **For agentic workers:** executing-plans / TDD per task.

**Goal:** zh/en UI + Settings language menu + system locale detect + common error localization.

**Architecture:** Pure `src/i18n/` dictionaries and helpers; persist `AppConfig.locale`; wire `App.tsx` Settings panel.

**Tech Stack:** React/TS, Vitest, existing Tauri `AppConfig`, no new deps.

## Global Constraints

- No i18next; scope B only (UI + known errors).
- Do not translate game names / paths / raw JSON.

## Tasks

- [x] Task 1: `resolveLocale` + `localizeError` + dict key tests (TDD)
- [x] Task 2: en/zh message dictionaries + `t()`
- [x] Task 3: `AppConfig.locale` + `save_locale` command
- [x] Task 4: Settings panel + wire App strings through `t()`
- [x] Task 5: verify (`npm test`, `cargo test -p steam-core`, `cargo check -p shelfmark`) + merge
