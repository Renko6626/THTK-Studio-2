use super::maps::TruthScriptKind;
use crate::{
    common::{project_config::ProjectConfig, toolchain},
    config::AppConfig,
    modules::ecl::error_parser::Diagnostic,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScriptMode {
    Compile,
    Decompile,
}
impl ScriptMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Decompile => "decompile",
        }
    }
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

impl TruthScriptKind {
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

pub struct ScriptRequest {
    pub mode: ScriptMode,
    pub kind: TruthScriptKind,
    pub input_path: String,
    pub output_path: Option<String>,
    pub encoding: Option<String>,
}

impl From<TruthScriptKind> for script_check::ScriptKind {
    fn from(kind: TruthScriptKind) -> Self {
        match kind {
            TruthScriptKind::Msg => Self::Msg,
            TruthScriptKind::Ending => Self::Ending,
            TruthScriptKind::Std => Self::Std,
        }
    }
}
impl From<script_check::ScriptKind> for TruthScriptKind {
    fn from(kind: script_check::ScriptKind) -> Self {
        match kind {
            script_check::ScriptKind::Msg => Self::Msg,
            script_check::ScriptKind::Ending => Self::Ending,
            script_check::ScriptKind::Std => Self::Std,
        }
    }
}
pub fn resolve_script_context(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    kind: TruthScriptKind,
    input_path: &str,
    encoding: Option<&str>,
) -> Result<script_check::ScriptContext, String> {
    let project = if let Some(p) = project {
        Some(script_check::ProjectSnapshot {
            game_version: p.game_version.clone(),
            encoding: p.encoding.clone(),
            truth_dir: p.toolchain.truth_dir.clone(),
            script_maps: script_check::ScriptMaps {
                msg: p.script_maps.msg.clone(),
                ending: p.script_maps.ending.clone(),
                std: p.script_maps.std.clone(),
            },
        })
    } else {
        root.map(|r| script_check::load_project_snapshot(std::path::Path::new(r)))
            .transpose()?
            .flatten()
    };
    script_check::resolve_context(&script_check::ContextInput {
        kind: kind.into(),
        input_path: input_path.into(),
        project_root: root.map(Into::into),
        explicit_game_version: None,
        explicit_encoding: encoding.map(Into::into),
        explicit_truth_dir: None,
        explicit_map_paths: Vec::new(),
        app: script_check::AppSnapshot {
            default_game_version: config.default_game_version.clone(),
            truth_dir: config.truth_dir.clone(),
            tool_overrides: config
                .tool_overrides
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        },
        project,
        bundled_truth_dir: toolchain::bundled_truth_dir(),
    })
}
pub(crate) fn adapt_result(result: script_check::ScriptResult) -> ScriptResult {
    ScriptResult {
        success: result.success,
        tool: result.tool,
        mode: result.mode,
        script_kind: result.script_kind,
        input_path: result.input_path,
        output_path: result.output_path,
        message: result.message,
        diagnostics: result
            .diagnostics
            .into_iter()
            .map(adapt_diagnostic)
            .collect(),
    }
}
pub(crate) fn adapt_diagnostic(d: script_check::Diagnostic) -> Diagnostic {
    Diagnostic {
        path: d.path,
        line: d.line,
        column: d.column,
        severity: d.severity,
        message: d.message,
    }
}
pub fn run(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    request: &ScriptRequest,
) -> ScriptResult {
    let attempt = (|| {
        let context = resolve_script_context(
            config,
            root,
            project,
            request.kind,
            &request.input_path,
            request.encoding.as_deref(),
        )?;
        script_check::run(
            &context,
            &script_check::ScriptRequest {
                operation: match request.mode {
                    ScriptMode::Compile => script_check::ScriptOperation::Compile,
                    ScriptMode::Decompile => script_check::ScriptOperation::Decompile,
                },
                input_path: request.input_path.clone().into(),
                output_path: request.output_path.as_ref().map(Into::into),
                source_text: None,
            },
        )
    })();
    match attempt {
        Ok(result) => adapt_result(result),
        Err(message) => ScriptResult {
            success: false,
            tool: request.kind.tool().into(),
            mode: request.mode.as_str().into(),
            script_kind: request.kind.as_str().into(),
            input_path: request.input_path.clone(),
            output_path: None,
            message,
            diagnostics: Vec::new(),
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn project_and_pragma_maps_share_lookup_parameter_shapes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        for (kind, opcode, signature, args) in [
            (TruthScriptKind::Msg, 17, "z(bs=4;mask=0x77,7,16)", "\"汉\""),
            (
                TruthScriptKind::Ending,
                3,
                "z(bs=4;mask=0x77,7,16)",
                "\"汉\"",
            ),
            (TruthScriptKind::Std, 3, "SSfff", "60, 0, 1.0, 2.0, 3.0"),
        ] {
            let magic = match kind {
                TruthScriptKind::Msg => "msg",
                TruthScriptKind::Ending => "end",
                TruthScriptKind::Std => "std",
            };
            fs::write(dir.path().join("project.map"),format!("!{magic}map\n!ins_names\n{opcode} projectName\n!ins_signatures\n{opcode} {signature}\n")).unwrap();
            fs::write(
                dir.path().join("source.map"),
                format!("!{magic}map\n!ins_names\n{opcode} sourceName\n"),
            )
            .unwrap();
            let mut project = ProjectConfig {
                game_version: "18".into(),
                encoding: "gbk".into(),
                ..Default::default()
            };
            let paths = vec!["project.map".into()];
            match kind {
                TruthScriptKind::Msg => project.script_maps.msg = paths,
                TruthScriptKind::Ending => project.script_maps.ending = paths,
                TruthScriptKind::Std => project.script_maps.std = paths,
            };
            let meta = if kind == TruthScriptKind::Std {
                "meta { unknown: 0, anm_path: \"stage.anm\", objects: {}, instances: [] }"
            } else {
                "meta { table: {0: {script: \"main\"}} }"
            };
            let source=format!("#pragma mapfile \"source.map\"\n{meta}\nscript main {{\n sourceName({args});\n}}\n");
            let input = dir.path().join(format!("input.{}", kind.extension()));
            fs::write(&input, &source).unwrap();
            let definitions = super::super::commands::source_definitions(
                18,
                kind,
                Some(root),
                Some(&project),
                input.to_str().unwrap(),
                &source,
            )
            .unwrap();
            let instruction = definitions
                .instructions
                .iter()
                .find(|i| i.opcode == opcode)
                .unwrap();
            assert_eq!(instruction.name.as_deref(), Some("sourceName"));
            assert_eq!(
                instruction.params.as_ref().unwrap().len(),
                if kind == TruthScriptKind::Std { 5 } else { 1 }
            );
        }
    }
}
