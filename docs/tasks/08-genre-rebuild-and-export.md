# Task 08 — Genre 集合重建（清残留）与导出点击无响应

两件事，都在 Shelfmark 仓库（本目录）。**只改这两处，不要顺手重构别的东西。**

## 背景

2026-11-08 实机验收（真实 762 App 库，Steam 用户 136330193）暴露两个问题。

### 问题 1：中文重复收藏集清不掉

`crates/steam-core/src/autocat/engine.rs` 的 Genre 规则已加 `canonical_genre()` 归一化（commit `8014dbc`），
新跑出来的名字只有英文。但 Steam 里已经写进去的 5 个中文残留还在：

`g_动作`、`g_模拟`、`g_免费开玩`、`g_角色扮演`、`g_大型多人在线`

想清掉它们，唯一途径是 Genre 规则的 `remove_existing = true`（先删所有同前缀集合再重建）。
但这个字段只来自规则包 JSON，而 `rule-packs/genre.json` 里没有这个字段，默认 false：

```json
"rules": [{ "kind": "genre", "prefix": "g_", "max_categories": 0 }]
```

界面的「覆盖策略：替换成员」是 `ConflictPolicy`，只影响**成员合并**，不删集合，两回事。

**要做的：**

1. `rule-packs/genre.json` 的 genre 规则加 `"remove_existing": true`
2. 确认 `canonical_genre` 归一后重建时不产生新的中文集合（已有单测 `genre_rule_merges_duplicate_localisations`，可补一条覆盖 remove_existing 的）
3. 补一个测试：已有 `g_动作` + `g_Action` 两个集合，`remove_existing = true` 时，两个都被删除、只重建一个 `g_Action`

### 问题 2：点「导出」没有任何反应

现象（cdp 实测）：

- 按钮 `导出（Steam 运行中会拒绝）`，`disabled = false`
- 点击后：队列不变（`1 创建 · 0 重命名 · 0 删除 · 5 添加 · 0 移除` 原样）、
  Steam 云存储文件 mtime 不变、页面无任何提示文案、控制台无报错
- 另一次（19:00）同样操作是成功的，写入了 18 个集合

代码路径：`src/App.tsx` 的 `onExport={() => run("export_session", async () => { const msg = await invoke<string>("export_session"); ... })}`，
按钮在 `src/pages/CollectionsPage.tsx`。

**要做的：**

1. 查清点击后 `export_session` 到底有没有被调用（怀疑点：`run()` 内部提前 return、按钮 onClick 未绑定、
   或 `pending` 为空判断用错了字段）
2. 修掉；无论成功失败都要有可见反馈（成功提示 + 失败原因，不能静默）
3. 不要改动后端写盘逻辑的正确性判断（Steam 运行中必须仍然拒绝导出）

## 约束

- `cargo test` 与 `npm run build` 必须都绿
- 只提交一次，commit message 用 `fix(autocat): rebuild genre collections and surface export result` 或更贴切的
- **不要 push**
- 不要改 `docs/feedback/`、`docs/tasks/07-*.md`、`docs/tasks/2026-11-08-e2e-verification.md`
- 不要动 `src-tauri/` 里与导出无关的部分
