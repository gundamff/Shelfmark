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


##  

- 262144  0.8 tok/s������
- �� Steam API ��
- dialog ��dialog �������� disabled ��
