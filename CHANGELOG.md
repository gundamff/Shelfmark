# Changelog

本项目遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## v0.1.0 — 2026-11-09

首个公开版本。

### 核心能力

- **Steam 库检测**：自动定位安装路径、用户目录与 `cloud-storage-namespace-1.json`，并读取 `steam.exe` FileVersion 给出支持性提示
- **收藏集读写**：直接读写 Steam 云存储命名空间，保留动态 `filterSpec` 条目不动
- **演示 / 实时会话**：演示会话用内置样本，实时会话读取真实库；实时会话仅 Windows
- **安全导出**：Steam 运行中拒绝导出；写入前自动备份原文件
- **元数据补全**：Steam Store API + 本地 `appinfo.vdf` 解析，带进度、可分批发
- **AutoCat 规则包**：genre、tags、year、flags、devpub、user score、achievements 七类 JSON 规则包，支持从 `%AppData%/Shelfmark/rule-packs/` 加载自定义包
- **预览与待导出队列**：先试运行看计数，再入队，最后一次性导出；`remove_existing` 可清理旧分类重算
- **成就分类**：设置 Steam Web API Key 后可拉取成就完成度并按 0% / 100% / 部分 归档，单次上限 40 个应用，比例缓存仅当前会话有效

### 已知限制

- 仅支持 Windows 版 Steam 布局
- 动态筛选型收藏集（`filterSpec` 非空）不会被改写
- 本机 Steam 客户端 11.15.70.27 超出文档化的 10.x 支持范围，实测可用但请自行备份

### 开发

- `cargo test --workspace` 全绿（66 个用例）
- `scripts/e2e-autocat.ps1` 端到端回归：启动应用、连 CDP、跑 AutoCat 预览、校验待导出计数一致性，全程不执行导出
