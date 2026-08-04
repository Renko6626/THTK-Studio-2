use crate::common::cmd_runner;
use crate::config::AppConfig;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct ToolchainDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub exe_name: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainStatus {
    pub tool: String,
    pub label: String,
    pub exe_name: String,
    pub configured_path: String,
    pub resolved_path: String,
    pub available: bool,
    pub version: String,
    pub message: String,
    /// 该工具实际可用的版本集合（运行时探测 ∩ 静态表；探测失败则为静态表）。
    pub supported_versions: Vec<u32>,
}

pub const TOOLCHAIN_DESCRIPTORS: [ToolchainDescriptor; 5] = [
    ToolchainDescriptor {
        id: "thecl",
        label: "Enemy Script Compiler",
        exe_name: "thecl.exe",
    },
    ToolchainDescriptor {
        id: "thmsg",
        label: "Message Script Tool",
        exe_name: "thmsg.exe",
    },
    ToolchainDescriptor {
        id: "thanm",
        label: "Animation Tool",
        exe_name: "thanm.exe",
    },
    ToolchainDescriptor {
        id: "thstd",
        label: "Stage Data Tool",
        exe_name: "thstd.exe",
    },
    ToolchainDescriptor {
        id: "thdat",
        label: "Archive Tool",
        exe_name: "thdat.exe",
    },
];

pub fn find_toolchain_descriptor(tool_id: &str) -> Option<&'static ToolchainDescriptor> {
    TOOLCHAIN_DESCRIPTORS.iter().find(|descriptor| descriptor.id == tool_id)
}

pub fn resolve_tool_override(config: &AppConfig, tool_id: &str) -> String {
    if let Some(path) = config.tool_overrides.get(tool_id) {
        if !path.trim().is_empty() {
            return path.trim().to_string();
        }
    }

    if tool_id == "thecl" && !config.thecl_path.trim().is_empty() {
        return config.thecl_path.trim().to_string();
    }

    String::new()
}

/// 应用项目级覆盖，得到本次调用真正生效的配置。
///
/// 目前只覆盖 thtk_dir：`.thtk-project.json` 的 `toolchain.thtkDir` 非空时顶掉全局值。
/// 单个工具的显式覆盖（`tool_overrides` / `thecl_path`）优先级仍然更高——那是用户
/// 对某个 exe 的精确指定，不应该被项目级的目录设置顶掉，`resolve_tool_path` 里
/// 的顺序已经保证了这一点。
///
/// 所有会调起外部工具或展示工具链状态的路径都应该先过这个函数，否则项目配置里的
/// thtkDir 就只是写进 JSON 却无人读取的死数据。
pub fn effective_config(config: &AppConfig, project_root: Option<&str>) -> AppConfig {
    effective_context(config, project_root).config
}

/// 一次读盘拿到本次调用需要的**全部**项目上下文。
///
/// 工具链命令此前读两次 `.thtk-project.json`：一次给 `effective_config` 解析
/// thtkDir，一次给 `game_version::resolve` 取版本。除了多一次 IO，两次读取
/// 之间文件若被改动，同一次调用还会用上两份不一致的配置。
///
/// 配合 [`crate::common::game_version::resolve_from`] 使用。
pub struct EffectiveContext {
    /// 应用过项目级覆盖的配置
    pub config: AppConfig,
    /// 本次读到的项目配置；没有项目根或文件不存在时为 None
    pub project: Option<crate::common::project_config::ProjectConfig>,
}

pub fn effective_context(config: &AppConfig, project_root: Option<&str>) -> EffectiveContext {
    let Some(root) = project_root else {
        return EffectiveContext {
            config: config.clone(),
            project: None,
        };
    };
    let Some(project) = crate::common::project_config::load_project_config(root) else {
        return EffectiveContext {
            config: config.clone(),
            project: None,
        };
    };

    let project_thtk_dir = project.toolchain.thtk_dir.trim();
    if project_thtk_dir.is_empty() {
        return EffectiveContext {
            config: config.clone(),
            project: Some(project),
        };
    }

    let mut effective = config.clone();
    effective.thtk_dir = project_thtk_dir.to_string();
    EffectiveContext {
        config: effective,
        project: Some(project),
    }
}

pub fn resolve_tool_path(config: &AppConfig, tool_id: &str, exe_name: &str) -> String {
    let override_path = resolve_tool_override(config, tool_id);
    if !override_path.is_empty() {
        return override_path;
    }

    if config.thtk_dir.trim().is_empty() {
        return String::new();
    }

    PathBuf::from(&config.thtk_dir)
        .join(exe_name)
        .to_string_lossy()
        .to_string()
}

pub fn get_toolchain_status(config: &AppConfig, tool_id: &str) -> Result<ToolchainStatus, String> {
    let descriptor = find_toolchain_descriptor(tool_id)
        .ok_or_else(|| format!("Unsupported toolchain '{}'", tool_id))?;

    let resolved_path = resolve_tool_path(config, descriptor.id, descriptor.exe_name);
    let configured_path = {
        let override_path = resolve_tool_override(config, descriptor.id);
        if !override_path.is_empty() {
            override_path
        } else {
            config.thtk_dir.trim().to_string()
        }
    };

    if resolved_path.trim().is_empty() {
        return Ok(ToolchainStatus {
            tool: descriptor.id.to_string(),
            label: descriptor.label.to_string(),
            exe_name: descriptor.exe_name.to_string(),
            configured_path,
            resolved_path,
            available: false,
            version: String::new(),
            message: "Toolchain path is not configured".to_string(),
            supported_versions: Vec::new(),
        });
    }

    match query_tool_version(&resolved_path) {
        Ok(version) => {
            // 必须在结构体字面量之前算：字面量里 resolved_path 字段会先被移动。
            let supported_versions = probe_supported_versions(&resolved_path, descriptor.id);
            Ok(ToolchainStatus {
                tool: descriptor.id.to_string(),
                label: descriptor.label.to_string(),
                exe_name: descriptor.exe_name.to_string(),
                configured_path,
                resolved_path,
                available: true,
                version,
                message: "Toolchain is available".to_string(),
                supported_versions,
            })
        }
        Err(error) => Ok(ToolchainStatus {
            tool: descriptor.id.to_string(),
            label: descriptor.label.to_string(),
            exe_name: descriptor.exe_name.to_string(),
            configured_path,
            resolved_path,
            available: false,
            version: String::new(),
            message: error,
            supported_versions: Vec::new(),
        }),
    }
}

pub fn get_all_toolchain_statuses(config: &AppConfig) -> Vec<ToolchainStatus> {
    TOOLCHAIN_DESCRIPTORS
        .iter()
        .filter_map(|descriptor| get_toolchain_status(config, descriptor.id).ok())
        .collect()
}

fn query_tool_version(exe_path: &str) -> Result<String, String> {
    let parent_dir = Path::new(exe_path).parent();
    let result = cmd_runner::run_tool(exe_path, &["-V"], parent_dir)?;
    let output = format!("{}\n{}", result.stdout, result.stderr).trim().to_string();

    if output.is_empty() {
        return Err("Toolchain did not return version output".to_string());
    }

    Ok(output.lines().next().unwrap_or("").trim().to_string())
}

/// 从工具的 usage 文本里解析它自报的支持版本列表。
///
/// thtk 的 usage 形如：
/// ```text
/// VERSION can be:
///   6, 7, ..., 103 (for Uwabami Breakers), ..., 19, or 20
/// ```
/// 解析不到就返回 None——调用方降级到 `game_version` 的静态表。
/// thdat 的 usage 不含该标记，返回 None 是预期行为不是故障。
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

/// 无参运行工具拿 usage 并解析出可用版本。
///
/// 不硬编码「thtk 版本 X 支持到 thYY」——那张表一定会过期。thtk 出 th21 时
/// 只需在 `game_version` 静态表加一行，探测逻辑无需改动。
///
/// 两个要点：
/// - thtk 的 `print_usage()` 走 **stdout**，且无参运行的退出码非 0，必须忽略退出码；
/// - 探测结果与静态表**取交集**：探测可能报出我们表里没有的新版本，那些版本我们
///   既没有标题也没有工具支持信息，暂不放出。
fn probe_supported_versions(exe_path: &str, tool_id: &str) -> Vec<u32> {
    let parent_dir = Path::new(exe_path).parent();
    let probed = cmd_runner::run_tool(exe_path, &[], parent_dir)
        .ok()
        .and_then(|result| parse_supported_versions(&result.stdout));

    match probed {
        Some(list) => crate::common::game_version::versions_for_tool(tool_id)
            .into_iter()
            .filter(|id| list.contains(id))
            .collect(),
        None => crate::common::game_version::versions_for_tool(tool_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::project_config::{ProjectConfig, ProjectToolchainConfig};
    use std::env;
    use std::fs;

    fn temp_root(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("thtk-toolchain-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_project_thtk_dir(root: &Path, thtk_dir: &str) {
        let config = ProjectConfig {
            toolchain: ProjectToolchainConfig {
                thtk_dir: thtk_dir.to_string(),
            },
            ..ProjectConfig::default()
        };
        crate::common::project_config::save_project_config(&root.to_string_lossy(), &config)
            .unwrap();
    }

    fn global_config(thtk_dir: &str) -> AppConfig {
        AppConfig {
            thtk_dir: thtk_dir.to_string(),
            ..AppConfig::default()
        }
    }

    #[test]
    fn project_thtk_dir_overrides_global() {
        let dir = temp_root("override");
        write_project_thtk_dir(&dir, "/project/thtk");

        let effective = effective_config(&global_config("/global/thtk"), Some(&dir.to_string_lossy()));

        assert_eq!(effective.thtk_dir, "/project/thtk");
        assert!(resolve_tool_path(&effective, "thmsg", "thmsg.exe").contains("/project/thtk"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_or_missing_project_override_keeps_global() {
        let dir = temp_root("fallback");

        // 没有 .thtk-project.json
        assert_eq!(
            effective_config(&global_config("/global/thtk"), Some(&dir.to_string_lossy())).thtk_dir,
            "/global/thtk"
        );

        // 有配置但 thtkDir 为空白 —— 不能把全局值清掉
        write_project_thtk_dir(&dir, "   ");
        assert_eq!(
            effective_config(&global_config("/global/thtk"), Some(&dir.to_string_lossy())).thtk_dir,
            "/global/thtk"
        );

        // 没有项目根
        assert_eq!(
            effective_config(&global_config("/global/thtk"), None).thtk_dir,
            "/global/thtk"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn explicit_tool_override_still_wins_over_project_dir() {
        let dir = temp_root("precedence");
        write_project_thtk_dir(&dir, "/project/thtk");

        let mut config = global_config("/global/thtk");
        config
            .tool_overrides
            .insert("thmsg".to_string(), "/exact/thmsg.exe".to_string());

        let effective = effective_config(&config, Some(&dir.to_string_lossy()));

        // 项目级目录只顶掉 thtk_dir，用户对单个 exe 的精确指定优先级更高
        assert_eq!(effective.thtk_dir, "/project/thtk");
        assert_eq!(
            resolve_tool_path(&effective, "thmsg", "thmsg.exe"),
            "/exact/thmsg.exe"
        );
        // 未被精确指定的工具仍走项目目录
        assert!(resolve_tool_path(&effective, "thstd", "thstd.exe").contains("/project/thtk"));

        let _ = fs::remove_dir_all(&dir);
    }

    // ---- usage 探测：thtk 自报支持哪些版本 ----

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
}
