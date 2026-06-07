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
