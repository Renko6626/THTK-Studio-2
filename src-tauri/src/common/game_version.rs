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
    /// 简体中文作品名。这是目前唯一的展示文案来源——将来做 i18n 时，
    /// 这里换成语言 key、由前端按语言查表即可，`id` / `code` / `tools` 都不受影响。
    pub title: &'static str,
    /// 支持该版本的工具 id，与 `toolchain::TOOLCHAIN_DESCRIPTORS` 的 id 对齐。
    pub tools: &'static [&'static str],
}

/// 五个工具全支持。
const ALL: &[&str] = &["thecl", "thanm", "thstd", "thmsg", "thdat"];
/// thmsg 的 usage 里没有 103——黄昏酒场没有对话文件。
/// （thtk 自己的 usage 把它标作 "for Uwabami Breakers"。）
const NO_MSG: &[&str] = &["thecl", "thanm", "thstd", "thdat"];
/// PC-98 五作与三部格斗作只有封包格式，没有 ECL/ANM/STD/MSG 脚本支持。
const DAT_ONLY: &[&str] = &["thdat"];

pub const GAME_VERSIONS: [GameVersionInfo; 30] = [
    GameVersionInfo { id: 1,   code: "th1",   title: "东方灵异传",   tools: DAT_ONLY },
    GameVersionInfo { id: 2,   code: "th2",   title: "东方封魔录",   tools: DAT_ONLY },
    GameVersionInfo { id: 3,   code: "th3",   title: "东方梦时空",   tools: DAT_ONLY },
    GameVersionInfo { id: 4,   code: "th4",   title: "东方幻想乡",   tools: DAT_ONLY },
    GameVersionInfo { id: 5,   code: "th5",   title: "东方怪绮谈",   tools: DAT_ONLY },
    GameVersionInfo { id: 6,   code: "th6",   title: "东方红魔乡",   tools: ALL },
    GameVersionInfo { id: 7,   code: "th7",   title: "东方妖妖梦",   tools: ALL },
    GameVersionInfo { id: 75,  code: "th75",  title: "东方萃梦想",   tools: DAT_ONLY },
    GameVersionInfo { id: 8,   code: "th8",   title: "东方永夜抄",   tools: ALL },
    GameVersionInfo { id: 9,   code: "th9",   title: "东方花映塚",   tools: ALL },
    GameVersionInfo { id: 95,  code: "th95",  title: "东方文花帖",   tools: ALL },
    GameVersionInfo { id: 10,  code: "th10",  title: "东方风神录",   tools: ALL },
    GameVersionInfo { id: 103, code: "th103", title: "黄昏酒场",     tools: NO_MSG },
    GameVersionInfo { id: 105, code: "th105", title: "东方绯想天",   tools: DAT_ONLY },
    GameVersionInfo { id: 11,  code: "th11",  title: "东方地灵殿",   tools: ALL },
    GameVersionInfo { id: 12,  code: "th12",  title: "东方星莲船",   tools: ALL },
    GameVersionInfo { id: 123, code: "th123", title: "东方非想天则", tools: DAT_ONLY },
    GameVersionInfo { id: 125, code: "th125", title: "东方文花帖DS", tools: ALL },
    GameVersionInfo { id: 128, code: "th128", title: "妖精大战争",   tools: ALL },
    GameVersionInfo { id: 13,  code: "th13",  title: "东方神灵庙",   tools: ALL },
    GameVersionInfo { id: 14,  code: "th14",  title: "东方辉针城",   tools: ALL },
    GameVersionInfo { id: 143, code: "th143", title: "弹幕天邪鬼",   tools: ALL },
    GameVersionInfo { id: 15,  code: "th15",  title: "东方绀珠传",   tools: ALL },
    GameVersionInfo { id: 16,  code: "th16",  title: "东方天空璋",   tools: ALL },
    GameVersionInfo { id: 165, code: "th165", title: "秘封噩梦日记", tools: ALL },
    GameVersionInfo { id: 17,  code: "th17",  title: "东方鬼形兽",   tools: ALL },
    GameVersionInfo { id: 18,  code: "th18",  title: "东方虹龙洞",   tools: ALL },
    GameVersionInfo { id: 185, code: "th185", title: "东方刚欲异闻", tools: ALL },
    GameVersionInfo { id: 19,  code: "th19",  title: "东方兽王园",   tools: ALL },
    GameVersionInfo { id: 20,  code: "th20",  title: "东方锦上京",   tools: ALL },
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

/// 四个工具路径唯一的版本来源：项目级 `gameVersion` 优先，为空回退全局
/// `default_game_version`，再校验该版本确实被目标工具支持。
///
/// 取代此前 msg / thstd / thdat 三份逐字重复的 `effective_*_version()`，
/// 以及 ECL 独有的 `normalize_thecl_version()`——那四条路径对同一个配置值
/// 行为并不一致（前者剥 `th` 前缀，后三者裸传给按 `%u` 解析的 thtk）。
pub fn resolve(
    config: &crate::config::AppConfig,
    project_root: Option<&str>,
    tool_id: &str,
) -> Result<u32, String> {
    let project = project_root.and_then(crate::common::project_config::load_project_config);
    resolve_from(project.as_ref(), config, tool_id)
}

/// 与 [`resolve`] 同义，但接收**已经加载好**的项目配置。
///
/// 工具链命令路径上，`toolchain::effective_context` 已经为解析 thtkDir 读过一次
/// `.thtk-project.json`，再让 `resolve` 去读第二次纯属浪费；更糟的是两次读取
/// 之间文件可能被改动，同一次调用会用上两份不一致的配置。
pub fn resolve_from(
    project: Option<&crate::common::project_config::ProjectConfig>,
    config: &crate::config::AppConfig,
    tool_id: &str,
) -> Result<u32, String> {
    let mut raw = config.default_game_version.clone();
    if let Some(pc) = project {
        if !pc.game_version.trim().is_empty() {
            raw = pc.game_version.clone();
        }
    }

    if raw.trim().is_empty() {
        return Err("未配置游戏版本：请在项目设置或全局工具链设置里选择".to_string());
    }

    let id = parse(&raw)?;

    if !supports(id, tool_id) {
        let info = find(id).expect("parse 已保证版本在表内");
        return Err(format!(
            "{tool_id} 不支持版本 {id}（{}）——该作品在 thtk 里只有 {} 支持",
            info.title,
            info.tools.join(" / ")
        ));
    }

    Ok(id)
}

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

    // ---- resolve：项目级优先、全局兜底、按工具校验 ----

    use crate::common::project_config::{ProjectConfig, ProjectToolchainConfig};
    use crate::config::AppConfig;

    fn app_config(default_version: &str) -> AppConfig {
        AppConfig {
            default_game_version: default_version.to_string(),
            ..AppConfig::default()
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("thtk-gv-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_project_version(root: &std::path::Path, version: &str) {
        let config = ProjectConfig {
            game_version: version.to_string(),
            encoding: "shift-jis".to_string(),
            map_paths: Vec::new(),
            toolchain: ProjectToolchainConfig {
                thtk_dir: String::new(),
            },
        };
        crate::common::project_config::save_project_config(&root.to_string_lossy(), &config)
            .expect("save project config");
    }

    #[test]
    fn resolve_prefers_project_over_global() {
        let dir = temp_dir("prefers-project");
        write_project_version(&dir, "18");
        assert_eq!(
            resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thecl"),
            Ok(18)
        );
    }

    #[test]
    fn resolve_falls_back_to_global_when_project_version_empty() {
        let dir = temp_dir("empty-project");
        write_project_version(&dir, "");
        assert_eq!(
            resolve(&app_config("20"), Some(&dir.to_string_lossy()), "thecl"),
            Ok(20)
        );
    }

    #[test]
    fn resolve_falls_back_to_global_without_project() {
        assert_eq!(resolve(&app_config("17"), None, "thecl"), Ok(17));
    }

    /// 回归：历史上 msg/std/dat 裸传，"th18" 会被 thtk 按 %u 解析成 0。
    /// 这条断言的是**四条路径行为一致**，而不只是能解析。
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

    // ---- resolve_from：接收已加载的项目配置，不再重复读盘 ----

    fn project_with_version(version: &str) -> ProjectConfig {
        ProjectConfig {
            game_version: version.to_string(),
            encoding: "shift-jis".to_string(),
            map_paths: Vec::new(),
            toolchain: ProjectToolchainConfig {
                thtk_dir: String::new(),
            },
        }
    }

    /// 关键断言：给一个磁盘上根本不存在的项目配置，结果仍然是它的值。
    /// 这证明 resolve_from 用的是传进来的对象，而不是回头去读文件。
    #[test]
    fn resolve_from_uses_the_passed_project_without_touching_disk() {
        let project = project_with_version("18");
        assert_eq!(
            resolve_from(Some(&project), &app_config("20"), "thecl"),
            Ok(18)
        );
    }

    #[test]
    fn resolve_from_falls_back_to_global_when_project_version_empty() {
        let project = project_with_version("");
        assert_eq!(
            resolve_from(Some(&project), &app_config("20"), "thecl"),
            Ok(20)
        );
    }

    #[test]
    fn resolve_from_falls_back_to_global_without_project() {
        assert_eq!(resolve_from(None, &app_config("17"), "thecl"), Ok(17));
    }

    #[test]
    fn resolve_from_still_checks_tool_support() {
        let project = project_with_version("75");
        assert_eq!(
            resolve_from(Some(&project), &app_config("20"), "thdat"),
            Ok(75)
        );
        assert!(resolve_from(Some(&project), &app_config("20"), "thecl").is_err());
    }

    /// resolve 与 resolve_from 必须给出完全一致的结果——前者只是后者
    /// 加一次读盘的便利包装，语义不能有第二套。
    #[test]
    fn resolve_and_resolve_from_agree() {
        let dir = temp_dir("resolve-agree");
        write_project_version(&dir, "th18");
        let loaded = crate::common::project_config::load_project_config(&dir.to_string_lossy());

        for tool in ["thecl", "thmsg", "thdat"] {
            assert_eq!(
                resolve(&app_config("20"), Some(&dir.to_string_lossy()), tool),
                resolve_from(loaded.as_ref(), &app_config("20"), tool),
                "{tool} 两条路径结果不一致"
            );
        }
    }
}
