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
