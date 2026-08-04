use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT_CONFIG_FILENAME: &str = ".thtk-project.json";

/// 允许的 encoding 取值——指的是**游戏文本**的编码，不是 .decl/.dmsg 源文件的
/// （那些在磁盘上始终是 UTF-8）。
///
/// - `shift-jis`：原作唯一的原生编码，默认值。
/// - `gbk`：汉化版常用。它同时装得下简体汉字与日文假名/汉字，而汉化版常有简日
///   混排。**前提是游戏侧已做适配**（字体 charset 补丁、字节边界判断、转区）。
/// - `utf-8`：原版游戏读不了（thcrap 文档：games don't support any form of
///   Unicode）。保留是为了兼容既有配置与自定义引擎，工具会在打包时告诫。
pub const SUPPORTED_ENCODINGS: [&str; 3] = ["shift-jis", "gbk", "utf-8"];

/// `.thtk-project.json` 顶层允许出现的键。
///
/// serde 默认**忽略**未知字段，所以把 `mapPaths` 拼成 `mapPath` 会安静地解析成功、
/// 内容被丢掉，UI 看到的是 status=loaded 加一个空列表，一保存用户的东西就没了。
/// 手写 JSON 时拼错键名恰恰是最常见的笔误，只靠 serde 的语法/类型报错拦不住。
/// 这里显式白名单，只在面向 UI 的加载器上生效。
const KNOWN_TOP_LEVEL_KEYS: [&str; 4] = ["gameVersion", "encoding", "mapPaths", "toolchain"];
const KNOWN_TOOLCHAIN_KEYS: [&str; 1] = ["thtkDir"];

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

/// 拒绝顶层不是对象、以及任何无法识别的键。
///
/// 宁可报错也不要静默丢弃：用户手写的内容被无声吞掉，然后被下一次保存覆盖，
/// 比直接告诉他"这个字段我不认识"糟糕得多。
fn check_known_keys(raw: &serde_json::Value) -> Result<(), String> {
    let Some(object) = raw.as_object() else {
        return Err("配置文件的顶层不是 JSON 对象".to_string());
    };

    if let Some(key) = object
        .keys()
        .find(|key| !KNOWN_TOP_LEVEL_KEYS.contains(&key.as_str()))
    {
        return Err(format!(
            "无法识别的字段 {key:?}（可用字段：{}）",
            KNOWN_TOP_LEVEL_KEYS.join(" / ")
        ));
    }

    if let Some(toolchain) = object.get("toolchain") {
        let Some(toolchain_object) = toolchain.as_object() else {
            return Err("toolchain 不是 JSON 对象".to_string());
        };
        if let Some(key) = toolchain_object
            .keys()
            .find(|key| !KNOWN_TOOLCHAIN_KEYS.contains(&key.as_str()))
        {
            return Err(format!(
                "toolchain 下有无法识别的字段 {key:?}（可用字段：{}）",
                KNOWN_TOOLCHAIN_KEYS.join(" / ")
            ));
        }
    }

    Ok(())
}

/// 把 game_version 归一成 thtk 直接可用的数字形式（`"TH18"` → `"18"`）。
///
/// 空值保持不动（表示回退全局默认）；解析不了的值也原样保留，交给
/// [`validate_project_config`] 去报错——归一化不是校验的地方。
///
/// 归一化必须发生在**读取**而不只是保存时：老配置文件里的 `"th18"` 若原样
/// 交给前端，下拉框的 option value 是纯数字，会一个都对不上而显示空白；
/// semantic-loader 拿它拼 `th{version}.eclm` 会得到 `"thth18.eclm"`。
fn canonicalize_game_version(config: &mut ProjectConfig) {
    let trimmed = config.game_version.trim();
    if trimmed.is_empty() {
        config.game_version = String::new();
        return;
    }
    if let Ok(id) = crate::common::game_version::parse(trimmed) {
        config.game_version = id.to_string();
    }
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

    // 空表示回退全局默认，允许；非空则必须是 thtk 认识的版本。
    // 不校验的话，"21" 或 "th１８"（全角）这类值会一路带到命令行，
    // thtk 按 %u 解析后变成 0 或直接失败，报错完全指不到配置本身。
    let game_version = config.game_version.trim();
    if !game_version.is_empty() {
        crate::common::game_version::parse(game_version)?;
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

    // 先解析成 Value 做键名检查，再转成结构体——直接 from_str 会让未知键静默消失
    let raw: serde_json::Value = match serde_json::from_str(&content) {
        Ok(raw) => raw,
        Err(e) => return invalid(format!("JSON 解析失败: {e}")),
    };

    if let Err(e) = check_known_keys(&raw) {
        return invalid(e);
    }

    let mut config: ProjectConfig = match serde_json::from_value(raw) {
        Ok(config) => config,
        Err(e) => return invalid(format!("字段类型不正确: {e}")),
    };
    canonicalize_game_version(&mut config);

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
    let mut config: ProjectConfig = serde_json::from_str(&content).ok()?;
    canonicalize_game_version(&mut config);
    Some(config)
}

/// 保存 .thtk-project.json 到项目根目录。
///
/// 先写同目录临时文件再 rename，避免写入中断留下半截 JSON 把用户的配置毁掉。
/// 同目录是必要条件——跨挂载点 rename 不是原子的，也可能直接失败。
pub fn save_project_config(project_root: &str, config: &ProjectConfig) -> Result<(), String> {
    // 归一化后再写：校验是按 trim 过的值做的，直接落盘会让 `" utf-8 "` 这种
    // 通过校验却在消费端按等值比较时既不等于 shift-jis 也不等于 utf-8。
    let mut config = ProjectConfig {
        game_version: config.game_version.trim().to_string(),
        encoding: config.encoding.trim().to_string(),
        map_paths: config
            .map_paths
            .iter()
            .map(|path| path.trim().to_string())
            .collect(),
        toolchain: ProjectToolchainConfig {
            thtk_dir: config.toolchain.thtk_dir.trim().to_string(),
        },
    };
    canonicalize_game_version(&mut config);
    validate_project_config(&config)?;

    let config_path = config_path_for(project_root);
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

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

    fn config_with_version(version: &str) -> ProjectConfig {
        ProjectConfig {
            game_version: version.to_string(),
            encoding: "shift-jis".to_string(),
            map_paths: Vec::new(),
            toolchain: ProjectToolchainConfig {
                thtk_dir: String::new(),
            },
        }
    }

    /// 加载时把 game_version 归一成 thtk 直接可用的数字形式。
    ///
    /// 不归一的话有两处会坏：前端下拉框的 option value 是纯数字，旧配置里的
    /// "th18" 对不上任何一项会显示空白；semantic-loader 用它拼 `th{version}.eclm`，
    /// 会得到 "thth18.eclm"。
    #[test]
    fn load_canonicalizes_th_prefixed_version() {
        let dir = temp_root("canonicalize-version");
        fs::write(
            config_path_for(&root_str(&dir)),
            r#"{"gameVersion":"TH18","encoding":"shift-jis","mapPaths":[],"toolchain":{"thtkDir":""}}"#,
        )
        .unwrap();

        let detailed = load_project_config_detailed(&root_str(&dir));
        assert_eq!(detailed.status, ProjectConfigStatus::Loaded);
        assert_eq!(detailed.value.unwrap().game_version, "18");

        let best_effort = load_project_config(&root_str(&dir)).unwrap();
        assert_eq!(best_effort.game_version, "18");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_leaves_empty_game_version_alone() {
        let dir = temp_root("empty-version-untouched");
        fs::write(
            config_path_for(&root_str(&dir)),
            r#"{"gameVersion":"","encoding":"utf-8","mapPaths":[],"toolchain":{"thtkDir":""}}"#,
        )
        .unwrap();

        let loaded = load_project_config(&root_str(&dir)).unwrap();
        assert_eq!(loaded.game_version, "", "空值表示回退全局默认，不能被改写");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_rejects_unknown_game_version() {
        assert!(validate_project_config(&config_with_version("21")).is_err());
        assert!(validate_project_config(&config_with_version("abc")).is_err());
    }

    #[test]
    fn validate_accepts_th_prefixed_game_version() {
        assert!(validate_project_config(&config_with_version("th18")).is_ok());
    }

    #[test]
    fn validate_still_accepts_empty_game_version() {
        assert!(
            validate_project_config(&config_with_version("")).is_ok(),
            "空字符串表示回退全局默认，必须继续接受"
        );
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
        // 绝对路径的写法必须按平台给：Windows 下 "/abs/x" 的 is_absolute() 是 false
        // （缺盘符前缀），用它做断言会让这个测试只在 Unix 上通过。
        #[cfg(windows)]
        let absolute = r"C:\abs\th18.eclm";
        #[cfg(not(windows))]
        let absolute = "/abs/th18.eclm";

        let resolved = resolve_map_paths(
            "/projects/th18",
            &["maps/th18.eclm".to_string(), absolute.to_string()],
        );

        assert!(resolved[0].contains("projects"));
        assert!(resolved[0].ends_with("th18.eclm"));
        // 绝对路径原样保留
        assert_eq!(resolved[1], absolute);
    }

    #[test]
    fn invalid_on_misspelled_key_instead_of_silently_dropping_it() {
        let dir = temp_root("typo-key");
        let path = dir.join(PROJECT_CONFIG_FILENAME);
        // mapPaths 拼成 mapPath —— serde 默认会忽略它，内容被丢掉且报 loaded
        let original = r#"{"gameVersion":"18","mapPath":["maps/th18.eclm"]}"#;
        fs::write(&path, original).unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Invalid);
        let error = result.error.unwrap();
        assert!(error.contains("mapPath"), "错误里要点名是哪个键: {error}");
        // 加载不得改动用户的文件
        assert_eq!(fs::read_to_string(&path).unwrap(), original);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_on_unknown_nested_toolchain_key() {
        let dir = temp_root("typo-nested");
        fs::write(
            dir.join(PROJECT_CONFIG_FILENAME),
            r#"{"toolchain":{"thtkdir":"D:/thtk"}}"#,
        )
        .unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Invalid);
        assert!(result.error.unwrap().contains("thtkdir"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_when_top_level_is_not_an_object() {
        let dir = temp_root("not-object");
        // 合法 JSON，但 serde 会把它当成按位置解析的结构体，全取默认值
        fs::write(dir.join(PROJECT_CONFIG_FILENAME), "[]").unwrap();

        let result = load_project_config_detailed(&root_str(&dir));

        assert_eq!(result.status, ProjectConfigStatus::Invalid);
        assert!(result.error.unwrap().contains("不是 JSON 对象"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_normalizes_whitespace_so_consumers_can_compare_by_equality() {
        let dir = temp_root("normalize");
        let config = ProjectConfig {
            game_version: "  18  ".to_string(),
            encoding: "  utf-8  ".to_string(),
            map_paths: vec!["  maps/th18.eclm  ".to_string()],
            toolchain: ProjectToolchainConfig {
                thtk_dir: "  D:/thtk  ".to_string(),
            },
        };

        save_project_config(&root_str(&dir), &config).unwrap();
        let loaded = load_project_config_detailed(&root_str(&dir)).value.unwrap();

        // 消费端一律按等值比较 encoding，留着空白会让它既不等于 shift-jis 也不等于 utf-8
        assert_eq!(loaded.encoding, "utf-8");
        assert_eq!(loaded.game_version, "18");
        assert_eq!(loaded.map_paths, vec!["maps/th18.eclm".to_string()]);
        assert_eq!(loaded.toolchain.thtk_dir, "D:/thtk");

        let _ = fs::remove_dir_all(&dir);
    }
}
