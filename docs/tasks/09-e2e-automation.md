# Task 09 — 端到端验收自动化（Shelfmark）

## 背景

2026-11-08/09 的实机验收是靠人工 + 临时 Playwright 脚本（`D:\bots\scripts\shelf_e2e_step*.py`）跑的，
每次都要手动点预览、点加入待导出、点导出，而且踩过两个坑：

1. dialog 内外的同名按钮（`加入待导出`）会点错，导致点的是页面背景那个 disabled 的
2. 模型设置里上下文开到 262144 会让速度掉到 0.8 tok/s（与代码无关，但验收时要写进文档）

## 要做

1. 在仓库内建一个可复跑的验收脚本 `scripts/e2e-autocat.ps1`（纯 ASCII，PowerShell 5.1 兼容）：
   - 用 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` 启动 debug 包
   - 通过 CDP 驱动界面：检测 Steam → 打开实时会话 → AutoCat(规则 genre) → Preview → 加入待导出
   - 断言：预览里的「新建 N / 添加操作 M」与待导出里的「N 创建 / M 添加」完全一致
   - 不执行导出（写盘步骤需要人工确认 Steam 已退出）
   - 输出一份 `logs/e2e-autocat-<时间戳>.json` 记录各项计数
2. 补一条 Rust 集成测试：`remove_existing = true` 时，已存在的同前缀集合（含本地化命名的残留）会被删除后重建，
   且重建后不产生重复集合。现有 `genre_rule_merges_duplicate_localisations` 只覆盖了合并，没覆盖删除。
3. 在 `docs/tasks/09-e2e-automation.md` 末尾补一节「已知环境坑」：上下文长度、并行数、dialog 作用域。

## 约束

- `cargo test` 与 `npm run build` 必须绿
- 只提交一次，不要 push
- 不要改 `docs/feedback/`、`docs/tasks/07-*`、`docs/tasks/08-*`

## 已知环境坑

- **上下文长度**：模型设置里上下文开到 262144 后，生成速度掉到 0.8 tok/s。与代码无关，跑验收前先改回较小值，否则容易误判成程序卡死。
- **并行数**：模型设置里的并行请求数开高会让多个请求同时排队，同样拖慢验收；跑验收前改回默认值。
- **Steam 必须完全退出**：`export` 检测到 Steam 进程在运行会直接拒绝写盘（见 `tests/export_safety.rs::export_refuses_when_steam_running`）。本脚本只跑到「加入待导出」，写盘步骤留给人工确认。
- **dialog 作用域**：`加入待导出` 在 dialog 内和页面背景各有一个同名按钮，背景那个是 disabled 的。驱动脚本必须先限定到 dialog 作用域再点，否则会点到背景那个并静默失败。

## 2026-11-09 实跑记录

初版脚本首次实跑直接失败，暴露四个问题，已全部修正：

1. **`cargo tauri dev` 不存在** —— 本仓库的 tauri CLI 来自 npm（`node_modules/.bin/tauri`），正确命令是 `npm run tauri dev`。原脚本 `Start-Process cargo` 立即退出，随后 90 秒全在空等 CDP。
2. **按钮文案写成了英文** —— 界面默认跟随系统，实际是中文：「检测 Steam」「打开实时会话」「试运行 Genre」「加入待导出」。`Click-ByText` 找不到按钮时静默返回 false，流程照样往下走。
3. **CDP 收发没做 id 配对，计数也没落盘** —— 原脚本 `$log` 里四个计数字段硬编码为 `$null`，即使流程跑通也永远是假绿。
4. **无断言、无进程清理** —— 跑完留下 npm / vite / shelfmark 进程占用 1420 与 9222。

修正要点：按钮名用 `\uXXXX` 转义（保证 ps1 纯 ASCII，PS 5.1 读无 BOM 的 UTF-8 会把 CJK 字面量按 ANSI 解坏）；CDP 响应按 id 配对并累积分片；计数写入 JSON 并做一致性断言；`finally` 里清理进程。

修正后实跑结果（762 款游戏，元数据 120，debug 已构建）：

| 项 | 值 |
|---|---|
| 预览 | 匹配 120 / 跳过 642 / 新建 1 / 添加操作 5 |
| 待导出 | 1 创建 · 0 重命名 · 0 删除 · 5 添加 · 0 移除 |
| 断言 | 通过（create 1=1，add 5=5，delete=0） |
| 耗时 | 28 秒 |

导出按设计未执行。
