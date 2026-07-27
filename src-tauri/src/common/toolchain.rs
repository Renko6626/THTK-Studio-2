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
    let Some(root) = project_root else {
        return config.clone();
    };
    let Some(project) = crate::common::project_config::load_project_config(root) else {
        return config.clone();
    };

    let project_thtk_dir = project.toolchain.thtk_dir.trim();
    if project_thtk_dir.is_empty() {
        return config.clone();
    }

    let mut effective = config.clone();
    effective.thtk_dir = project_thtk_dir.to_string();
    effective
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
        });
    }

    match query_tool_version(&resolved_path) {
        Ok(version) => Ok(ToolchainStatus {
            tool: descriptor.id.to_string(),
            label: descriptor.label.to_string(),
            exe_name: descriptor.exe_name.to_string(),
            configured_path,
            resolved_path,
            available: true,
            version,
            message: "Toolchain is available".to_string(),
        }),
        Err(error) => Ok(ToolchainStatus {
            tool: descriptor.id.to_string(),
            label: descriptor.label.to_string(),
            exe_name: descriptor.exe_name.to_string(),
            configured_path,
            resolved_path,
            available: false,
            version: String::new(),
            message: error,
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
}
