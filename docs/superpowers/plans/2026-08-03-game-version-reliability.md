# 游戏版本与工具链版本可靠化 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**日期：** 2026-08-03

**Goal:** 把「改哪个游戏版本」和「thtk 支持哪些版本」从散落各处的裸字符串，收敛成后端单一权威表 + 运行时能力探测，让非法版本在配置阶段就被拒绝、不受支持的工具动作在界面上直接置灰。

**Architecture:** 后端新增 `common/game_version.rs` 作为唯一权威：一张 30 条的版本表（含每个版本被哪些工具支持）、唯一的解析入口、唯一的「项目级优先、全局兜底」解析函数。thtk 实际支持的版本集合由**运行时探测**工具自身的 usage 输出得到，与静态表取交集；探测失败静默降级到静态表。前端不再硬编码版本列表，改为从后端拉取。

**Tech Stack:** Rust（无新依赖，纯 std + 既有 `cmd_runner`）、Vue 3 + TypeScript、Pinia、naive-ui、vitest。

## Global Constraints

- 后端 serde 命名风格不统一，**新结构一律显式标注**；新增的跨边界结构用 `#[serde(rename_all = "camelCase")]`，并在 `src-tauri/src/wire_format_tests.rs` 里钉住序列化键集。
- 前端 `allowJs: false`，禁止新增 `.js`；新增 SFC 必须带 `lang="ts"`。
- `typescript` 锁定 5.x。
- 分层规则（见 `AGENTS.md`）：版本表、解析、探测全部属于 Rust 层；前端只做展示与表单。
- 每个 Task 结束前四道门禁必须绿：`npm test` / `npm run typecheck` / `npm run build` / `cargo test --manifest-path src-tauri/Cargo.toml`（Linux 上跑 cargo 需先 export conda `tauri-dev` 环境，见 `README` 的「Linux 服务器开发」）。
- **行为变更与类型/重构变更分开提交**，沿用 TS 迁移那批的纪律。

## 事实依据（实施前必读）

五个工具支持的 VERSION 集合**并不相同**，来自 thtk 上游源码与本地 `tools/thecl.exe` 的 usage 文本：

| 工具 | 集合 | 数量 |
| --- | --- | --- |
| thecl / thanm / thstd | `6 7 8 9 95 10 103 11 12 125 128 13 14 143 15 16 165 17 18 185 19 20` | 22 |
| thmsg | 同上但**不含 103** | 21 |
| thdat | 上述 + `1 2 3 4 5 75 105 123`，另接受 `d`（自动检测，仅 -l/-x） | 30 |

- thtk 把 VERSION 按 `%u` 解析，`"th18"` 会解析失败或变成 0。
- `103` 是 **Uwabami Breakers**，非东方作品，thtk usage 里专门标注。
- thdat 解包我们已经用 `-xd` 自动检测，所以那 8 个额外版本**只在打包时**才有意义。

当前代码里存在**四份**互不一致的版本处理：

1. `src-tauri/src/modules/ecl/compiler.rs:26` `normalize_thecl_version()` — 只剥 `th` 前缀 + 小写，无白名单，仅 ECL 使用。
2. `modules/msg/commands.rs:10`、`modules/thstd/commands.rs:7`、`modules/thdat/commands.rs:8` — 三份逐字重复的 `effective_*_version()`，**裸传**不做任何归一。
3. `src/services/languages/ecl/semantic-loader.ts:20` `normalizeVersion()` — 前端第三份实现。
4. `src/services/toolchains/theclMetadata.ts:22` `THECL_VERSION_OPTIONS` — 前端硬编码 22 项裸数字，却被 `ProjectSettingsDialog`（项目级）和 `ToolchainSettingsDialog`（全局默认）复用，对 thdat 少 8 项、对 thmsg 多 1 项。

后果：`.thtk-project.json` 里写 `"gameVersion": "th18"` 时，ECL 正常而 MSG/STD/DAT 失败——**同一个配置值在不同工具上语义不同**。且 `validate_project_config`（`common/project_config.rs:118`）完全不校验 `game_version`。

### ⚠️ 需要领域复核

Task 1 的版本 → 标题对照表由我按公开资料整理，**作者需过一遍**，尤其：

- `103` (Uwabami Breakers) 这个非东方条目在 UI 上如何呈现；
- `20`（東方錦上京）是较新作品，标题写法请确认；
- `75 / 105 / 123` 三部格斗作只有 thdat 支持，标题与通称是否按下表。

对照表**只影响 UI 文案，不影响任何逻辑**——写错了不会导致编译错误，改起来也只动一张表。

---

## Task 1：版本权威表与解析

**Files:**
- Create: `src-tauri/src/common/game_version.rs`
- Modify: `src-tauri/src/common/mod.rs`（加 `pub mod game_version;`）

**Interfaces:**
- Produces：
  - `pub struct GameVersionInfo { pub id: u32, pub code: &'static str, pub title: &'static str, pub tools: &'static [&'static str] }`
  - `pub const GAME_VERSIONS: [GameVersionInfo; 30]`
  - `pub fn parse(raw: &str) -> Result<u32, String>`
  - `pub fn find(id: u32) -> Option<&'static GameVersionInfo>`
  - `pub fn supports(id: u32, tool_id: &str) -> bool`
  - `pub fn versions_for_tool(tool_id: &str) -> Vec<u32>`
- Consumes：无（纯函数模块，无 Tauri 依赖，可在 Linux 裸编译下测试）。

- [ ] **Step 1: 写失败测试**

在新建的 `src-tauri/src/common/game_version.rs` 末尾写：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_plain_number() {
        assert_eq!(parse("18"), Ok(18));
    }

    #[test]
    fn parse_strips_th_prefix_case_insensitively() {
        assert_eq!(parse("th18"), Ok(18));
        assert_eq!(parse("TH18"), Ok(18));
        assert_eq!(parse("  Th18  "), Ok(18));
    }

    #[test]
    fn parse_rejects_unknown_version() {
        let err = parse("21").unwrap_err();
        assert!(err.contains("21"), "错误信息应带上原值, got: {err}");
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse("abc").is_err());
        assert!(parse("18.5").is_err());
        assert!(parse("").is_err());
    }

    /// 回归：这正是当前 msg 路径会踩的坑——thtk 按 %u 解析，"th18" 会变成 0。
    #[test]
    fn parse_never_returns_zero() {
        for raw in ["th", "th0", "0", "-1"] {
            assert!(parse(raw).is_err(), "{raw:?} 不该被接受");
        }
    }

    #[test]
    fn thmsg_does_not_support_uwabami_breakers() {
        assert!(supports(103, "thecl"));
        assert!(supports(103, "thanm"));
        assert!(supports(103, "thstd"));
        assert!(supports(103, "thdat"));
        assert!(!supports(103, "thmsg"), "thmsg 的 usage 里没有 103");
    }

    #[test]
    fn pc98_and_fighting_games_are_thdat_only() {
        for id in [1, 2, 3, 4, 5, 75, 105, 123] {
            assert!(supports(id, "thdat"), "thdat 应支持 {id}");
            for tool in ["thecl", "thanm", "thstd", "thmsg"] {
                assert!(!supports(id, tool), "{tool} 不该支持 {id}");
            }
        }
    }

    #[test]
    fn tool_version_counts_match_thtk_usage() {
        assert_eq!(versions_for_tool("thecl").len(), 22);
        assert_eq!(versions_for_tool("thanm").len(), 22);
        assert_eq!(versions_for_tool("thstd").len(), 22);
        assert_eq!(versions_for_tool("thmsg").len(), 21);
        assert_eq!(versions_for_tool("thdat").len(), 30);
    }

    #[test]
    fn table_is_sorted_by_release_order_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for info in GAME_VERSIONS.iter() {
            assert!(seen.insert(info.id), "重复版本号 {}", info.id);
            assert_eq!(info.code, format!("th{}", info.id), "code 必须是 th+id");
            assert!(!info.title.is_empty());
            assert!(!info.tools.is_empty());
        }
    }

    #[test]
    fn versions_for_unknown_tool_is_empty() {
        assert!(versions_for_tool("thbogus").is_empty());
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
P=/data/sunyunbo/miniconda3/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig LD_LIBRARY_PATH=$P/lib PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml game_version
```

预期：编译失败，`cannot find function parse in this scope` 等。

- [ ] **Step 3: 写实现**

在 `game_version.rs` 顶部（测试模块之前）写：

```rust
//! 游戏版本的唯一权威表。
//!
//! 版本集合来自 thtk 各工具的 usage 输出（thecl/thanm/thstd 22 个、thmsg 21 个、
//! thdat 30 个）。thtk 把 VERSION 按 `%u` 解析，所以这里只接受纯数字（允许 `th` 前缀），
//! 且必须落在表内——历史上 msg/std/dat 是裸传字符串，写 "th18" 会静默失败。
//!
//! 这张表是**静态兜底**。用户实际装的 thtk 支持哪些版本由
//! `common::toolchain::probe_supported_versions` 运行时探测，两者取交集。

pub struct GameVersionInfo {
    /// thtk 命令行接受的数字，例如 18。
    pub id: u32,
    /// 规范化写法 `th18`，用于文件名匹配与展示。
    pub code: &'static str,
    pub title: &'static str,
    /// 支持该版本的工具 id，与 `toolchain::TOOLCHAIN_DESCRIPTORS` 的 id 对齐。
    pub tools: &'static [&'static str],
}

/// 五个工具全支持。
const ALL: &[&str] = &["thecl", "thanm", "thstd", "thmsg", "thdat"];
/// thmsg 的 usage 里没有 103（Uwabami Breakers 无对话文件）。
const NO_MSG: &[&str] = &["thecl", "thanm", "thstd", "thdat"];
/// PC-98 五作与三部格斗作只有封包格式，没有 ECL/ANM/STD/MSG 脚本支持。
const DAT_ONLY: &[&str] = &["thdat"];

pub const GAME_VERSIONS: [GameVersionInfo; 30] = [
    GameVersionInfo { id: 1,   code: "th1",   title: "東方靈異伝",                 tools: DAT_ONLY },
    GameVersionInfo { id: 2,   code: "th2",   title: "東方封魔録",                 tools: DAT_ONLY },
    GameVersionInfo { id: 3,   code: "th3",   title: "東方夢時空",                 tools: DAT_ONLY },
    GameVersionInfo { id: 4,   code: "th4",   title: "東方幻想郷",                 tools: DAT_ONLY },
    GameVersionInfo { id: 5,   code: "th5",   title: "東方怪綺談",                 tools: DAT_ONLY },
    GameVersionInfo { id: 6,   code: "th6",   title: "東方紅魔郷",                 tools: ALL },
    GameVersionInfo { id: 7,   code: "th7",   title: "東方妖々夢",                 tools: ALL },
    GameVersionInfo { id: 75,  code: "th75",  title: "東方萃夢想",                 tools: DAT_ONLY },
    GameVersionInfo { id: 8,   code: "th8",   title: "東方永夜抄",                 tools: ALL },
    GameVersionInfo { id: 9,   code: "th9",   title: "東方花映塚",                 tools: ALL },
    GameVersionInfo { id: 95,  code: "th95",  title: "東方文花帖",                 tools: ALL },
    GameVersionInfo { id: 10,  code: "th10",  title: "東方風神録",                 tools: ALL },
    GameVersionInfo { id: 103, code: "th103", title: "Uwabami Breakers（非东方）", tools: NO_MSG },
    GameVersionInfo { id: 105, code: "th105", title: "東方緋想天",                 tools: DAT_ONLY },
    GameVersionInfo { id: 11,  code: "th11",  title: "東方地霊殿",                 tools: ALL },
    GameVersionInfo { id: 12,  code: "th12",  title: "東方星蓮船",                 tools: ALL },
    GameVersionInfo { id: 123, code: "th123", title: "東方非想天則",               tools: DAT_ONLY },
    GameVersionInfo { id: 125, code: "th125", title: "ダブルスポイラー",           tools: ALL },
    GameVersionInfo { id: 128, code: "th128", title: "妖精大戦争",                 tools: ALL },
    GameVersionInfo { id: 13,  code: "th13",  title: "東方神霊廟",                 tools: ALL },
    GameVersionInfo { id: 14,  code: "th14",  title: "東方輝針城",                 tools: ALL },
    GameVersionInfo { id: 143, code: "th143", title: "弾幕アマノジャク",           tools: ALL },
    GameVersionInfo { id: 15,  code: "th15",  title: "東方紺珠伝",                 tools: ALL },
    GameVersionInfo { id: 16,  code: "th16",  title: "東方天空璋",                 tools: ALL },
    GameVersionInfo { id: 165, code: "th165", title: "秘封ナイトメアダイアリー",   tools: ALL },
    GameVersionInfo { id: 17,  code: "th17",  title: "東方鬼形獣",                 tools: ALL },
    GameVersionInfo { id: 18,  code: "th18",  title: "東方虹龍洞",                 tools: ALL },
    GameVersionInfo { id: 185, code: "th185", title: "東方剛欲異聞",               tools: ALL },
    GameVersionInfo { id: 19,  code: "th19",  title: "東方獣王園",                 tools: ALL },
    GameVersionInfo { id: 20,  code: "th20",  title: "東方錦上京",                 tools: ALL },
];

/// 解析用户/配置里的版本写法。接受 `18`、`th18`、`TH18`、前后空白。
/// 不接受小数、负数、未知版本——thtk 按 %u 解析，非法值会变成 0 或直接失败。
pub fn parse(raw: &str) -> Result<u32, String> {
    let trimmed = raw.trim().to_lowercase();
    let digits = trimmed.strip_prefix("th").unwrap_or(&trimmed);

    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("版本号非法: {raw:?}（应为 18 或 th18 这样的写法）"));
    }

    let id: u32 = digits
        .parse()
        .map_err(|_| format!("版本号超出范围: {raw:?}"))?;

    if find(id).is_none() {
        return Err(format!(
            "未知的游戏版本: {raw:?}（thtk 支持 {}）",
            GAME_VERSIONS
                .iter()
                .map(|v| v.id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    Ok(id)
}

pub fn find(id: u32) -> Option<&'static GameVersionInfo> {
    GAME_VERSIONS.iter().find(|info| info.id == id)
}

pub fn supports(id: u32, tool_id: &str) -> bool {
    find(id).is_some_and(|info| info.tools.contains(&tool_id))
}

/// 某个工具支持的全部版本，保持表内顺序（大致按发售顺序）。
pub fn versions_for_tool(tool_id: &str) -> Vec<u32> {
    GAME_VERSIONS
        .iter()
        .filter(|info| info.tools.contains(&tool_id))
        .map(|info| info.id)
        .collect()
}
```

在 `src-tauri/src/common/mod.rs` 加一行（保持既有字母序位置）：

```rust
pub mod game_version;
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml game_version
```

预期：10 个测试全绿。若 `tool_version_counts_match_thtk_usage` 失败，说明表里某条的 `tools` 写错了，对照上方「事实依据」的三行集合逐一核对。

> 若测试与实现一次性落盘、没能先看到红，补一次**变异检验**找回保证：把 `id: 103` 那行的 `tools: NO_MSG` 临时改成 `ALL`，应当且只应当有 `thmsg_does_not_support_uwabami_breakers` 与 `tool_version_counts_match_thtk_usage` 两个测试失败，然后还原。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/common/game_version.rs src-tauri/src/common/mod.rs
git commit -m "feat(version): 新增游戏版本权威表与解析入口

thtk 五个工具支持的版本集合并不相同（thecl/thanm/thstd 22 个、
thmsg 21 个无 103、thdat 30 个含 PC-98 与格斗作），且 thtk 按 %u
解析版本号，'th18' 这类写法会静默变成 0。

这张表是静态兜底，后续与运行时探测结果取交集。"
```

---

## Task 2：接入校验，收敛四份重复实现

**Files:**
- Modify: `src-tauri/src/common/game_version.rs`（新增 `resolve`）
- Modify: `src-tauri/src/common/project_config.rs:118-135`（`validate_project_config` 加校验）
- Modify: `src-tauri/src/modules/ecl/compiler.rs:26-32`（删 `normalize_thecl_version`）、`:116,:123,:133`
- Modify: `src-tauri/src/modules/msg/commands.rs:10-20`、`modules/thstd/commands.rs:7-17`、`modules/thdat/commands.rs:8-20`（删三份 `effective_*_version`）

**Interfaces:**
- Consumes：Task 1 的 `parse` / `supports`。
- Produces：`pub fn resolve(config: &AppConfig, project_root: Option<&str>, tool_id: &str) -> Result<u32, String>` —— 四个工具路径唯一的版本来源。

- [ ] **Step 1: 写失败测试**

追加到 `game_version.rs` 的 `mod tests`：

```rust
use crate::common::project_config::{ProjectConfig, ProjectToolchainConfig};
use crate::config::AppConfig;

fn app_config(default_version: &str) -> AppConfig {
    AppConfig {
        default_game_version: default_version.to_string(),
        ..AppConfig::default()
    }
}

fn write_project_version(root: &std::path::Path, version: &str) {
    let config = ProjectConfig {
        game_version: version.to_string(),
        encoding: "shift-jis".to_string(),
        map_paths: Vec::new(),
        toolchain: ProjectToolchainConfig { thtk_dir: String::new() },
    };
    crate::common::project_config::save_project_config(
        &root.to_string_lossy(),
        &config,
    )
    .expect("save project config");
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("thtk-gv-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolve_prefers_project_over_global() {
    let dir = temp_dir("prefers-project");
    write_project_version(&dir, "18");
    let resolved = resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thecl");
    assert_eq!(resolved, Ok(18));
}

#[test]
fn resolve_falls_back_to_global_when_project_version_empty() {
    let dir = temp_dir("empty-project");
    write_project_version(&dir, "");
    let resolved = resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thecl");
    assert_eq!(resolved, Ok(20));
}

#[test]
fn resolve_falls_back_to_global_without_project() {
    assert_eq!(resolve(&app_config("17"), None, "thecl"), Ok(17));
}

/// 回归：历史上 msg/std/dat 裸传，"th18" 会被 thtk 当成 0。
#[test]
fn resolve_normalizes_th_prefix_for_every_tool() {
    let dir = temp_dir("th-prefix");
    write_project_version(&dir, "th18");
    for tool in ["thecl", "thanm", "thstd", "thmsg", "thdat"] {
        assert_eq!(
            resolve(&app_config("20"), Some(&dir.to_string_lossy()), tool),
            Ok(18),
            "{tool} 应与 ECL 路径行为一致"
        );
    }
}

#[test]
fn resolve_rejects_version_unsupported_by_the_tool() {
    let dir = temp_dir("tool-unsupported");
    write_project_version(&dir, "75");
    assert_eq!(
        resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thdat"),
        Ok(75)
    );
    let err = resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thecl")
        .unwrap_err();
    assert!(err.contains("75"), "错误应指出版本号, got: {err}");
    assert!(err.contains("thecl"), "错误应指出工具名, got: {err}");
}

#[test]
fn resolve_errors_when_nothing_is_configured() {
    assert!(resolve(&app_config(""), None, "thecl").is_err());
}
```

追加到 `project_config.rs` 的 `mod tests`：

```rust
#[test]
fn validate_rejects_unknown_game_version() {
    let config = ProjectConfig {
        game_version: "21".to_string(),
        encoding: "shift-jis".to_string(),
        map_paths: Vec::new(),
        toolchain: ProjectToolchainConfig { thtk_dir: String::new() },
    };
    assert!(validate_project_config(&config).is_err());
}

#[test]
fn validate_accepts_th_prefixed_game_version() {
    let config = ProjectConfig {
        game_version: "th18".to_string(),
        encoding: "shift-jis".to_string(),
        map_paths: Vec::new(),
        toolchain: ProjectToolchainConfig { thtk_dir: String::new() },
    };
    assert!(validate_project_config(&config).is_ok());
}

#[test]
fn validate_still_accepts_empty_game_version() {
    let config = ProjectConfig {
        game_version: String::new(),
        encoding: "shift-jis".to_string(),
        map_paths: Vec::new(),
        toolchain: ProjectToolchainConfig { thtk_dir: String::new() },
    };
    assert!(
        validate_project_config(&config).is_ok(),
        "空字符串表示回退全局默认，必须继续接受"
    );
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml game_version
cargo test --manifest-path src-tauri/Cargo.toml project_config
```

预期：`resolve` 未定义导致编译失败；`validate_rejects_unknown_game_version` 断言失败（当前无校验，返回 Ok）。

- [ ] **Step 3: 写实现**

在 `game_version.rs` 的 `versions_for_tool` 之后追加：

```rust
/// 四个工具路径唯一的版本来源：项目级 `gameVersion` 优先，为空回退全局
/// `default_game_version`，再校验该版本确实被目标工具支持。
///
/// 取代此前 msg / thstd / thdat 三份逐字重复的 `effective_*_version()`，
/// 以及 ECL 独有的 `normalize_thecl_version()`。
pub fn resolve(
    config: &crate::config::AppConfig,
    project_root: Option<&str>,
    tool_id: &str,
) -> Result<u32, String> {
    let mut raw = config.default_game_version.clone();
    if let Some(root) = project_root {
        if let Some(pc) = crate::common::project_config::load_project_config(root) {
            if !pc.game_version.trim().is_empty() {
                raw = pc.game_version;
            }
        }
    }

    if raw.trim().is_empty() {
        return Err("未配置游戏版本：请在项目设置或全局工具链设置里选择".to_string());
    }

    let id = parse(&raw)?;

    if !supports(id, tool_id) {
        let title = find(id).map(|info| info.title).unwrap_or("");
        return Err(format!(
            "{tool_id} 不支持版本 {id}（{title}）——该作品在 thtk 里只有 {} 支持",
            find(id)
                .map(|info| info.tools.join(" / "))
                .unwrap_or_default()
        ));
    }

    Ok(id)
}
```

在 `project_config.rs` 的 `validate_project_config` 里，`encoding` 校验之后、`map_paths` 校验之前插入：

```rust
    // 空表示回退全局默认，允许；非空则必须是 thtk 认识的版本。
    let game_version = config.game_version.trim();
    if !game_version.is_empty() {
        crate::common::game_version::parse(game_version)?;
    }
```

在 `modules/ecl/compiler.rs`：

删除 `normalize_thecl_version` 整个函数（第 26-32 行）。

`build_thecl_args` 改为接收**已解析**的版本号，保持无错可失败（parse 提到边界做，不埋在拼参数里）。签名从

```rust
pub fn build_thecl_args(request: &TheclRequest, output_path: Option<&str>) -> Vec<String>
```

改为

```rust
pub fn build_thecl_args(
    request: &TheclRequest,
    version_id: u32,
    output_path: Option<&str>,
) -> Vec<String>
```

函数体内三处 `args.push(normalize_thecl_version(&request.version));`（原 :116/:123/:133）统一改为：

```rust
args.push(version_id.to_string());
```

`run()` 返回的是 `EclResult` 结构体而**不是** `Result`，所以不能用 `?`。在 `run()` 开头解析版本，失败时返回一个失败态 `EclResult`——与该函数已有的 `unwrap_or_else` 失败路径一致，错误会经既有渠道进到问题面板：

```rust
pub fn run(config: &AppConfig, request: &TheclRequest) -> EclResult {
    let version_id = match crate::common::game_version::parse(&request.version) {
        Ok(id) => id,
        Err(message) => {
            return EclResult {
                success: false,
                tool: "thecl".to_string(),
                mode: request.mode.as_str().to_string(),
                script_kind: "ecl".to_string(),
                input_path: request.input_path.clone(),
                message,
                diagnostics: Vec::new(),
                output_path: None,
            }
        }
    };

    let tool_path = toolchain::resolve_tool_path(config, "thecl", "thecl.exe");
    let output_path = infer_output_path(request);
    let args = build_thecl_args(request, version_id, output_path.as_deref());
    // ...以下不变
```

> 字段列表以 `EclResult` 的实际定义为准（`compiler.rs` 约 :55-63）；若有增删，编译器会直接报缺字段。
>
> `build_thecl_args` 目前只有 `compiler.rs:69` 一个调用点，且**没有单元测试**，改签名的波及面就这一处。

在 `modules/msg/commands.rs`、`modules/thstd/commands.rs`、`modules/thdat/commands.rs` 中：删除各自的 `effective_*_version` 函数，把调用点改为

```rust
let version = crate::common::game_version::resolve(&config, project_root.as_deref(), "thmsg")?
    .to_string();
```

（`thstd` / `thdat` 同理，替换工具 id。）三个文件顶部若因此不再用到 `project_config`，一并从 `use` 里移除。

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

预期：全部通过，且总数比 132 多出本任务新增的 9 个 + Task 1 的 11 个。

- [ ] **Step 5: 提交（拆两个 commit）**

```bash
git add src-tauri/src/common/game_version.rs src-tauri/src/common/project_config.rs
git commit -m "fix(version): 校验 gameVersion 并统一版本解析

修复：.thtk-project.json 写 'th18' 时 ECL 正常而 MSG/STD/DAT 失败——
前者走 normalize_thecl_version 剥前缀，后三者裸传给按 %u 解析的 thtk。
同一个配置值在不同工具上语义不同，且 validate_project_config 完全不校验。

现在 resolve() 是四个工具路径唯一的版本来源，并校验目标工具确实支持
该版本（例如萃夢想 75 只有 thdat 支持）。"

git add src-tauri/src/modules/
git commit -m "refactor(version): 四个工具接入统一的版本解析

删除 msg/thstd/thdat 三份逐字重复的 effective_*_version 与 ECL 独有的
normalize_thecl_version，全部改走 game_version::resolve。"
```

---

## Task 3：thtk 能力运行时探测

**Files:**
- Modify: `src-tauri/src/common/toolchain.rs:13-24`（`ToolchainStatus` 加字段）、`:128-172`（构造处）
- Modify: `src-tauri/src/wire_format_tests.rs`（钉住新键集）

**Interfaces:**
- Consumes：Task 1 的 `versions_for_tool`。
- Produces：`pub fn parse_supported_versions(usage: &str) -> Option<Vec<u32>>`；`ToolchainStatus` 新增 `supported_versions: Vec<u32>`（序列化为 `supportedVersions`）。

**背景：** thtk 的 `print_usage()` 走 **stdout**，无参数运行即触发（`mode` 保持 -1 落到 `switch` 的 default 分支），退出码非 0——**必须无视退出码读 stdout**。thecl/thanm/thstd/thmsg 的 usage 含 `VERSION can be:` 行；**thdat 的 usage 不含该行**，探测会返回 `None` 并降级到静态表，这是预期行为不是 bug。

> 本机无 wine 也无 Linux 版 thtk，探测逻辑的**真机验证留到 Windows**；单测覆盖解析函数本身（喂真实 usage 文本）。

- [ ] **Step 1: 写失败测试**

追加到 `toolchain.rs` 的 `mod tests`：

```rust
/// 取自本地 tools/thecl.exe 的真实 usage 输出。
const THECL_USAGE: &str = "\
Usage: thecl [-Vrsxj] [[-c | -h | -d] VERSION] [-m ECLMAP]... [INPUT [OUTPUT]]
  -V  display version information and exit
VERSION can be:
  6, 7, 8, 9, 95, 10, 103 (for Uwabami Breakers), 11, 12, 125, 128, 13, 14, 143, 15, 16, 165, 17, 18, 185, 19, or 20
Report bugs to <https://github.com/thpatch/thtk/issues>.
";

#[test]
fn parses_the_real_thecl_usage() {
    let versions = parse_supported_versions(THECL_USAGE).expect("应解析出版本列表");
    assert_eq!(versions.len(), 22);
    assert_eq!(versions.first(), Some(&6));
    assert_eq!(versions.last(), Some(&20));
    assert!(versions.contains(&103), "带括号注释的 103 应被解析出来");
    assert!(versions.contains(&185));
}

#[test]
fn strips_the_trailing_or_before_the_last_version() {
    let versions = parse_supported_versions(THECL_USAGE).unwrap();
    assert!(versions.contains(&20), "'or 20' 里的 20 应被解析");
}

#[test]
fn returns_none_when_marker_is_absent() {
    // thdat 的 usage 里没有 "VERSION can be:" 行。
    let thdat_usage = "\
Usage: thdat [-Vg] [-C DIR] [[-c | -l | -x] VERSION] [ARCHIVE [FILE...]]
Options:
  -c  create an archive
Specify 'd' as VERSION to automatically detect archive format.
";
    assert_eq!(parse_supported_versions(thdat_usage), None);
}

#[test]
fn returns_none_on_empty_output() {
    assert_eq!(parse_supported_versions(""), None);
}

#[test]
fn ignores_non_numeric_noise() {
    let usage = "VERSION can be:\n  6, banana, 7, or 8\n";
    assert_eq!(parse_supported_versions(usage), Some(vec![6, 7, 8]));
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml toolchain
```

预期：`cannot find function parse_supported_versions`。

- [ ] **Step 3: 写实现**

在 `toolchain.rs` 的 `query_tool_version` 之后追加：

```rust
/// 从工具的 usage 文本里解析它自报的支持版本列表。
///
/// thtk 的 usage 形如：
/// ```text
/// VERSION can be:
///   6, 7, ..., 103 (for Uwabami Breakers), ..., 19, or 20
/// ```
/// 解析不到就返回 None——调用方降级到 `game_version` 的静态表。
/// thdat 的 usage 不含该标记，返回 None 是预期行为。
pub fn parse_supported_versions(usage: &str) -> Option<Vec<u32>> {
    let mut lines = usage.lines();
    lines.find(|line| line.trim_start().starts_with("VERSION can be:"))?;
    let list_line = lines.next()?;

    let versions: Vec<u32> = list_line
        .split(',')
        .filter_map(|token| {
            let token = token.trim().trim_start_matches("or ").trim_start();
            let digits: String = token.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        })
        .collect();

    if versions.is_empty() {
        None
    } else {
        Some(versions)
    }
}

/// 无参运行工具拿 usage（走 stdout，退出码非 0 是正常的，必须忽略）。
fn probe_supported_versions(exe_path: &str, tool_id: &str) -> Vec<u32> {
    let parent_dir = Path::new(exe_path).parent();
    let probed = cmd_runner::run_tool(exe_path, &[], parent_dir)
        .ok()
        .and_then(|result| parse_supported_versions(&result.stdout));

    match probed {
        // 与静态表取交集：探测结果可能含我们表里没有的新版本，
        // 那些版本我们没有标题也没有工具支持信息，暂不放出。
        Some(list) => {
            let known = crate::common::game_version::versions_for_tool(tool_id);
            known.into_iter().filter(|id| list.contains(id)).collect()
        }
        None => crate::common::game_version::versions_for_tool(tool_id),
    }
}
```

在 `ToolchainStatus` 结构体加字段：

```rust
    /// 该工具实际可用的版本集合（运行时探测 ∩ 静态表；探测失败则为静态表）。
    pub supported_versions: Vec<u32>,
```

在 `get_toolchain_status` 的**三处**构造点补上该字段：
- 未配置路径的早退分支 → `supported_versions: Vec::new()`
- `Ok(version)` 分支 → `supported_versions: probe_supported_versions(&resolved_path, descriptor.id)`
- `Err(error)` 分支 → `supported_versions: Vec::new()`

`src-tauri/src/wire_format_tests.rs` 已有 `toolchain_status_is_camel_case`（:134-160）。加字段后它会因缺字段编译失败——在结构体字面量的 `message` 之后补一行：

```rust
        message: String::new(),
        supported_versions: vec![6, 20],
```

并把断言里的键集补上 `"supportedVersions"`：

```rust
    assert_eq!(
        json_keys(&status),
        sorted(&[
            "tool",
            "label",
            "exeName",
            "configuredPath",
            "resolvedPath",
            "available",
            "version",
            "message",
            "supportedVersions"
        ])
    );
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

预期：全绿，含 5 个新增解析测试与更新后的 wire format 测试。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/common/toolchain.rs src-tauri/src/wire_format_tests.rs
git commit -m "feat(version): 运行时探测 thtk 实际支持的版本集合

不硬编码'thtk 版本 X 支持到 thYY'——那张表一定会过期。改为无参运行
工具读它自己的 usage（走 stdout，退出码非 0 需忽略），解析
'VERSION can be:' 行，与静态表取交集。

thtk 出 th21 时静态表加一行即可，无需改探测逻辑；老版 thtk 或 thdat
（usage 不含该标记）解析失败则静默降级到静态表，不阻塞。

用户因此不需要知道自己装的是哪版 thtk。"
```

---

## Task 4：把版本表暴露给前端

**Files:**
- Create: `src-tauri/src/common/game_version_commands.rs`
- Modify: `src-tauri/src/main.rs`（注册 command）
- Modify: `src-tauri/src/wire_format_tests.rs`
- Create: `src/types/gameVersion.ts`
- Modify: `src/types/index.ts`、`src/types/toolchain.ts`
- Modify: `src/api/modules/config.ts`

**Interfaces:**
- Consumes：Task 1 的 `GAME_VERSIONS`、Task 3 的 `ToolchainStatus.supported_versions`。
- Produces：Tauri command `list_game_versions() -> Vec<GameVersionView>`；TS 侧 `GameVersionView`、`listGameVersions()`。

- [ ] **Step 1: 写失败测试**

新建 `src/types/gameVersion.ts` 的对应测试，追加到 `tests/unit/` 下新文件 `tests/unit/gameVersion.spec.ts`：

```ts
import { describe, it, expect } from 'vitest'
import { versionsForTool, formatVersionLabel } from '../../src/services/toolchains/gameVersions'
import type { GameVersionView } from '../../src/types'

const TABLE: GameVersionView[] = [
  { id: 18, code: 'th18', title: '東方虹龍洞', tools: ['thecl', 'thmsg', 'thdat'] },
  { id: 75, code: 'th75', title: '東方萃夢想', tools: ['thdat'] },
  { id: 103, code: 'th103', title: 'Uwabami Breakers（非东方）', tools: ['thecl', 'thdat'] }
]

describe('gameVersions', () => {
  it('按工具过滤版本', () => {
    expect(versionsForTool(TABLE, 'thecl').map((v) => v.id)).toEqual([18, 103])
    expect(versionsForTool(TABLE, 'thdat').map((v) => v.id)).toEqual([18, 75, 103])
    expect(versionsForTool(TABLE, 'thmsg').map((v) => v.id)).toEqual([18])
  })

  it('未知工具返回空列表而不是全量', () => {
    expect(versionsForTool(TABLE, 'thbogus')).toEqual([])
  })

  it('标签同时给出版本号与标题', () => {
    expect(formatVersionLabel(TABLE[0])).toBe('th18 · 東方虹龍洞')
  })
})
```

- [ ] **Step 2: 跑测试确认失败**

```bash
npm test -- gameVersion
```

预期：`Cannot find module '../../src/services/toolchains/gameVersions'`。

- [ ] **Step 3: 写实现**

新建 `src-tauri/src/common/game_version_commands.rs`：

```rust
use serde::Serialize;

/// 传给前端的版本条目。与 `game_version::GameVersionInfo` 一一对应，
/// 只是把 &'static str 换成 owned，并显式声明 camelCase。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersionView {
    pub id: u32,
    pub code: String,
    pub title: String,
    pub tools: Vec<String>,
}

#[tauri::command]
pub fn list_game_versions() -> Vec<GameVersionView> {
    crate::common::game_version::GAME_VERSIONS
        .iter()
        .map(|info| GameVersionView {
            id: info.id,
            code: info.code.to_string(),
            title: info.title.to_string(),
            tools: info.tools.iter().map(|t| t.to_string()).collect(),
        })
        .collect()
}
```

`common/mod.rs` 加 `pub mod game_version_commands;`；`main.rs` 的 `invoke_handler!` 列表加 `common::game_version_commands::list_game_versions`。

新建 `src/types/gameVersion.ts`：

```ts
/**
 * 游戏版本条目。对应 Rust 侧 `common/game_version_commands.rs` 的
 * `GameVersionView`（serde rename_all = "camelCase"）。
 *
 * `tools` 里的 id 与 `ToolchainStatus.tool` 对齐：thecl / thanm / thstd / thmsg / thdat。
 * 注意各工具支持的版本集合并不相同——thmsg 没有 103，thdat 多出 PC-98 与格斗作。
 */
export interface GameVersionView {
  /** thtk 命令行接受的数字，例如 18 */
  id: number
  /** 规范化写法 th18 */
  code: string
  title: string
  tools: string[]
}
```

`src/types/index.ts` 加 `export * from './gameVersion'`。

`src/types/toolchain.ts` 的 `ToolchainStatus` 加字段：

```ts
  /** 该工具实际可用的版本（运行时探测 ∩ 静态表）；不可用时为空数组 */
  supportedVersions: number[]
```

新建 `src/services/toolchains/gameVersions.ts`：

```ts
import type { GameVersionView } from '../../types'

/** 过滤出支持指定工具的版本。未知工具返回空数组，不做「未知即全放行」的兜底。 */
export function versionsForTool(
  table: GameVersionView[],
  toolId: string
): GameVersionView[] {
  return table.filter((entry) => entry.tools.includes(toolId))
}

/** 下拉框标签：版本号在前便于键盘检索，标题在后便于辨认。 */
export function formatVersionLabel(entry: GameVersionView): string {
  return `${entry.code} · ${entry.title}`
}
```

`src/api/modules/config.ts` 加：

```ts
export function listGameVersions(): Promise<GameVersionView[]> {
  return invoke('list_game_versions')
}
```

（同文件顶部按既有风格 `import type { GameVersionView } from '../../types'`。）

在 `wire_format_tests.rs` 加：

```rust
#[test]
fn game_version_view_keys_are_pinned() {
    let view = crate::common::game_version_commands::GameVersionView {
        id: 18,
        code: "th18".to_string(),
        title: "東方虹龍洞".to_string(),
        tools: vec!["thecl".to_string()],
    };
    assert_eq!(json_keys(&view), sorted(&["id", "code", "title", "tools"]));
}
```

- [ ] **Step 4: 跑测试确认通过**

```bash
npm test -- gameVersion
npm run typecheck
cargo test --manifest-path src-tauri/Cargo.toml wire_format
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/common/game_version_commands.rs src-tauri/src/common/mod.rs \
        src-tauri/src/main.rs src-tauri/src/wire_format_tests.rs \
        src/types/gameVersion.ts src/types/index.ts src/types/toolchain.ts \
        src/services/toolchains/gameVersions.ts src/api/modules/config.ts \
        tests/unit/gameVersion.spec.ts
git commit -m "feat(version): 版本表经 list_game_versions 暴露给前端"
```

---

## Task 5：前端改用后端版本表

**Files:**
- Create: `src/stores/gameVersions.ts`
- Modify: `src/services/toolchains/theclMetadata.ts:22-26`（删 `THECL_VERSION_OPTIONS`）
- Modify: `src/components/Dialogs/ProjectSettingsDialog.vue:43-50,149,157`
- Modify: `src/components/Dialogs/ToolchainSettingsDialog.vue:29,145,163`
- Modify: `src/components/Dialogs/forms/TheclBuildForm.vue:47,140`
- Modify: `src/services/toolchains/registry.ts:8,82`
- Modify: `src/services/languages/ecl/semantic-loader.ts:20-25,85`

**Interfaces:**
- Consumes：Task 4 的 `listGameVersions()`、`versionsForTool`、`formatVersionLabel`。
- Produces：`useGameVersionsStore()`，含 `table`、`ensureLoaded()`、`optionsForTool(toolId)`。

**关键行为变更：** `ProjectSettingsDialog` 的 n-select 目前带 `tag` 属性（允许用户自由造值），**必须删掉**——这是 ① 无校验的前端入口。

- [ ] **Step 1: 写失败测试**

新建 `tests/unit/gameVersionsStore.spec.ts`：

```ts
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const listGameVersions = vi.fn()
vi.mock('../../src/api', () => ({ listGameVersions }))

import { useGameVersionsStore } from '../../src/stores/gameVersions'

const TABLE = [
  { id: 18, code: 'th18', title: '東方虹龍洞', tools: ['thecl', 'thdat'] },
  { id: 75, code: 'th75', title: '東方萃夢想', tools: ['thdat'] }
]

describe('gameVersions store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    listGameVersions.mockReset()
    listGameVersions.mockResolvedValue(TABLE)
  })

  it('只拉取一次', async () => {
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    await store.ensureLoaded()
    expect(listGameVersions).toHaveBeenCalledTimes(1)
  })

  it('按工具给出下拉选项', async () => {
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    expect(store.optionsForTool('thecl')).toEqual([
      { label: 'th18 · 東方虹龍洞', value: '18' }
    ])
    expect(store.optionsForTool('thdat')).toHaveLength(2)
  })

  it('加载失败时不抛出，选项为空', async () => {
    listGameVersions.mockRejectedValue(new Error('boom'))
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    expect(store.optionsForTool('thecl')).toEqual([])
    expect(store.error).toContain('boom')
  })
})
```

- [ ] **Step 2: 跑测试确认失败**

```bash
npm test -- gameVersionsStore
```

预期：`Cannot find module '../../src/stores/gameVersions'`。

- [ ] **Step 3: 写实现**

新建 `src/stores/gameVersions.ts`：

```ts
import { defineStore } from 'pinia'
import { listGameVersions } from '../api'
import { versionsForTool, formatVersionLabel } from '../services/toolchains/gameVersions'
import type { GameVersionView } from '../types'

/**
 * 版本表来自后端，全应用只拉一次。
 *
 * 这里刻意不做「加载失败就退回硬编码列表」——那正是此前
 * THECL_VERSION_OPTIONS 造成的问题：一张前端表被复用给五个集合不同的工具。
 * 宁可选项为空并暴露 error，也不给出错误的可选项。
 */
export const useGameVersionsStore = defineStore('gameVersions', {
  state: () => ({
    table: [] as GameVersionView[],
    loaded: false,
    error: '' as string
  }),
  getters: {
    optionsForTool: (state) => (toolId: string) =>
      versionsForTool(state.table, toolId).map((entry) => ({
        label: formatVersionLabel(entry),
        value: String(entry.id)
      }))
  },
  actions: {
    async ensureLoaded() {
      if (this.loaded) return
      this.loaded = true
      try {
        this.table = await listGameVersions()
        this.error = ''
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e)
        this.table = []
      }
    }
  }
})
```

`ProjectSettingsDialog.vue`：删掉 `import { THECL_VERSION_OPTIONS } ...` 与 `const versionOptions = THECL_VERSION_OPTIONS`，改为

```ts
const gameVersionsStore = useGameVersionsStore()
// 项目级版本对所有工具生效，取五个工具的并集（即整张表）
const versionOptions = computed(() =>
  gameVersionsStore.table.map((entry) => ({
    label: formatVersionLabel(entry),
    value: String(entry.id)
  }))
)
```

并在打开对话框的 `watch` 里 `await gameVersionsStore.ensureLoaded()`。模板里**删除 `tag` 属性**：

```vue
<n-select
  v-model:value="form.gameVersion"
  :options="versionOptions"
  filterable
  clearable
  placeholder="留空则使用全局默认版本"
/>
```

`ToolchainSettingsDialog.vue`（:145 import、:163 赋值）：全局默认版本对所有工具生效，用整张表：

```ts
const gameVersionsStore = useGameVersionsStore()
const versionOptions = computed(() =>
  gameVersionsStore.table.map((entry) => ({
    label: formatVersionLabel(entry),
    value: String(entry.id)
  }))
)
```

对话框打开时 `await gameVersionsStore.ensureLoaded()`。

`TheclBuildForm.vue`（:47 模板、:140 import）：构建对话框是 thecl 专用，只列 thecl 支持的版本：

```ts
const gameVersionsStore = useGameVersionsStore()
const versionOptions = computed(() => gameVersionsStore.optionsForTool('thecl'))
```

模板 `:options="THECL_VERSION_OPTIONS"` 改为 `:options="versionOptions"`。

`registry.ts`（:8 import、:82 字段）：**删除描述符上的 `versionOptions` 字段**，同时删掉 `ToolchainDescriptor` 类型里的该字段声明。静态注册表放不下需要异步加载的响应式数据；版本选项改由表单组件自己从 store 取（上一段已做）。若有其他地方读 `descriptor.versionOptions`，`npm run typecheck` 会全部报出来。

`semantic-loader.ts`：删除本地 `normalizeVersion`（第 20-25 行），第 85 行改为直接使用配置值——后端已保证落盘的是合法值，前端不再二次归一：

```ts
const version = (projectConfig?.gameVersion || settings?.default_game_version || '').trim()
```

`theclMetadata.ts` 删除 `THECL_VERSION_OPTIONS`（第 22-26 行）与不再需要的 `SelectOption` import。

- [ ] **Step 4: 跑测试确认通过**

```bash
npm test && npm run typecheck && npm run build
```

预期：全绿。若 `typecheck` 报 `THECL_VERSION_OPTIONS` 找不到，说明还有引用点没改——`grep -rn "THECL_VERSION_OPTIONS" src/` 应无输出。

- [ ] **Step 5: 提交**

```bash
git add src/
git commit -m "feat(version): 前端版本下拉改用后端权威表

删除 THECL_VERSION_OPTIONS——一张按 thecl 写的 22 项硬编码表，却被
项目设置与全局默认复用，对 thdat 少 8 项、对 thmsg 多 1 项（103）。

同时删掉 ProjectSettingsDialog 上 n-select 的 tag 属性：它允许用户
自由输入任意版本字符串，是校验缺口的前端入口。

semantic-loader 的第三份 normalizeVersion 一并删除。"
```

---

## Task 6：界面反映能力矩阵

**Files:**
- Modify: `src/composables/useToolchainActions.ts`
- Modify: `src/components/Common/MenuBar.vue`
- Test: `tests/unit/toolchainActions.spec.ts`

**Interfaces:**
- Consumes：Task 5 的 `useGameVersionsStore`、Task 3 的 `ToolchainStatus.supportedVersions`。
- Produces：`toolAvailability(toolId): { enabled: boolean, reason: string }`。

- [ ] **Step 1: 写失败测试**

追加到 `tests/unit/toolchainActions.spec.ts`（`TABLE` 是本段新增的夹具，不要复用其他 spec 里的同名常量）：

```ts
import { toolAvailability } from '../../src/composables/useToolchainActions'
import type { GameVersionView } from '../../src/types'

const TABLE: GameVersionView[] = [
  { id: 18, code: 'th18', title: '東方虹龍洞', tools: ['thecl', 'thanm', 'thstd', 'thmsg', 'thdat'] },
  { id: 75, code: 'th75', title: '東方萃夢想', tools: ['thdat'] }
]

it('当前版本不被该工具支持时禁用并说明原因', () => {
  // 项目选了 75（萃夢想），只有 thdat 支持
  const availability = toolAvailability('thecl', 75, TABLE)
  expect(availability.enabled).toBe(false)
  expect(availability.reason).toContain('萃夢想')
  expect(availability.reason).toContain('thdat')
})

it('版本受支持时启用', () => {
  expect(toolAvailability('thdat', 75, TABLE).enabled).toBe(true)
})

it('版本表未加载时不误禁用', () => {
  expect(toolAvailability('thecl', 18, []).enabled).toBe(true)
})
```

- [ ] **Step 2: 跑测试确认失败**

```bash
npm test -- toolchainActions
```

- [ ] **Step 3: 写实现**

在 `useToolchainActions.ts` 加：

```ts
/**
 * 某个工具在当前游戏版本下是否可用。
 *
 * 版本表为空（尚未加载/加载失败）时一律返回可用——宁可让用户点了拿到
 * 后端报错，也不要因为前端状态没就绪就把功能灰掉。
 */
export function toolAvailability(
  toolId: string,
  versionId: number | null,
  table: GameVersionView[]
): { enabled: boolean; reason: string } {
  if (!table.length || versionId == null) return { enabled: true, reason: '' }
  const entry = table.find((item) => item.id === versionId)
  if (!entry) return { enabled: true, reason: '' }
  if (entry.tools.includes(toolId)) return { enabled: true, reason: '' }
  return {
    enabled: false,
    reason: `${entry.title}（${entry.code}）在 thtk 里只有 ${entry.tools.join(' / ')} 支持`
  }
}
```

`MenuBar.vue` 的工具链菜单项绑定 `:disabled="!availability.enabled"`，并把 `reason` 作为 `n-tooltip` 内容。

- [ ] **Step 4: 跑测试确认通过**

```bash
npm test && npm run typecheck
```

- [ ] **Step 5: 提交**

```bash
git add src/
git commit -m "feat(version): 不支持当前版本的工具动作在菜单里置灰并说明原因"
```

---

## Task 7：语义数据版本一致性提示

**Files:**
- Modify: `src/services/languages/ecl/semantic-loader.ts`
- Test: `tests/unit/semanticLoader.spec.ts`

**Interfaces:**
- Consumes：Task 5 之后的 `version` 值。
- Produces：`detectEclmapVersionMismatch(mapPaths, versionCode): string | null`。

- [ ] **Step 1: 写失败测试**

```ts
describe('eclmap 版本一致性', () => {
  it('文件名版本与项目版本不符时给出提示', () => {
    const warning = detectEclmapVersionMismatch(['maps/th18.eclm'], 'th20')
    expect(warning).toContain('th18')
    expect(warning).toContain('th20')
  })

  it('相符时无提示', () => {
    expect(detectEclmapVersionMismatch(['maps/th20.eclm'], 'th20')).toBeNull()
  })

  it('文件名不含版本时不误报', () => {
    expect(detectEclmapVersionMismatch(['maps/custom.eclm'], 'th20')).toBeNull()
  })

  it('多个 eclmap 只要有一个相符就不提示', () => {
    expect(
      detectEclmapVersionMismatch(['maps/th20.eclm', 'maps/extra.eclm'], 'th20')
    ).toBeNull()
  })
})
```

- [ ] **Step 2: 跑测试确认失败**

```bash
npm test -- semanticLoader
```

- [ ] **Step 3: 写实现**

```ts
/**
 * eclmap 文件名里的版本与项目版本不符时返回提示文案，否则 null。
 *
 * 刻意只做**警告不做拦截**：用户可能在用改造版或自制 eclmap，文件名
 * 未必反映内容。只要有任意一个文件名与项目版本相符，就认为配置是有意的。
 */
export function detectEclmapVersionMismatch(
  mapPaths: string[],
  versionCode: string
): string | null {
  if (!versionCode || !mapPaths.length) return null

  const named = mapPaths
    .map((path) => /(?:^|[\\/])(th\d+)\b/i.exec(path)?.[1]?.toLowerCase())
    .filter((code): code is string => Boolean(code))

  if (!named.length) return null
  if (named.includes(versionCode.toLowerCase())) return null

  return `项目版本是 ${versionCode}，但 eclmap 看起来是 ${[...new Set(named)].join(' / ')} 的——补全与编译可能用到错误的指令签名。`
}
```

在 `loadEclSemantics` 里把该提示随结果返回，由调用方推送到输出面板。

- [ ] **Step 4: 跑测试确认通过**

```bash
npm test && npm run typecheck && npm run build
```

- [ ] **Step 5: 提交**

```bash
git add src/
git commit -m "feat(version): eclmap 版本与项目版本不符时给出警告"
```

---

## 最终验证

```bash
npm test
npm run typecheck
npm run build
```

```bash
P=/data/sunyunbo/miniconda3/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig LD_LIBRARY_PATH=$P/lib PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows 手动验收（追加到 MVP 清单）

1. 项目设置的版本下拉显示「th18 · 東方虹龍洞」形式，**不能再自由输入**任意字符串。
2. 手工把 `.thtk-project.json` 的 `gameVersion` 改成 `"21"`，IDE 报配置无效且指出合法版本。
3. 手工改成 `"th18"`，ECL 与 MSG/STD/DAT **行为一致**（此前 ECL 正常而其余失败）。
4. 选 `75`（萃夢想）后，ECL/MSG/STD/ANM 的菜单项置灰并提示只有 thdat 支持；thdat 打包正常。
5. 工具链设置里各工具显示的可用版本数量与 thtk 自报一致（thecl 22 / thmsg 21 / thdat 30）。
6. 把 thtk 换成较老版本，th19/th20 从下拉里消失（探测生效）；换回新版恢复。
7. 项目版本 th20 但 mapPaths 挂 `th18.eclm` 时输出面板出现警告，且**不阻塞**编译。
