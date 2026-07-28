use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 最近项目列表上限，超出时淘汰最旧的一条。
pub const MAX_RECENT_PROJECTS: usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    /// 平台原生绝对路径
    pub path: String,
    /// 目录名，供列表显示
    pub name: String,
    /// Unix 毫秒时间戳
    pub last_opened_at: u64,
}

/// 跨分隔符比较用的规范化，与前端 `utils/pathNormalize.js` 的 `normalizePath` 同语义：
/// 反斜杠转正斜杠、去掉尾部分隔符（盘根除外）。
///
/// 刻意不做大小写折叠——前端的 `pathsEqual` 也没做。Windows 路径大小写不敏感，
/// 严格说两边都应该折叠，但那要连前端一起改，否则同一路径在两侧的判定会不一致。
fn normalize_for_compare(path: &str) -> String {
    let mut result = path.replace('\\', "/");

    if result.len() > 1 && result.ends_with('/') {
        let bytes = result.as_bytes();
        // "C:/" 这样的盘根要保留尾部斜杠
        let is_drive_root = result.len() == 3 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic();
        if !is_drive_root {
            result.pop();
        }
    }

    result
}

/// 取路径末段作为显示名，取不到时回退到整个路径
pub fn project_name(path: &str) -> String {
    let normalized = normalize_for_compare(path);
    normalized
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(&normalized)
        .to_string()
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 记录一次打开：同路径去重后置顶，并截断到上限。
pub fn record(list: &[RecentProject], path: &str, opened_at: u64) -> Vec<RecentProject> {
    let key = normalize_for_compare(path);

    let mut next = Vec::with_capacity(list.len() + 1);
    next.push(RecentProject {
        path: path.to_string(),
        name: project_name(path),
        last_opened_at: opened_at,
    });
    next.extend(
        list.iter()
            .filter(|item| normalize_for_compare(&item.path) != key)
            .cloned(),
    );
    next.truncate(MAX_RECENT_PROJECTS);
    next
}

/// 移除一条。
///
/// 列表读取时**不**自动剔除失效路径：磁盘临时离线（U 盘、网络盘未挂载）不该让
/// 记录消失。前端负责把不可用的项标出来，删不删由用户决定。
pub fn remove(list: &[RecentProject], path: &str) -> Vec<RecentProject> {
    let key = normalize_for_compare(path);
    list.iter()
        .filter(|item| normalize_for_compare(&item.path) != key)
        .cloned()
        .collect()
}

/// 按最近打开时间降序。record 本身就是置顶插入，这里是为了容忍手工编辑过的 settings.json。
pub fn sorted(list: &[RecentProject]) -> Vec<RecentProject> {
    let mut result = list.to_vec();
    result.sort_by(|a, b| b.last_opened_at.cmp(&a.last_opened_at));
    result
}

/// 列表项的展示形态：在存储结构上补一个"路径当前是否可用"。
///
/// `available` 不落盘，每次读取现算——U 盘拔了、网络盘没挂上都是暂时的，
/// 把它写进配置会让状态过期。
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecentProjectView {
    pub path: String,
    pub name: String,
    pub last_opened_at: u64,
    pub available: bool,
}

pub fn to_views(list: &[RecentProject]) -> Vec<RecentProjectView> {
    list.iter()
        .map(|item| RecentProjectView {
            path: item.path.clone(),
            name: item.name.clone(),
            last_opened_at: item.last_opened_at,
            available: Path::new(&item.path).is_dir(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, at: u64) -> RecentProject {
        RecentProject {
            path: path.to_string(),
            name: project_name(path),
            last_opened_at: at,
        }
    }

    #[test]
    fn record_puts_newest_first() {
        let list = record(&[], "/projects/a", 100);
        let list = record(&list, "/projects/b", 200);

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].path, "/projects/b");
        assert_eq!(list[1].path, "/projects/a");
    }

    #[test]
    fn record_dedupes_and_moves_to_top_updating_time() {
        let list = record(&[], "/projects/a", 100);
        let list = record(&list, "/projects/b", 200);
        let list = record(&list, "/projects/a", 300);

        assert_eq!(list.len(), 2, "重复打开不应新增条目");
        assert_eq!(list[0].path, "/projects/a");
        assert_eq!(list[0].last_opened_at, 300);
        assert_eq!(list[1].path, "/projects/b");
    }

    #[test]
    fn dedupe_ignores_separator_and_trailing_slash_differences() {
        let list = record(&[], r"D:\projects\th18", 100);
        let list = record(&list, "D:/projects/th18/", 200);

        assert_eq!(list.len(), 1, "同一路径的不同写法应视为一条");
        assert_eq!(list[0].last_opened_at, 200);
    }

    #[test]
    fn caps_at_max_and_evicts_oldest() {
        let mut list = Vec::new();
        for index in 0..(MAX_RECENT_PROJECTS + 3) {
            list = record(&list, &format!("/projects/p{index}"), 100 + index as u64);
        }

        assert_eq!(list.len(), MAX_RECENT_PROJECTS);
        assert_eq!(list[0].path, format!("/projects/p{}", MAX_RECENT_PROJECTS + 2));
        // 最早的三个被淘汰
        assert!(!list.iter().any(|item| item.path == "/projects/p0"));
        assert!(!list.iter().any(|item| item.path == "/projects/p2"));
        assert!(list.iter().any(|item| item.path == "/projects/p3"));
    }

    #[test]
    fn remove_matches_across_separators_and_leaves_others() {
        let list = vec![entry(r"D:\a", 100), entry("/b", 200)];

        let after = remove(&list, "D:/a");
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].path, "/b");

        // 不存在的路径不改变列表
        assert_eq!(remove(&after, "/nope").len(), 1);
    }

    #[test]
    fn sorted_repairs_hand_edited_order() {
        let list = vec![entry("/a", 100), entry("/c", 300), entry("/b", 200)];
        let ordered = sorted(&list);

        assert_eq!(ordered[0].path, "/c");
        assert_eq!(ordered[1].path, "/b");
        assert_eq!(ordered[2].path, "/a");
    }

    #[test]
    fn views_flag_missing_paths_without_dropping_them() {
        let dir = tempfile::tempdir().expect("tempdir");
        let existing = dir.path().to_string_lossy().to_string();

        let list = vec![entry(&existing, 200), entry("/definitely/not/here", 100)];
        let views = to_views(&list);

        assert_eq!(views.len(), 2, "失效路径不应被剔除，只标记不可用");
        assert!(views[0].available);
        assert!(!views[1].available);
    }

    #[test]
    fn project_name_takes_last_segment() {
        assert_eq!(project_name(r"D:\projects\th18"), "th18");
        assert_eq!(project_name("/projects/th18/"), "th18");
        assert_eq!(project_name("th18"), "th18");
        // 盘根取到盘符本身
        assert_eq!(project_name("D:/"), "D:");
    }
}
