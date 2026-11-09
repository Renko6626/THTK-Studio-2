use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub type CheckError = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptKind {
    Msg,
    Ending,
    Std,
}
impl ScriptKind {
    pub fn tool(self) -> &'static str {
        if self == Self::Std {
            "trustd"
        } else {
            "trumsg"
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Msg => "msgspec",
            Self::Ending => "endspec",
            Self::Std => "stdspec",
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Msg => "msg",
            Self::Ending => "ending",
            Self::Std => "std",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ScriptOperation {
    Check,
    Compile,
    Decompile,
}
impl ScriptOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Compile => "compile",
            Self::Decompile => "decompile",
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppSnapshot {
    pub default_game_version: String,
    pub truth_dir: String,
    pub tool_overrides: BTreeMap<String, String>,
}
impl Default for AppSnapshot {
    fn default() -> Self {
        Self {
            default_game_version: "20".into(),
            truth_dir: String::new(),
            tool_overrides: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ScriptMaps {
    pub msg: Vec<String>,
    pub ending: Vec<String>,
    pub std: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
    pub game_version: String,
    pub encoding: String,
    pub script_maps: ScriptMaps,
    pub truth_dir: String,
}
impl Default for ProjectSnapshot {
    fn default() -> Self {
        Self {
            game_version: String::new(),
            encoding: "shift-jis".into(),
            script_maps: ScriptMaps::default(),
            truth_dir: String::new(),
        }
    }
}
pub struct ContextInput {
    pub kind: ScriptKind,
    pub input_path: PathBuf,
    pub project_root: Option<PathBuf>,
    pub explicit_game_version: Option<String>,
    pub explicit_encoding: Option<String>,
    pub explicit_truth_dir: Option<PathBuf>,
    pub explicit_map_paths: Vec<PathBuf>,
    pub app: AppSnapshot,
    pub project: Option<ProjectSnapshot>,
    pub bundled_truth_dir: PathBuf,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptContext {
    pub kind: ScriptKind,
    pub game_version: u32,
    pub encoding: Option<String>,
    pub tool_path: PathBuf,
    pub map_paths: Vec<PathBuf>,
}
pub struct ScriptRequest {
    pub operation: ScriptOperation,
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub source_text: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub path: String,
    pub line: u32,
    pub column: Option<u32>,
    pub severity: String,
    pub message: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResult {
    pub success: bool,
    pub tool: String,
    pub mode: String,
    pub script_kind: String,
    pub input_path: String,
    pub output_path: Option<String>,
    pub message: String,
    pub diagnostics: Vec<Diagnostic>,
}
