use super::error_parser::{self, Diagnostic};
use crate::config::AppConfig;
use crate::common::cmd_runner;
use crate::common::toolchain;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TheclMode {
    Compile,
    Decompile,
    Header,
}

impl TheclMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TheclMode::Compile => "compile",
            TheclMode::Decompile => "decompile",
            TheclMode::Header => "header",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TheclRequest {
    pub mode: TheclMode,
    pub version: String,
    pub input_path: String,
    pub output_path: Option<String>,
    #[serde(default)]
    pub map_paths: Vec<String>,
    #[serde(default)]
    pub use_shift_jis: bool,
    #[serde(default)]
    pub raw_dump: bool,
    #[serde(default)]
    pub simple_creation: bool,
    #[serde(default)]
    pub show_offsets: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclResult {
    pub success: bool,
    pub tool: String,
    pub mode: String,
    pub script_kind: String,
    pub input_path: String,
    pub message: String,
    pub diagnostics: Vec<Diagnostic>,
    pub output_path: Option<String>,
}

pub fn run(config: &AppConfig, request: &TheclRequest) -> EclResult {
    // 版本在边界处解析一次。run() 返回的是 EclResult 而不是 Result，所以用不了 `?`；
    // 走与下方 run_tool 失败路径一致的结构化失败结果，错误经既有渠道进问题面板。
    let version_id = match crate::common::game_version::parse(&request.version) {
        Ok(id) => id,
        Err(message) => {
            return EclResult {
                success: false,
                tool: "thecl".to_string(),
                mode: request.mode.as_str().to_string(),
                script_kind: "ecl".to_string(),
                input_path: request.input_path.clone(),
                message,
                diagnostics: Vec::new(),
                output_path: None,
            }
        }
    };

    let tool_path = toolchain::resolve_tool_path(config, "thecl", "thecl.exe");
    let output_path = infer_output_path(request);
    let args = build_thecl_args(request, version_id, output_path.as_deref());
    let arg_refs = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();
    let work_dir = Path::new(&request.input_path).parent();

    let result = cmd_runner::run_tool(&tool_path, &arg_refs, work_dir)
        .unwrap_or_else(|err_msg| cmd_runner::CommandResult {
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to launch thecl: {}", err_msg),
            exit_code: None,
        });

    let combined_output = format!("{}\n{}", result.stdout, result.stderr);
    let diagnostics = error_parser::parse_thecl_output(&combined_output);

    // 将诊断中的相对路径解析为绝对路径，确保前端能正确匹配 Monaco model
    let diagnostics = diagnostics
        .into_iter()
        .map(|mut d| {
            let p = Path::new(&d.path);
            if !p.is_absolute() {
                if let Some(wd) = work_dir {
                    d.path = wd.join(p).to_string_lossy().to_string();
                }
            }
            d
        })
        .collect();

    EclResult {
        success: result.success,
        tool: "thecl".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "ecl".to_string(),
        input_path: request.input_path.clone(),
        message: combined_output,
        diagnostics,
        output_path: if result.success { output_path } else { None },
    }
}

pub fn build_thecl_args(
    request: &TheclRequest,
    version_id: u32,
    output_path: Option<&str>,
) -> Vec<String> {
    let mut args = Vec::new();

    match request.mode {
        TheclMode::Compile => {
            args.push("-c".to_string());
            args.push(version_id.to_string());
            if request.simple_creation {
                args.push("-s".to_string());
            }
        }
        TheclMode::Decompile => {
            args.push("-d".to_string());
            args.push(version_id.to_string());
            if request.raw_dump {
                args.push("-r".to_string());
            }
            if request.show_offsets {
                args.push("-x".to_string());
            }
        }
        TheclMode::Header => {
            args.push("-h".to_string());
            args.push(version_id.to_string());
        }
    }

    if request.use_shift_jis {
        args.push("-j".to_string());
    }

    for map_path in &request.map_paths {
        args.push("-m".to_string());
        args.push(map_path.clone());
    }

    args.push(request.input_path.clone());
    if let Some(output_path) = output_path {
        args.push(output_path.to_string());
    }

    args
}

pub fn infer_output_path(request: &TheclRequest) -> Option<String> {
    if let Some(output_path) = &request.output_path {
        return Some(output_path.clone());
    }

    let inferred = match request.mode {
        TheclMode::Compile => swap_or_append_extension(&request.input_path, ".decl", ".ecl"),
        TheclMode::Decompile => swap_or_append_extension(&request.input_path, ".ecl", ".decl"),
        TheclMode::Header => swap_or_append_extension(&request.input_path, ".decl", ".h"),
    };

    Some(inferred)
}

fn swap_or_append_extension(path: &str, from: &str, to: &str) -> String {
    let replaced = path.replace(from, to);
    if replaced == path {
        format!("{}{}", path, to)
    } else {
        replaced
    }
}
