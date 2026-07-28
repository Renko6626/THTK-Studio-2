use crate::common::recent_projects::{self, RecentProject};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

fn default_mcp_port() -> u16 {
    39127
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    // thtk 工具链的根目录
    pub thtk_dir: String,
    // 可选：显式指定 thecl.exe 路径，优先级高于 thtk_dir/thecl.exe
    pub thecl_path: String,
    // 默认 ECL map 路径，供 thecl 构建和编辑器高亮使用
    pub eclmap_path: String,
    // 通用工具链路径覆盖，key 例如 thecl/thmsg/thanm/thstd/thdat
    pub tool_overrides: BTreeMap<String, String>,
    // 默认目标游戏版本 (例如 "th14")
    pub default_game_version: String,
    // 编辑器主题 (给前端留的字段)
    pub theme: String,
    // MCP 服务器首选监听端口；被占用时自动回退到随机端口
    #[serde(default = "default_mcp_port")]
    pub mcp_port: u16,
    // 最近打开的项目，由专用命令维护，没有设置表单入口
    #[serde(default)]
    pub recent_projects: Vec<crate::common::recent_projects::RecentProject>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            thtk_dir: "".to_string(),
            thecl_path: "".to_string(),
            eclmap_path: "".to_string(),
            tool_overrides: BTreeMap::new(),
            default_game_version: "20".to_string(),
            theme: "dark".to_string(),
            mcp_port: default_mcp_port(),
            recent_projects: Vec::new(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    // 使用 Mutex 允许在多线程命令中安全修改配置
    pub config: Mutex<AppConfig>,
}

impl ConfigManager {
    pub fn new() -> Self {
        // 获取系统标准的配置目录
        // Windows: C:\Users\Name\AppData\Roaming\com.yourname.touhouide\config.json
        let proj_dirs = ProjectDirs::from("com", "abl", "thtk-studio")
            .expect("Could not determine config directory");

        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).unwrap_or_default();
        }

        let config_path = config_dir.join("settings.json");

        let config = Self::load_or_recover(&config_path);

        Self {
            config_path,
            config: Mutex::new(config),
        }
    }

    /// 读取 settings.json；损坏时**先备份再**回落到默认值。
    ///
    /// 原来是 `unwrap_or_default()` 直接吞掉解析错误，下一次保存就把默认值写回去，
    /// 用户的工具链路径、覆盖项和最近项目全部消失且无从恢复。现在最近项目
    /// 每次打开项目都会写这个文件，静默重置的代价比以前大得多。
    fn load_or_recover(config_path: &Path) -> AppConfig {
        if !config_path.exists() {
            return AppConfig::default();
        }

        let content = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("[config] 无法读取 {}: {e}", config_path.display());
                return AppConfig::default();
            }
        };

        match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                let backup = config_path.with_extension("json.bak");
                let backed_up = fs::write(&backup, &content).is_ok();
                eprintln!(
                    "[config] {} 解析失败({e})，已回落到默认设置。{}",
                    config_path.display(),
                    if backed_up {
                        format!("原文件已备份到 {}", backup.display())
                    } else {
                        "备份原文件也失败了".to_string()
                    }
                );
                AppConfig::default()
            }
        }
    }

    /// 同目录临时文件 + rename，避免写到一半断电/崩溃留下半截 JSON。
    /// 最近项目让这个文件的写入频率从"用户点保存"变成"每次打开项目"，值得加固。
    pub fn save(&self) -> Result<(), String> {
        let json = {
            let config = self.config.lock().unwrap_or_else(|e| e.into_inner());
            serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?
        };

        let temp_path = self
            .config_path
            .with_extension(format!("json.tmp{}", std::process::id()));

        fs::write(&temp_path, json).map_err(|e| format!("写入临时配置失败: {e}"))?;

        if let Err(e) = fs::rename(&temp_path, &self.config_path) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("替换配置文件失败: {e}"));
        }

        Ok(())
    }

    // 获取当前配置的一个副本
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    // 更新配置
    pub fn update_config(&self, new_config: AppConfig) -> Result<(), String> {
        {
            let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
            *config = new_config;
        } // 锁在这里释放
        self.save()
    }

    /// 更新设置表单能改的字段，保留后端自己管理的那些。见 [`merge_user_settings`]。
    pub fn update_user_settings(&self, incoming: AppConfig) -> Result<(), String> {
        let merged = {
            let current = self.config.lock().unwrap_or_else(|e| e.into_inner());
            merge_user_settings(&current, incoming)
        };
        self.update_config(merged)
    }

    pub fn recent_projects(&self) -> Vec<RecentProject> {
        recent_projects::sorted(&self.get_config().recent_projects)
    }

    pub fn set_recent_projects(&self, list: Vec<RecentProject>) -> Result<(), String> {
        {
            let mut config = self.config.lock().unwrap_or_else(|e| e.into_inner());
            config.recent_projects = list;
        }
        self.save()
    }
}

/// 把设置表单能改的字段合并进当前配置。
///
/// `save_settings` 的载荷是整个 `AppConfig`，但设置表单只填其中一部分，缺失字段
/// 在反序列化时会取 serde 默认值。这里刻意用**穷举式结构体字面量**从表单字段
/// 逐个取值，而不是 `..incoming` 加一份"要保留的字段"黑名单：黑名单的默认行为是
/// 覆盖，下一个没有表单入口的新字段会重复同一个静默重置 bug；穷举写法下漏掉字段
/// 是编译错误。
pub fn merge_user_settings(current: &AppConfig, incoming: AppConfig) -> AppConfig {
    AppConfig {
        // 设置表单负责的字段
        thtk_dir: incoming.thtk_dir,
        thecl_path: incoming.thecl_path,
        eclmap_path: incoming.eclmap_path,
        tool_overrides: incoming.tool_overrides,
        default_game_version: incoming.default_game_version,
        theme: incoming.theme,
        // 后端维护，表单不传：mcp_port 由用户手改 settings.json，
        // recent_projects 由 open_project / 最近项目命令写
        mcp_port: current.mcp_port,
        recent_projects: current.recent_projects.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::recent_projects::RecentProject;

    fn config_with_backend_state() -> AppConfig {
        AppConfig {
            thtk_dir: "D:/old".to_string(),
            mcp_port: 45678,
            recent_projects: vec![RecentProject {
                path: "D:/projects/th18".to_string(),
                name: "th18".to_string(),
                last_opened_at: 1234,
            }],
            ..AppConfig::default()
        }
    }

    /// 模拟工具链设置表单：只填自己那 6 个字段，其余走 serde 默认值
    fn payload_from_settings_form() -> AppConfig {
        AppConfig {
            thtk_dir: "D:/new".to_string(),
            default_game_version: "18".to_string(),
            ..AppConfig::default()
        }
    }

    #[test]
    fn saving_settings_form_does_not_wipe_backend_owned_fields() {
        let merged = merge_user_settings(&config_with_backend_state(), payload_from_settings_form());

        // 表单字段照常更新
        assert_eq!(merged.thtk_dir, "D:/new");
        assert_eq!(merged.default_game_version, "18");
        // 表单没有的字段不能被 serde 默认值冲掉
        assert_eq!(merged.mcp_port, 45678);
        assert_eq!(merged.recent_projects.len(), 1);
        assert_eq!(merged.recent_projects[0].path, "D:/projects/th18");
    }

    #[test]
    fn old_settings_json_without_recent_projects_still_loads() {
        // 升级前写下的配置没有 recent_projects / mcp_port 键
        let legacy = r#"{"thtk_dir":"D:/thtk","default_game_version":"17","theme":"dark"}"#;
        let config: AppConfig = serde_json::from_str(legacy).expect("旧配置必须能读");

        assert_eq!(config.thtk_dir, "D:/thtk");
        assert_eq!(config.default_game_version, "17");
        assert!(config.recent_projects.is_empty());
        assert_eq!(config.mcp_port, default_mcp_port());
    }

    #[test]
    fn malformed_recent_entry_does_not_reset_the_whole_config() {
        // 手工编辑过、某一条缺字段：不该让整个 AppConfig 反序列化失败
        let content =
            r#"{"thtk_dir":"D:/thtk","recent_projects":[{"path":"D:/a"},{"name":"b"}]}"#;
        let config: AppConfig = serde_json::from_str(content).expect("缺字段应走默认值");

        assert_eq!(config.thtk_dir, "D:/thtk");
        assert_eq!(config.recent_projects.len(), 2);
        assert_eq!(config.recent_projects[0].path, "D:/a");
        assert_eq!(config.recent_projects[0].name, "");
        assert_eq!(config.recent_projects[1].last_opened_at, 0);
    }
}
