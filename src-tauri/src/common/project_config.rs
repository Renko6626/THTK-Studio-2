use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const PROJECT_CONFIG_FILENAME: &str = ".thtk-project.json";

/// 项目级配置，保存在工作区根目录的 .thtk-project.json
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ProjectConfig {
    /// 目标游戏版本 (例如 "18", "th18")
    pub game_version: String,
    /// 默认编码 ("shift-jis" | "utf-8")
    pub encoding: String,
    /// ECL map 文件路径列表（相对于项目根目录或绝对路径）
    pub map_paths: Vec<String>,
    /// 工具链路径覆盖（可选，覆盖全局设置）
    pub toolchain: ProjectToolchainConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ProjectToolchainConfig {
    /// 覆盖全局 thtk_dir
    pub thtk_dir: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            game_version: String::new(),
            encoding: "shift-jis".to_string(),
            map_paths: Vec::new(),
            toolchain: ProjectToolchainConfig::default(),
        }
    }
}

/// 从项目根目录读取 .thtk-project.json，不存在则返回 None
pub fn load_project_config(project_root: &str) -> Option<ProjectConfig> {
    let config_path = Path::new(project_root).join(PROJECT_CONFIG_FILENAME);
    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&config_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 保存 .thtk-project.json 到项目根目录
pub fn save_project_config(project_root: &str, config: &ProjectConfig) -> Result<(), String> {
    let config_path = Path::new(project_root).join(PROJECT_CONFIG_FILENAME);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// 将项目配置中的相对 map 路径解析为绝对路径
pub fn resolve_map_paths(project_root: &str, map_paths: &[String]) -> Vec<String> {
    map_paths
        .iter()
        .map(|p| {
            let path = Path::new(p);
            if path.is_absolute() {
                p.clone()
            } else {
                Path::new(project_root)
                    .join(path)
                    .to_string_lossy()
                    .to_string()
            }
        })
        .collect()
}
