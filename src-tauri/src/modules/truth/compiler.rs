use super::{catalog, diagnostics, maps::TruthScriptKind};
use crate::{
    common::{project_config::ProjectConfig, toolchain},
    config::AppConfig,
    modules::ecl::error_parser::Diagnostic,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

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

pub fn run(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    request: &ScriptRequest,
) -> ScriptResult {
    let mut result = ScriptResult {
        success: false,
        tool: request.kind.tool().into(),
        mode: request.mode.as_str().into(),
        script_kind: request.kind.as_str().into(),
        input_path: request.input_path.clone(),
        output_path: None,
        message: String::new(),
        diagnostics: Vec::new(),
    };
    let attempt = (|| -> Result<(), String> {
        let input = std::path::absolute(&request.input_path).map_err(|e| e.to_string())?;
        if !input.is_file() {
            return Err(format!("输入文件不存在：{}", input.display()));
        }
        let cwd = input.parent().ok_or("Source has no parent directory")?;
        let game = crate::common::game_version::resolve_from(project, config, request.kind.tool())?;
        let maps = catalog::map_paths(game, request.kind, root, project)?;
        if matches!(request.mode, ScriptMode::Compile)
            && !input
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case(request.kind.extension()))
        {
            return Err(format!(
                "请使用 truth 原生 .{} 源码；旧 .dmsg/.dstd 已废弃",
                request.kind.extension()
            ));
        }
        let output = request
            .output_path
            .as_ref()
            .filter(|p| !p.trim().is_empty())
            .map(|p| {
                let p = Path::new(p);
                if p.is_absolute() {
                    p.into()
                } else {
                    cwd.join(p)
                }
            })
            .unwrap_or_else(|| {
                input.with_extension(match request.mode {
                    ScriptMode::Decompile => request.kind.extension(),
                    ScriptMode::Compile => {
                        if request.kind == TruthScriptKind::Std {
                            "std"
                        } else {
                            "msg"
                        }
                    }
                })
            });
        if output == input
            || output
                .canonicalize()
                .ok()
                .zip(input.canonicalize().ok())
                .is_some_and(|(a, b)| a == b)
        {
            return Err("输出路径不能覆盖输入文件".into());
        }
        let exe = toolchain::resolve_tool_path(
            config,
            request.kind.tool(),
            &format!("{}.exe", request.kind.tool()),
        );
        let mut cmd = Command::new(exe);
        cmd.arg(request.mode.as_str())
            .arg(&input)
            .args(["-g", &game.to_string(), "-o"])
            .arg(&output)
            .current_dir(cwd)
            .env_remove("TRUTH_MAP_PATH");
        for map in maps {
            cmd.arg("-m").arg(map);
        }
        if request.kind == TruthScriptKind::Ending {
            cmd.arg("--ending");
        }
        if request.kind != TruthScriptKind::Std {
            let encoding = request
                .encoding
                .as_deref()
                .filter(|s| !s.is_empty())
                .or(project.map(|p| p.encoding.as_str()))
                .unwrap_or("shift-jis");
            if !crate::common::project_config::SUPPORTED_ENCODINGS.contains(&encoding) {
                return Err(format!("不支持的游戏文本编码：{encoding}"));
            }
            cmd.args(["--encoding", encoding]);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let process = cmd
            .output()
            .map_err(|e| format!("无法启动 {}：{e}", request.kind.tool()))?;
        let stderr = String::from_utf8_lossy(&process.stderr);
        let stdout = String::from_utf8_lossy(&process.stdout);
        result.diagnostics = diagnostics::parse(&stderr, cwd);
        result.message = format!("{stdout}\n{stderr}").trim().into();
        result.success = process.status.success();
        if result.success {
            result.output_path = Some(output.to_string_lossy().into_owned());
            if result.message.is_empty() {
                result.message = "完成".into();
            }
        } else if result.message.is_empty() {
            result.message = format!("{} 退出：{}", request.kind.tool(), process.status);
        }
        Ok(())
    })();
    if let Err(error) = attempt {
        result.message = error;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    #[ignore = "requires node scripts/build-truth.mjs for the host platform"]
    fn bundled_tools_share_maps_encoding_and_diagnostics_with_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        let config = AppConfig::default();
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
            let mut request = ScriptRequest {
                mode: ScriptMode::Compile,
                kind,
                input_path: input.to_string_lossy().into_owned(),
                output_path: Some("built.bin".into()),
                encoding: None,
            };
            let result = run(&config, Some(root), Some(&project), &request);
            assert!(result.success, "{}", result.message);
            let bytes = fs::read(dir.path().join("built.bin")).unwrap();
            if kind != TruthScriptKind::Std {
                assert!(bytes.windows(4).any(|w| w == [0xcd, 0xc4, 0x95, 0xbc]));
            }
            let decoded = run(
                &config,
                Some(root),
                Some(&project),
                &ScriptRequest {
                    mode: ScriptMode::Decompile,
                    kind,
                    input_path: result.output_path.unwrap(),
                    output_path: Some(format!("decoded.{}", kind.extension())),
                    encoding: None,
                },
            );
            assert!(decoded.success, "{}", decoded.message);
            let text = fs::read_to_string(decoded.output_path.unwrap()).unwrap();
            assert!(text.contains("projectName("));
            if kind != TruthScriptKind::Std {
                assert!(text.contains("汉"));
            }
            fs::write(&input, source.replace(args, "")).unwrap();
            let failed = run(&config, Some(root), Some(&project), &request);
            assert!(!failed.success);
            assert!(
                failed
                    .diagnostics
                    .iter()
                    .any(|d| d.line == 4 && d.severity == "error"),
                "{}",
                failed.message
            );
            if kind == TruthScriptKind::Msg {
                fs::write(&input, &source).unwrap();
                crate::common::project_config::save_project_config(root, &project).unwrap();
                let check = crate::modules::mcp::tools::run_script(
                    &config,
                    Some(root),
                    kind,
                    "check",
                    input.to_string_lossy().into_owned(),
                    None,
                    None,
                )
                .unwrap();
                assert_eq!(check["success"], true);
                assert_eq!(check["checkOnly"], true);
                assert!(check["outputPath"].is_null());
                let lookup = crate::modules::mcp::tools::lookup_script(
                    &config,
                    Some(root),
                    kind,
                    input.to_str().unwrap(),
                    "sourceName",
                )
                .unwrap();
                assert_eq!(lookup["instructions"][0]["opcode"], opcode);
                assert_eq!(lookup["instructions"][0]["name"], "sourceName");
            }
            request.output_path = Some(input.to_string_lossy().into_owned());
            assert!(!run(&config, Some(root), Some(&project), &request).success);
        }
    }
}
