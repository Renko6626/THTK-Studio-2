use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

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

        // 尝试加载现有配置，否则使用默认值
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        Self {
            config_path,
            config: Mutex::new(config),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap();
        let json = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    // 获取当前配置的一个副本
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    // 更新配置
    pub fn update_config(&self, new_config: AppConfig) -> Result<(), String> {
        {
            let mut config = self.config.lock().unwrap();
            *config = new_config;
        } // 锁在这里释放
        self.save()
    }
}
