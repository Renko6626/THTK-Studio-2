use crate::*;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const PROJECT_FILE: &str = ".thtk-project.json";

/// Normalize the spelling only; catalogue validation is specific to the caller.
pub fn parse_game_version(raw: &str) -> Result<u32, CheckError> {
    let trimmed = raw.trim().to_lowercase();
    let digits = trimmed.strip_prefix("th").unwrap_or(&trimmed);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("版本号非法: {raw:?}（应为 18 或 th18 这样的写法）"));
    }
    digits
        .parse()
        .map_err(|_| format!("版本号超出范围: {raw:?}"))
}
fn nonblank(s: &str) -> Option<&str> {
    (!s.trim().is_empty()).then(|| s.trim())
}
pub fn resolve_context(input: &ContextInput) -> Result<ScriptContext, CheckError> {
    let project = input.project.as_ref();
    let version = input
        .explicit_game_version
        .as_deref()
        .or_else(|| project.and_then(|p| nonblank(&p.game_version)))
        .unwrap_or(&input.app.default_game_version);
    let game_version = parse_game_version(version)?;
    let encoding = if input.kind == ScriptKind::Std {
        None
    } else {
        let value = input
            .explicit_encoding
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| project.map(|p| p.encoding.as_str()))
            .unwrap_or("shift-jis");
        if !["shift-jis", "gbk", "utf-8"].contains(&value) {
            return Err(format!("不支持的游戏文本编码：{value}"));
        }
        Some(value.into())
    };
    let root = input
        .project_root
        .as_deref()
        .map(std::path::absolute)
        .transpose()
        .map_err(|e| e.to_string())?;
    let mut map_paths = catalog::map_paths(
        game_version,
        input.kind,
        root.as_deref().and_then(Path::to_str),
        project,
    )?;
    map_paths.extend(input.explicit_map_paths.iter().cloned());
    map_paths = map_paths
        .iter()
        .map(std::path::absolute)
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let directory = input
        .explicit_truth_dir
        .clone()
        .or_else(|| {
            project
                .and_then(|p| nonblank(&p.truth_dir))
                .map(PathBuf::from)
        })
        .or_else(|| nonblank(&input.app.truth_dir).map(PathBuf::from))
        .unwrap_or_else(|| input.bundled_truth_dir.clone());
    let tool = input.kind.tool();
    let tool_path = if let Some(path) = input.app.tool_overrides.get(tool).and_then(|s| nonblank(s))
    {
        PathBuf::from(path)
    } else {
        directory.join(format!("{tool}{}", std::env::consts::EXE_SUFFIX))
    };
    Ok(ScriptContext {
        kind: input.kind,
        game_version,
        encoding,
        tool_path: std::path::absolute(tool_path).map_err(|e| e.to_string())?,
        map_paths,
    })
}
fn read_optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, CheckError> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}
pub fn load_app_snapshot() -> Result<AppSnapshot, CheckError> {
    let dirs = directories::ProjectDirs::from("com", "abl", "thtk-studio")
        .ok_or("Cannot locate application configuration directory")?;
    Ok(read_optional(&dirs.config_dir().join("settings.json"))?.unwrap_or_default())
}
pub fn load_project_snapshot(root: &Path) -> Result<Option<ProjectSnapshot>, CheckError> {
    #[derive(Deserialize, Default)]
    #[serde(default, rename_all = "camelCase")]
    struct Toolchain {
        truth_dir: String,
    }
    #[derive(Deserialize)]
    #[serde(default, rename_all = "camelCase")]
    struct ProjectFile {
        game_version: String,
        encoding: String,
        script_maps: ScriptMaps,
        toolchain: Toolchain,
    }
    impl Default for ProjectFile {
        fn default() -> Self {
            Self {
                game_version: String::new(),
                encoding: "shift-jis".into(),
                script_maps: ScriptMaps::default(),
                toolchain: Toolchain::default(),
            }
        }
    }
    Ok(
        read_optional::<ProjectFile>(&root.join(PROJECT_FILE))?.map(|p| ProjectSnapshot {
            game_version: p.game_version,
            encoding: p.encoding,
            script_maps: p.script_maps,
            truth_dir: p.toolchain.truth_dir,
        }),
    )
}
pub fn discover_project_root(source: &Path) -> Result<Option<PathBuf>, CheckError> {
    let source = std::path::absolute(source).map_err(|e| e.to_string())?;
    for parent in source.parent().into_iter().flat_map(Path::ancestors) {
        match fs::metadata(parent.join(PROJECT_FILE)) {
            Ok(_) => return Ok(Some(parent.into())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("{}: {e}", parent.display())),
        }
    }
    Ok(None)
}
