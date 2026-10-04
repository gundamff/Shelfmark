# Shelfmark Manual Collection Edit UI Plan

> **For agentic workers:** executing-plans / subagent-driven-development.

**Goal:** Let users create collections and add/remove games via the UI, queuing a `CollectionChangeset` for export (demo + live). Persist last selected Steam user id in `%AppData%/Shelfmark/config.json`.

**Architecture:** Thin Tauri commands mutate `Session.pending` / in-memory collections view; no disk write until export. New `config` module in `steam-core` for JSON settings.

## Tasks

- [x] Task 1: `steam-core` config load/save (`last_user_id3`, optional steam path override)
- [x] Task 2: Commands: `create_collection`, `add_games_to_collection`, `remove_games_from_collection`, `get_config`/`save_config`
- [x] Task 3: React UI for selection + edits; remember user
- [x] Task 4: tests + merge
