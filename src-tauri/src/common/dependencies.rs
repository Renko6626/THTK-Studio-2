//! 依赖状态（只读）与内置 ECL map 的释放。
//!
//! 生产环境使用 `ProjectDirs::data_dir()`（不是 config 目录）：
//!
//! ```text
//! <data_dir>/
//!   deps/
//!     maps/
//!       bundled/th{version}.eclm   # 从二进制内嵌基线释放出来的
//! ```
//!
//! M0/M1 只做两件事：
//!
//! - 把 `assets/maps/ecl/` 里内嵌的 zero318 基线幂等释放到 `deps/maps/bundled/`，
//!   让 thecl 与编辑器都能拿到真实文件路径；
//! - 汇总 thtk 手动配置与 ECL map 覆盖情况，供设置界面的只读依赖卡片使用。
//!
//! 不做下载、不做安装：`deps/manifest.json`、thtk nightly 安装与 map 快照
//! 属于 M2/M3，不在本模块实现。状态也不写入 `AppConfig`，避免改动既有 wire format。

use crate::common::toolchain;
use crate::config::AppConfig;
use directories::ProjectDirs;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// 进程内临时文件后缀序列号。
///
/// 只用 PID 不够：setup、设置对话框与 semantic-loader 都可能在同一个进程里
/// 并发调用 `ensure_bundled_maps`，同名临时文件会被互相覆盖，先完成的一方
/// rename 后另一方再 rename 就会 ENOENT（或在 Windows 上读到半截内容）。
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 内置 ECL map 的来源 commit（zero318/TouhouMaps）。
pub const ECL_MAP_SOURCE_COMMIT: &str = "0673f4221f5e975842162e07cd7b765fc7570ebd";
/// zero318/TouhouMaps 的许可证。
pub const ECL_MAP_LICENSE: &str = "Unlicense";
/// thtk 的许可证说明。thtk 不是 GPL，随包/下载分发时保留 `COPYING` 即可。
pub const THTK_LICENSE: &str = "BSD-style (retain COPYING)";

const DEPENDENCY_DIR_NAME: &str = "deps";
const MAPS_DIR_NAME: &str = "maps";
const BUNDLED_DIR_NAME: &str = "bundled";

pub(crate) struct BundledEclMap {
    pub version: u32,
    pub file_name: &'static str,
    pub content: &'static str,
}

/// 随二进制分发的零基线。只收录上游在固定 commit 上确实提供的版本；
/// th13/th14/th128/th143/th16/th165/th17 没有对应文件，不放假数据。
pub(crate) const BUNDLED_ECL_MAPS: [BundledEclMap; 15] = [
    BundledEclMap {
        version: 6,
        file_name: "th06.eclm",
        content: include_str!("../../assets/maps/ecl/th06.eclm"),
    },
    BundledEclMap {
        version: 7,
        file_name: "th07.eclm",
        content: include_str!("../../assets/maps/ecl/th07.eclm"),
    },
    BundledEclMap {
        version: 8,
        file_name: "th08.eclm",
        content: include_str!("../../assets/maps/ecl/th08.eclm"),
    },
    BundledEclMap {
        version: 9,
        file_name: "th09.eclm",
        content: include_str!("../../assets/maps/ecl/th09.eclm"),
    },
    BundledEclMap {
        version: 95,
        file_name: "th095.eclm",
        content: include_str!("../../assets/maps/ecl/th095.eclm"),
    },
    BundledEclMap {
        version: 10,
        file_name: "th10.eclm",
        content: include_str!("../../assets/maps/ecl/th10.eclm"),
    },
    BundledEclMap {
        version: 103,
        file_name: "th103.eclm",
        content: include_str!("../../assets/maps/ecl/th103.eclm"),
    },
    BundledEclMap {
        version: 11,
        file_name: "th11.eclm",
        content: include_str!("../../assets/maps/ecl/th11.eclm"),
    },
    BundledEclMap {
        version: 12,
        file_name: "th12.eclm",
        content: include_str!("../../assets/maps/ecl/th12.eclm"),
    },
    BundledEclMap {
        version: 125,
        file_name: "th125.eclm",
        content: include_str!("../../assets/maps/ecl/th125.eclm"),
    },
    BundledEclMap {
        version: 15,
        file_name: "th15.eclm",
        content: include_str!("../../assets/maps/ecl/th15.eclm"),
    },
    BundledEclMap {
        version: 18,
        file_name: "th18.eclm",
        content: include_str!("../../assets/maps/ecl/th18.eclm"),
    },
    BundledEclMap {
        version: 185,
        file_name: "th185.eclm",
        content: include_str!("../../assets/maps/ecl/th185.eclm"),
    },
    BundledEclMap {
        version: 19,
        file_name: "th19.eclm",
        content: include_str!("../../assets/maps/ecl/th19.eclm"),
    },
    BundledEclMap {
        version: 20,
        file_name: "th20.eclm",
        content: include_str!("../../assets/maps/ecl/th20.eclm"),
    },
];

/// 依赖来源。M0/M1 只会产生 `None` / `Manual` / `Bundled`。
/// M2 的托管 thtk 与 map 快照需要新取值时在这里补，前端
/// `src/types/dependencies.ts` 的联合类型与 wire test 一并更新。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencySource {
    None,
    Manual,
    Bundled,
}

/// thtk 的手动配置状态。
///
/// `available` 只检查解析出的 `thecl.exe` 是否存在，不启动子进程——
/// 真正的版本探测在 `toolchain::get_toolchain_status` 里，设置界面的
/// 工具链总览已经展示；依赖卡片不该为一行摘要再启动两个进程。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThtkDependencyStatus {
    pub source: DependencySource,
    pub configured: bool,
    pub available: bool,
    pub thtk_dir: String,
    pub resolved_path: String,
    pub message: String,
    pub license: String,
}

/// 托管 ECL map 的覆盖情况。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapDependencyStatus {
    pub source: DependencySource,
    pub bundled_dir: String,
    /// 已释放且可用的版本，按 `BUNDLED_ECL_MAPS` 的顺序。
    pub covered_versions: Vec<u32>,
    /// thecl 支持、但没有内置 map 的版本，按 `game_version` 表顺序。
    pub missing_versions: Vec<u32>,
    pub source_commit: String,
    pub license: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub thtk: ThtkDependencyStatus,
    pub maps: MapDependencyStatus,
    pub warnings: Vec<String>,
}

/// 依赖根目录（`deps/`）的持有者。
///
/// 生产用 [`DependencyManager::for_current_user`]，测试用
/// [`DependencyManager::with_base_dir`] 指到临时目录——本模块所有落盘
/// 操作都经过 base dir，不在内部读取用户目录。
#[derive(Debug, Clone)]
pub struct DependencyManager {
    base_dir: PathBuf,
}

impl DependencyManager {
    /// 生产环境：`<ProjectDirs::data_dir()>/deps`。
    pub fn for_current_user() -> Result<Self, String> {
        let dirs = ProjectDirs::from("com", "abl", "thtk-studio")
            .ok_or_else(|| "无法确定应用数据目录（ProjectDirs 不可用）".to_string())?;
        Ok(Self::with_base_dir(dirs.data_dir()))
    }

    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn deps_dir(&self) -> PathBuf {
        self.base_dir.join(DEPENDENCY_DIR_NAME)
    }

    pub fn bundled_maps_dir(&self) -> PathBuf {
        self.deps_dir().join(MAPS_DIR_NAME).join(BUNDLED_DIR_NAME)
    }

    /// 把内嵌基线写到 `deps/maps/bundled/th{version}.eclm`。
    ///
    /// 幂等：内容一致的文件不重写；不一致（或缺失）时先写临时文件再 rename，
    /// 避免写到一半留下半截 map。任何一步失败都带上是哪个文件、哪个路径。
    pub fn ensure_bundled_maps(&self) -> Result<(), String> {
        let dir = self.bundled_maps_dir();
        fs::create_dir_all(&dir)
            .map_err(|error| format!("创建内置 map 目录失败 {}: {error}", dir.display()))?;

        for map in BUNDLED_ECL_MAPS.iter() {
            let target = dir.join(map.file_name);
            if fs::read_to_string(&target).is_ok_and(|existing| existing == map.content) {
                continue;
            }

            // PID + 进程内序列号：并发调用各自拿到不同临时名，rename 不会
            // 互相踩到同一路径。
            let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let temp = target.with_extension(format!("eclm.tmp{}-{sequence}", std::process::id()));
            if let Err(error) = fs::write(&temp, map.content) {
                let _ = fs::remove_file(&temp);
                return Err(format!(
                    "写入内置 map 失败 {}（{}）: {error}",
                    temp.display(),
                    map.file_name
                ));
            }
            if let Err(error) = fs::rename(&temp, &target) {
                let _ = fs::remove_file(&temp);
                return Err(format!(
                    "替换内置 map 失败 {}（{}）: {error}",
                    target.display(),
                    map.file_name
                ));
            }
        }

        Ok(())
    }

    /// 某个版本的托管 map 路径。
    ///
    /// M0/M1 只查 `deps/maps/bundled/`。M2 的下载快照
    /// （`deps/maps/snapshots/<sha>/`）也应该在这里按优先级接入，
    /// 但下载、校验与切换不在本模块。
    pub fn managed_map_path(&self, version: &str) -> Option<PathBuf> {
        let dir = self.bundled_maps_dir();
        map_file_names(version)
            .into_iter()
            .map(|name| dir.join(name))
            .find(|candidate| candidate.is_file())
    }

    /// 只读状态汇总。不会写盘——调用方若需要 bundled 目录真实存在，
    /// 先调用 [`DependencyManager::ensure_bundled_maps`]。
    pub fn status(&self, config: &AppConfig) -> DependencyStatus {
        let thtk = self.thtk_status(config);
        let maps = self.maps_status();

        let mut warnings = Vec::new();
        if !thtk.configured {
            warnings.push(
                "未配置 thtk：请在「工具链设置」里指定目录；托管安装/更新将在后续版本实现。"
                    .to_string(),
            );
        } else if !thtk.available {
            warnings.push(format!(
                "thtk 路径已配置，但未找到 {}。",
                thtk.resolved_path
            ));
        }

        if maps.covered_versions.is_empty() {
            warnings.push("内置 ECL map 尚未释放到数据目录。".to_string());
        } else if !maps.missing_versions.is_empty() {
            warnings.push(format!(
                "内置 map 未覆盖 {}：在这些版本上需要手动指定 mapPaths。",
                format_version_list(&maps.missing_versions)
            ));
        }

        DependencyStatus {
            thtk,
            maps,
            warnings,
        }
    }

    fn thtk_status(&self, config: &AppConfig) -> ThtkDependencyStatus {
        let override_path = toolchain::resolve_tool_override(config, "thecl");
        let thtk_dir = config.thtk_dir.trim().to_string();
        let configured = !override_path.is_empty() || !thtk_dir.is_empty();
        let resolved_path = toolchain::resolve_tool_path(config, "thecl", "thecl.exe");
        let available = !resolved_path.is_empty() && Path::new(&resolved_path).is_file();

        let (source, message) = if !configured {
            (
                DependencySource::None,
                toolchain::not_configured_message("thecl"),
            )
        } else if available {
            (DependencySource::Manual, "已找到 thecl.exe".to_string())
        } else {
            (
                DependencySource::Manual,
                format!("已配置 {resolved_path}，但该文件不存在"),
            )
        };

        ThtkDependencyStatus {
            source,
            configured,
            available,
            thtk_dir,
            resolved_path,
            message,
            license: THTK_LICENSE.to_string(),
        }
    }

    fn maps_status(&self) -> MapDependencyStatus {
        let bundled_dir = self.bundled_maps_dir();
        let covered_versions: Vec<u32> = BUNDLED_ECL_MAPS
            .iter()
            .filter(|map| bundled_dir.join(map.file_name).is_file())
            .map(|map| map.version)
            .collect();

        // 以「thecl 支持的版本」为全集：th01–05 之类没有 ECL 的作品不该被标成缺失。
        let missing_versions = crate::common::game_version::versions_for_tool("thecl")
            .into_iter()
            .filter(|version| !covered_versions.contains(version))
            .collect();

        let source = if covered_versions.is_empty() {
            DependencySource::None
        } else {
            DependencySource::Bundled
        };

        MapDependencyStatus {
            source,
            bundled_dir: bundled_dir.to_string_lossy().to_string(),
            covered_versions,
            missing_versions,
            source_commit: ECL_MAP_SOURCE_COMMIT.to_string(),
            license: ECL_MAP_LICENSE.to_string(),
        }
    }
}

/// 某个版本可能对应的 map 文件名，按优先级排列。
///
/// 覆盖三种写法：`th{version}.eclm`、`{version}.eclm`、`{version}.eclmap`。
/// 内置基线沿用上游的补零命名（th06–th09、th095），所以补零形式也要生成——
/// 否则配置里写的 `6` 永远找不到 `th06.eclm`。解析不出已知版本时返回空列表，
/// 不拿可疑值去拼文件路径。
pub(crate) fn map_file_names(version: &str) -> Vec<String> {
    let Ok(id) = crate::common::game_version::parse(version) else {
        return Vec::new();
    };

    let mut names: Vec<String> = Vec::new();
    let mut push = |name: String| {
        if !names.contains(&name) {
            names.push(name);
        }
    };

    // 内置基线的规范名（可能带补零）
    if let Some(map) = BUNDLED_ECL_MAPS.iter().find(|map| map.version == id) {
        push(map.file_name.to_string());
    }

    let stems: Vec<String> = if id < 10 {
        vec![id.to_string(), format!("{id:02}")]
    } else {
        vec![id.to_string()]
    };
    for stem in stems {
        push(format!("th{stem}.eclm"));
        push(format!("{stem}.eclm"));
        push(format!("{stem}.eclmap"));
    }

    names
}

fn format_version_list(versions: &[u32]) -> String {
    versions
        .iter()
        .map(|version| format!("th{version}"))
        .collect::<Vec<_>>()
        .join(" / ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn manager(name: &str) -> (TempDir, DependencyManager) {
        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir {name}: {error}"));
        let manager = DependencyManager::with_base_dir(dir.path());
        (dir, manager)
    }

    #[test]
    fn releases_all_bundled_maps_byte_identically() {
        let (_dir, manager) = manager("release");
        manager.ensure_bundled_maps().expect("释放内置 map");

        assert_eq!(
            manager.bundled_maps_dir(),
            manager.deps_dir().join("maps").join("bundled")
        );

        for map in BUNDLED_ECL_MAPS.iter() {
            let released = std::fs::read_to_string(manager.bundled_maps_dir().join(map.file_name))
                .unwrap_or_else(|error| panic!("读取 {}: {error}", map.file_name));
            assert_eq!(
                released, map.content,
                "{} 的内容必须原样保留",
                map.file_name
            );
        }

        let count = std::fs::read_dir(manager.bundled_maps_dir())
            .expect("bundled 目录应存在")
            .count();
        assert_eq!(count, BUNDLED_ECL_MAPS.len());
    }

    #[test]
    fn ensure_bundled_maps_is_idempotent() {
        let (_dir, manager) = manager("idempotent");
        manager.ensure_bundled_maps().expect("第一次释放");
        manager.ensure_bundled_maps().expect("第二次释放");

        let released =
            std::fs::read_to_string(manager.bundled_maps_dir().join("th20.eclm")).expect("th20");
        assert_eq!(released, include_str!("../../assets/maps/ecl/th20.eclm"));
    }

    #[test]
    fn ensure_bundled_maps_repairs_a_corrupted_release() {
        let (_dir, manager) = manager("repair");
        manager.ensure_bundled_maps().expect("首次释放");

        let th20 = manager.bundled_maps_dir().join("th20.eclm");
        let untouched_path = manager.bundled_maps_dir().join("th18.eclm");
        let untouched = std::fs::read_to_string(&untouched_path).expect("读取 th18");
        std::fs::write(&th20, b"stale bundled map").expect("写坏 th20");

        manager.ensure_bundled_maps().expect("再次释放");

        assert_eq!(
            std::fs::read_to_string(&th20).expect("读取修复后的 th20"),
            include_str!("../../assets/maps/ecl/th20.eclm"),
            "内容不一致的已释放文件必须被重新写入"
        );
        assert_eq!(
            std::fs::read_to_string(&untouched_path).expect("读取 th18"),
            untouched,
            "其他已释放文件不应被重写"
        );
        assert_eq!(
            std::fs::read_dir(manager.bundled_maps_dir())
                .expect("bundled 目录应存在")
                .count(),
            BUNDLED_ECL_MAPS.len(),
            "不应留下临时文件"
        );
    }

    #[test]
    fn ensure_reports_the_failing_path_when_the_directory_cannot_be_created() {
        let dir = tempfile::tempdir().expect("tempdir");
        // deps/maps 被一个普通文件占位，create_dir_all 必然失败
        std::fs::create_dir_all(dir.path().join("deps")).expect("mkdir deps");
        std::fs::write(dir.path().join("deps").join("maps"), b"not a directory").expect("write");

        let manager = DependencyManager::with_base_dir(dir.path());
        let error = manager.ensure_bundled_maps().unwrap_err();

        assert!(
            error.contains("内置 map"),
            "错误应说明失败原因, got: {error}"
        );
        assert!(error.contains("maps"), "错误应带上路径, got: {error}");
    }

    #[test]
    fn status_reports_coverage_and_missing_versions() {
        let (_dir, manager) = manager("coverage");
        manager.ensure_bundled_maps().expect("释放内置 map");

        let status = manager.status(&AppConfig::default());

        assert_eq!(
            status.maps.covered_versions,
            vec![6, 7, 8, 9, 95, 10, 103, 11, 12, 125, 15, 18, 185, 19, 20]
        );
        assert_eq!(
            status.maps.missing_versions,
            vec![128, 13, 14, 143, 16, 165, 17]
        );
        assert_eq!(status.maps.source, DependencySource::Bundled);
        assert_eq!(status.maps.license, "Unlicense");
        assert_eq!(status.maps.source_commit, ECL_MAP_SOURCE_COMMIT);
        assert_eq!(
            status.maps.bundled_dir,
            manager.bundled_maps_dir().to_string_lossy()
        );
        assert!(
            status
                .warnings
                .iter()
                .any(|warning| warning.contains("th128")),
            "缺失版本应出现在 warnings 里, got: {:?}",
            status.warnings
        );
    }

    #[test]
    fn a_missing_released_file_is_not_reported_as_covered() {
        let (_dir, manager) = manager("partial");
        manager.ensure_bundled_maps().expect("释放内置 map");
        std::fs::remove_file(manager.bundled_maps_dir().join("th18.eclm")).expect("remove th18");

        let status = manager.status(&AppConfig::default());

        assert!(!status.maps.covered_versions.contains(&18));
        assert!(status.maps.missing_versions.contains(&18));
        assert_eq!(status.maps.source, DependencySource::Bundled);
    }

    #[test]
    fn status_without_release_reports_no_coverage() {
        let (_dir, manager) = manager("empty");

        let status = manager.status(&AppConfig::default());

        assert!(status.maps.covered_versions.is_empty());
        assert_eq!(status.maps.source, DependencySource::None);
        assert_eq!(
            status.maps.missing_versions,
            crate::common::game_version::versions_for_tool("thecl")
        );
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("尚未释放")));
    }

    #[test]
    fn managed_map_path_accepts_prefixed_padded_and_plain_versions() {
        let (_dir, manager) = manager("managed-path");
        manager.ensure_bundled_maps().expect("释放内置 map");

        let th18 = manager.bundled_maps_dir().join("th18.eclm");
        for raw in ["18", "th18", "TH18", "  th18  "] {
            assert_eq!(manager.managed_map_path(raw), Some(th18.clone()), "{raw}");
        }

        // 6 与 95 的内置文件是上游的补零命名
        assert_eq!(
            manager.managed_map_path("6"),
            Some(manager.bundled_maps_dir().join("th06.eclm"))
        );
        assert_eq!(
            manager.managed_map_path("th95"),
            Some(manager.bundled_maps_dir().join("th095.eclm"))
        );

        assert_eq!(
            manager.managed_map_path("13"),
            None,
            "未内置的版本不能假装有"
        );
        assert_eq!(manager.managed_map_path("bogus"), None);
        assert_eq!(manager.managed_map_path(""), None);
    }

    #[test]
    fn thtk_status_is_derived_from_manual_config_without_spawning() {
        let (dir, manager) = manager("thtk");
        let thtk_dir = dir.path().join("thtk");
        std::fs::create_dir_all(&thtk_dir).expect("mkdir thtk");
        std::fs::write(thtk_dir.join("thecl.exe"), b"").expect("fake thecl");

        let config = AppConfig {
            thtk_dir: thtk_dir.to_string_lossy().to_string(),
            ..AppConfig::default()
        };
        let status = manager.status(&config);

        assert!(status.thtk.configured);
        assert!(status.thtk.available);
        assert_eq!(status.thtk.source, DependencySource::Manual);
        assert_eq!(status.thtk.thtk_dir, config.thtk_dir);
        assert_eq!(status.thtk.license, THTK_LICENSE);
        assert!(!status
            .warnings
            .iter()
            .any(|warning| warning.contains("未配置 thtk")));
    }

    #[test]
    fn unconfigured_thtk_is_reported_as_missing() {
        let (_dir, manager) = manager("no-thtk");

        let status = manager.status(&AppConfig::default());

        assert!(!status.thtk.configured);
        assert!(!status.thtk.available);
        assert_eq!(status.thtk.source, DependencySource::None);
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("未配置 thtk")));
    }

    #[test]
    fn map_file_names_cover_all_three_spellings() {
        let names = map_file_names("18");
        assert!(names.contains(&"th18.eclm".to_string()));
        assert!(names.contains(&"18.eclm".to_string()));
        assert!(names.contains(&"18.eclmap".to_string()));
        // eclm 先于 eclmap：内置基线是 eclm
        assert!(
            names.iter().position(|name| name == "18.eclm")
                < names.iter().position(|name| name == "18.eclmap")
        );

        let names = map_file_names("6");
        assert_eq!(names.first().map(String::as_str), Some("th06.eclm"));
        assert!(names.contains(&"6.eclm".to_string()));

        assert!(map_file_names("bogus").is_empty());
        assert!(map_file_names("18.5").is_empty());
        assert!(map_file_names("").is_empty());
    }
}
