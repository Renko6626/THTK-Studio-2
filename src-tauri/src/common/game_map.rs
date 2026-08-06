//! gamemap：把游戏版本映射到共享的 mapfile。
//!
//! 格式取自 [truth](https://github.com/ExpHP/truth) 的 `map/any.msgm`，
//! 这样我们的表和它的表可以互换：
//!
//! ```text
//! !gamemap
//! !game_files
//! 17  th14.stdm
//! ```
//!
//! 存在的意义是把「哪些版本共用哪份表」变成**数据**。此前两个 map_parser 都写着
//! `let _ = version;`——任何版本都返回 th17 的表。结果碰巧对（thstd 的 th14–th20
//! 共用 `formats_v2`），但理由是错的：它靠"只有一份数据"蒙对，将来 ZUN 分了新档
//! 会继续静默返回旧表。

use regex::Regex;
use std::sync::OnceLock;

fn entry_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\d+)\s+(\S+)").expect("valid regex"))
}

/// 查不到返回 `None`——**不做任何回退**。
/// 调用方必须把"这个版本没有数据"如实告诉用户，而不是拿别的版本的表顶上。
pub fn resolve_map_file(gamemap: &str, version: u32) -> Option<String> {
    for raw in gamemap.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some(caps) = entry_regex().captures(line) {
            if caps[1].parse::<u32>().ok() == Some(version) {
                return Some(caps[2].to_string());
            }
        }
    }
    None
}

/// gamemap 里列出的全部版本，供 UI 展示"哪些版本有语义数据"。
pub fn mapped_versions(gamemap: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for raw in gamemap.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some(caps) = entry_regex().captures(line) {
            if let Ok(v) = caps[1].parse::<u32>() {
                out.push(v);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANY_STDM: &str = "\
!gamemap
!game_files

# 注释行应被跳过
6   th06.stdm
14  th14.stdm
17  th14.stdm
185 th14.stdm
# NEWHU: 185
";

    #[test]
    fn maps_version_to_shared_file() {
        assert_eq!(resolve_map_file(ANY_STDM, 17).as_deref(), Some("th14.stdm"));
        assert_eq!(resolve_map_file(ANY_STDM, 185).as_deref(), Some("th14.stdm"));
        assert_eq!(resolve_map_file(ANY_STDM, 6).as_deref(), Some("th06.stdm"));
    }

    /// 关键：查不到就是查不到，**不许**回退到任意一份表。
    /// 这正是 `let _ = version;` 的病根——靠只有一份数据蒙对。
    #[test]
    fn unmapped_version_returns_none_instead_of_guessing() {
        assert_eq!(resolve_map_file(ANY_STDM, 20), None);
        assert_eq!(resolve_map_file(ANY_STDM, 99), None);
    }

    #[test]
    fn ignores_comments_and_section_headers() {
        // `!gamemap` / `!game_files` / `# NEWHU` 都不能被当成条目
        assert_eq!(mapped_versions(ANY_STDM), vec![6, 14, 17, 185]);
    }

    #[test]
    fn empty_gamemap_maps_nothing() {
        assert_eq!(resolve_map_file("", 17), None);
        assert!(mapped_versions("!gamemap\n").is_empty());
    }
}
