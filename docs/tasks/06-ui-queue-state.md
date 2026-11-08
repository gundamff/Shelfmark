# Task 06 — AutoCat "加入待导出" does nothing from the wizard (frontend state bug)

Repo: D:\gits\Depressurizer. Scope: frontend only (`src/App.tsx`, `src/pages/CollectionsPage.tsx`,
`src/components/AutocatWizard.tsx`). Backend is correct — do not change Rust.

## Measured on the real app (live session, 762 games, 120 with metadata)

Backend is healthy. Calling the Tauri commands directly returns correct data:

```
pending_before : create 0,  add_members 18
queue_last_autocat : create 18, add_members 18
pending_after  : create 18, add_members 18
```

But the UI 待导出 panel keeps showing:

```
{"create":0,"rename":0,"delete":0,"add_members":0,"remove_members":0}
```

## Two concrete frontend defects

1. `src/pages/CollectionsPage.tsx:198-204` — the 加入待导出 button is
   `disabled={busy || !autocat}`. When the plan comes from the **wizard** (`AutocatWizard.tsx`),
   the page-level `autocat` summary is never set, so the button stays disabled and the click is a no-op.
2. Even when the queue succeeds, the 待导出 panel does not reflect it. After
   `invoke("queue_last_autocat")` the returned changeset must become the rendered `pending` state, and it
   must not be overwritten by a stale snapshot.

## Required

1. Single source of truth for the AutoCat summary: when the wizard finishes a preview, publish the summary to
   the app state (callback/prop), so 加入待导出 becomes enabled without a second dry run.
2. After queueing, render exactly the changeset returned by `queue_last_autocat`. If a refresh runs
   afterwards, it must read `pending_changeset` and keep the same numbers.
3. The wizard's own 加入待导出 button must behave identically (same state, same numbers).
4. Remove the possibility of the two numbers diverging: preview panel and 待导出 panel must be fed from the
   same state object.

## Acceptance
- `npm run build` green
- `cargo test -p steam-core` still green (untouched)
- one commit: `fix(ui): surface AutoCat plan and queued changeset in app state`
- Do not push. Do not touch src-tauri except if a command signature is genuinely missing (it is not).
