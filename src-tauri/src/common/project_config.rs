use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT_CONFIG_FILENAME: &str = ".thtk-project.json";

/// 允许的 encoding 取值。项目内既有需要 Shift-JIS 的原始游戏文本，
/// 也有 UTF-8 的 .decl/.dmsg 源码，因此两者都必须支持。
pub const SUPPORTED_ENCODINGS: [&str; 2] = ["shift-jis", "utf-8"];

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

/// 配置文件的三种状态。区分"不存在"与"损坏"是必需的：
/// 二者都返回 None 会让 UI 把损坏的文件当成"还没有配置"，
/// 用户一按保存就静默覆盖掉自己手写的内容。
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectConfigStatus {
    /// 项目根下没有 .thtk-project.json
    Absent,
    /// 读取并校验通过
    Loaded,
    /// 文件存在但无法读取、不是合法 JSON，或字段取值非法
    Invalid,
}

/// 面向 UI 的加载结果，始终带上配置文件的绝对路径，
/// 以便在损坏时能把"将要被覆盖的是哪个文件"如实展示给用户。
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigLoad {
    pub status: ProjectConfigStatus,
    pub value: Option<ProjectConfig>,
    pub error: Option<String>,
    pub path: String,
}

fn config_path_for(project_root: &str) -> PathBuf {
    Path::new(project_root).join(PROJECT_CONFIG_FILENAME)
}

/// 校验配置字段。game_version 允许为空（表示回退到全局默认版本）。
pub fn validate_project_config(config: &ProjectConfig) -> Result<(), String> {
    let encoding = config.encoding.trim();
    if !SUPPORTED_ENCODINGS.contains(&encoding) {
        return Err(format!(
            "encoding 取值非法: {:?}（支持 {}）",
            config.encoding,
            SUPPORTED_ENCODINGS.join(" / ")
        ));
    }

    if let Some(index) = config.map_paths.iter().position(|p| p.trim().is_empty()) {
        return Err(format!("mapPaths[{index}] 为空路径"));
    }

    Ok(())
}

/// 读取 .thtk-project.json 并区分不存在 / 有效 / 损坏三态。
/// 供项目配置 UI 使用；工具链调用路径请用 [`load_project_config`]。
pub fn load_project_config_detailed(project_root: &str) -> ProjectConfigLoad {
    let config_path = config_path_for(project_root);
    let path_string = config_path.to_string_lossy().to_string();

    let invalid = |error: String| ProjectConfigLoad {
        status: ProjectConfigStatus::Invalid,
        value: None,
        error: Some(error),
        path: path_string.clone(),
    };

    if !config_path.exists() {
        return ProjectConfigLoad {
            status: ProjectConfigStatus::Absent,
            value: None,
            error: None,
            path: path_string,
        };
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => return invalid(format!("无法读取配置文件: {e}")),
    };

    let config: ProjectConfig = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(e) => return invalid(format!("JSON 解析失败: {e}")),
    };

    if let Err(e) = validate_project_config(&config) {
        return invalid(e);
    }

    ProjectConfigLoad {
        status: ProjectConfigStatus::Loaded,
        value: Some(config),
        error: None,
        path: path_string,
    }
}

/// 从项目根目录读取 .thtk-project.json，不存在或无法解析时返回 None。
///
/// 这是工具链调用路径（ecl / msg / thstd / thdat / mcp）使用的尽力而为版本：
/// 只要能解析出结构就采用，**不做**字段校验，以免 UI 侧新增的校验规则
/// 意外改变既有项目的编译行为。需要向用户报告损坏状态时用
/// [`load_project_config_detailed`]。
pub fn load_project_config(project_root: &str) -> Option<ProjectConfig> {
    let config_path = config_path_for(project_root);
    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&config_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 保存 .thtk-project.json 到项目根目录。
///
/// 先写同目录临时文件再 rename，避免写入中断留下半截 JSON 把用户的配置毁掉。
/// 同目录是必要条件——跨挂载点 rename 不是原子的，也可能直接失败。
pub fn save_project_config(project_root: &str, config: &ProjectConfig) -> Result<(), String> {
    validate_project_config(config)?;

    let config_path = config_path_for(project_root);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;

    // 带 pid 后缀，避免多个实例同时保存时互相踩临时文件
    let temp_path = config_path.with_extension(format!("json.tmp{}", std::process::id()));

    fs::write(&temp_path, json).map_err(|e| format!("写入临时文件失败: {e}"))?;

    if let Err(e) = fs::rename(&temp_path, &config_path) {
        // rename 失败时清掉临时文件，别在项目根里留垃圾
        let _ = fs::remove_file(&temp_path);
        return Err(format!("替换配置文件失败: {e}"));
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// 建一个临时目录当项目根。用测试名做后缀避免并行测试互相干扰。
    fn temp_root(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("thtk-project-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn root_str(dir: &Path) -> String {
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn absent_when_no_config_file() {
        let dir = temp_root("absent");
        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Absent);
        assert!(result.value.is_none());
        assert!(result.error.is_none());
        assert!(result.path.ends_with(PROJECT_CONFIG_FILENAME));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_on_broken_json_and_keeps_file_content() {
        let dir = temp_root("broken");
        let path = dir.join(PROJECT_CONFIG_FILENAME);
        let original = "{ this is not json";
        fs::write(&path, original).unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Invalid);
        assert!(result.value.is_none());
        assert!(result.error.unwrap().contains("JSON 解析失败"));
        // 加载不得改动用户的文件
        assert_eq!(fs::read_to_string(&path).unwrap(), original);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_on_bad_encoding_value() {
        let dir = temp_root("bad-encoding");
        fs::write(
            dir.join(PROJECT_CONFIG_FILENAME),
            r#"{"gameVersion":"18","encoding":"utf8"}"#,
        )
        .unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Invalid);
        assert!(result.error.unwrap().contains("encoding"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_valid_config_and_fills_defaults() {
        let dir = temp_root("valid");
        fs::write(
            dir.join(PROJECT_CONFIG_FILENAME),
            r#"{"gameVersion":"18","mapPaths":["maps/th18.eclm"]}"#,
        )
        .unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Loaded);
        let value = result.value.unwrap();
        assert_eq!(value.game_version, "18");
        // 缺省字段回落到 Default
        assert_eq!(value.encoding, "shift-jis");
        assert_eq!(value.map_paths, vec!["maps/th18.eclm".to_string()]);
        assert_eq!(value.toolchain.thtk_dir, "");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn best_effort_loader_ignores_validation_but_still_rejects_broken_json() {
        let dir = temp_root("best-effort");
        let path = dir.join(PROJECT_CONFIG_FILENAME);

        // encoding 非法：详细加载器报 Invalid，尽力而为加载器仍然采用，
        // 以免 UI 校验规则回头改变既有项目的工具链行为。
        fs::write(&path, r#"{"encoding":"utf8"}"#).unwrap();
        assert_eq!(
            load_project_config_detailed(&root_str(&dir)).status,
            ProjectConfigStatus::Invalid
        );
        assert_eq!(load_project_config(&root_str(&dir)).unwrap().encoding, "utf8");

        // JSON 本身坏掉时两者都拿不到值
        fs::write(&path, "{{{").unwrap();
        assert!(load_project_config(&root_str(&dir)).is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_roundtrip_and_leaves_no_temp_file() {
        let dir = temp_root("roundtrip");
        let config = ProjectConfig {
            game_version: "17".to_string(),
            encoding: "utf-8".to_string(),
            map_paths: vec!["maps/th17.eclm".to_string()],
            toolchain: ProjectToolchainConfig {
                thtk_dir: "D:\\tools\\thtk".to_string(),
            },
        };

        save_project_config(&root_str(&dir), &config).unwrap();

        let result = load_project_config_detailed(&root_str(&dir));
        assert_eq!(result.status, ProjectConfigStatus::Loaded);
        let loaded = result.value.unwrap();
        assert_eq!(loaded.game_version, "17");
        assert_eq!(loaded.encoding, "utf-8");
        assert_eq!(loaded.map_paths, config.map_paths);
        assert_eq!(loaded.toolchain.thtk_dir, "D:\\tools\\thtk");

        // 原子写入不应留下临时文件
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name.contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "残留临时文件: {leftovers:?}");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_rejects_invalid_config_without_touching_existing_file() {
        let dir = temp_root("reject");
        let path = dir.join(PROJECT_CONFIG_FILENAME);
        let original = r#"{"gameVersion":"18","encoding":"shift-jis","mapPaths":[],"toolchain":{"thtkDir":""}}"#;
        fs::write(&path, original).unwrap();

        let bad = ProjectConfig {
            encoding: "latin-1".to_string(),
            ..ProjectConfig::default()
        };
        assert!(save_project_config(&root_str(&dir), &bad).is_err());
        // 校验失败必须发生在任何写入之前
        assert_eq!(fs::read_to_string(&path).unwrap(), original);

        let empty_map = ProjectConfig {
            map_paths: vec!["  ".to_string()],
            ..ProjectConfig::default()
        };
        let err = save_project_config(&root_str(&dir), &empty_map).unwrap_err();
        assert!(err.contains("mapPaths[0]"));
        assert_eq!(fs::read_to_string(&path).unwrap(), original);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolves_relative_map_paths_against_project_root() {
        let resolved = resolve_map_paths(
            "/projects/th18",
            &["maps/th18.eclm".to_string(), "/abs/th18.eclm".to_string()],
        );

        assert!(resolved[0].contains("projects"));
        assert!(resolved[0].ends_with("th18.eclm"));
        // 绝对路径原样保留
        assert_eq!(resolved[1], "/abs/th18.eclm");
    }
}
