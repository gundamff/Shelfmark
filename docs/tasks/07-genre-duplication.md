# Task 07 — 按游戏类型 AutoCat 产生中英重复收藏集

## 现象（2026-11-08 实机导出发现）

真实库（762 App）跑 AutoCat，规则「按游戏类型」，预览 `匹配 120 / 跳过 642 / 新建 18 / 添加操作 18`，
导出后 Steam 云存储里出现 18 个 `g_` 收藏集，但同一类型被拆成中英两份：

| 收藏集 | 成员 |
|---|---|
| `g_Action` | 81 |
| `g_动作` | 1（War Thunder 236390） |
| `g_Simulation` | 17 |
| `g_模拟` | 1（同上） |
| `g_Free To Play` | 12 |
| `g_免费开玩` | 1（同上） |
| `g_RPG` | 17 |
| `g_角色扮演` | 1（The Witcher 3） |
| `g_Massively Multiplayer` | —（缺失，被 `g_大型多人在线` 顶替，仅 1 成员） |

后果：类型收藏集数量翻倍、每个中文副本只有 1 个成员，用户看到一堆残缺收藏集。

## 根因

`crates/steam-core/src/autocat/engine.rs` 的 Genre 规则直接用元数据里的原始字符串拼名字：

```rust
let col_name = format!("{prefix}{genre}");
```

而 `meta-cache.json` 里 genre 中英混杂——Steam 商店 API 返回英文（`Action`），
早年抓取/本地化的缓存是中文（`动作`）。同一个语义两个字符串 → 两个收藏集。

## 修法

新增 `canonical_genre(raw) -> String`：把大小写、空白、常见中英写法归一到 Steam 英文标准名，
未收录的取值原样返回（不丢数据）。Genre 规则改为 `format!("{prefix}{}", canonical_genre(genre))`。

已覆盖：Action/Adventure/Casual/Indie/Massively Multiplayer/Racing/RPG/Simulation/Sports/Strategy/
Early Access/Free To Play/Animation & Modeling/Audio Production/Design & Illustration/Education/
Game Development/Photo Editing/Software Training/Utilities/Video Production/Web Publishing/
Sexual Content/Gore，含对应中文写法。

## 测试

- `canonical_genre_folds_localised_spellings`：中英归一 + 未收录保留 + 空白处理
- `genre_rule_merges_duplicate_localisations`：两个游戏分别用中文/英文 genre，断言只产生 3 个收藏集且每个 2 成员

`cargo test -p steam-core` → **54 passed, 0 failed**（原 52 + 新增 2）

## 遗留清理

已经写进 Steam 的 5 个中文残缺收藏集（`g_动作`、`g_模拟`、`g_免费开玩`、`g_角色扮演`、`g_大型多人在线`）
需要清掉。用「覆盖策略 = 替换成员」（`remove_existing = true`）再跑一次导出即可，
该策略会先删掉所有同前缀的旧收藏集再重建。

## 待办

- [x] 归一化映射 + 单测
- [ ] 重新构建 debug 包，实机重跑导出（覆盖策略=替换成员），确认只剩英文集合
- [ ] 元数据拉全量后复验匹配数（现在只有 120/762 有 genre，中文残缺样本占比大）
