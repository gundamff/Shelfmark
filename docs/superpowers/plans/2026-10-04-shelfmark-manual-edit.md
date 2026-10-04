# Shelfmark Manual Collection Edit UI Plan

> **For agentic workers:** executing-plans / subagent-driven-development.

**Goal:** Let users create collections and add/remove games via the UI, queuing a `CollectionChangeset` for export (demo + live). Persist last selected Steam user id in `%AppData%/Shelfmark/config.json`.

**Architecture:** Thin Tauri commands mutate `Session.pending` / in-memory collections view; no disk write until export. New `config` module in `steam-core` for JSON settings.

## Tasks

- [ ] Task 1: `steam-core` config load/save (`last_user_id3`, optional steam path override)
- [ ] Task 2: Commands: `create_collection`, `add_games_to_collection`, `remove_games_from_collection`, `get_config`/`save_config`
- [ ] Task 3: React UI for selection + edits; remember user
- [ ] Task 4: tests + merge
