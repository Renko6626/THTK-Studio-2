use crate::common::cmd_runner;
use crate::common::toolchain;
use crate::config::AppConfig;
use crate::modules::ecl::error_parser::Diagnostic;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StdMode {
    Decompile,
    Compile,
}

impl StdMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StdMode::Decompile => "decompile",
            StdMode::Compile => "compile",
        }
    }
}

fn default_with_comments() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StdRequest {
    pub mode: StdMode,
    pub version: String,
    pub input_path: String,
    pub output_path: Option<String>,
    /// decompile 时是否在每行末尾追加 ` // <description>`;compile 路径忽略此字段
    #[serde(default = "default_with_comments")]
    pub with_comments: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StdResult {
    pub success: bool,
    pub tool: String,        // "thstd"
    pub mode: String,        // "decompile" | "compile"
    pub script_kind: String, // "std"
    pub input_path: String,
    pub output_path: Option<String>,
    pub message: String,
    /// 始终为空;复用 ECL 的 Diagnostic 类型以便前端面板复用。
    pub diagnostics: Vec<Diagnostic>,
}

/// thstd 与 thecl/thmsg 版本归一化一致:`th17` → `17`,`17` → `17`。
pub fn normalize_thstd_version(version: &str) -> String {
    let trimmed = version.trim().to_lowercase();
    if let Some(stripped) = trimmed.strip_prefix("th") {
        return stripped.to_string();
    }
    trimmed
}

/// 推断输出路径:
///   - decompile: `.std` → `.dstd`(同目录同 stem)
///   - compile:   `.dstd` → `.std`
///   - 找不到对应扩展则直接追加。
pub fn infer_output_path(request: &StdRequest) -> String {
    if let Some(output_path) = &request.output_path {
        return output_path.clone();
    }
    match request.mode {
        StdMode::Decompile => swap_or_append_extension(&request.input_path, ".std", ".dstd"),
        StdMode::Compile => swap_or_append_extension(&request.input_path, ".dstd", ".std"),
    }
}

fn swap_or_append_extension(path: &str, from: &str, to: &str) -> String {
    let replaced = path.replace(from, to);
    if replaced == path {
        format!("{}{}", path, to)
    } else {
        replaced
    }
}

fn fail(request: &StdRequest, msg: impl Into<String>) -> StdResult {
    StdResult {
        success: false,
        tool: "thstd".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "std".to_string(),
        input_path: request.input_path.clone(),
        output_path: None,
        message: msg.into(),
        diagnostics: Vec::new(),
    }
}

/// 生成跨调用唯一的临时文件路径(进程 ID + 调用级计数器)。
fn unique_temp_path(stem: &str, ext: &str) -> std::path::PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "thtk-std-{}-{seq}-{stem}.{ext}",
        std::process::id()
    ))
}

pub fn run(config: &AppConfig, request: &StdRequest) -> StdResult {
    let tool_path = toolchain::resolve_tool_path(config, "thstd", "thstd.exe");
    if tool_path.trim().is_empty() {
        return fail(request, &crate::common::toolchain::not_configured_message("thstd"));
    }

    match request.mode {
        StdMode::Decompile => run_decompile(&tool_path, request),
        StdMode::Compile => run_compile(&tool_path, request),
    }
}

fn run_decompile(tool_path: &str, request: &StdRequest) -> StdResult {
    let version = normalize_thstd_version(&request.version);
    let output_path = infer_output_path(request);
    let input_parent = Path::new(&request.input_path).parent();

    // thstd -d 把输出写到第 4 个 CLI 参数指定的文件(UTF-8)
    let stem = Path::new(&request.input_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());
    let temp_path = unique_temp_path(&stem, "dstd.raw");
    let temp_str = temp_path.to_string_lossy().to_string();

    let args: [&str; 4] = [
        "-d",
        version.as_str(),
        request.input_path.as_str(),
        temp_str.as_str(),
    ];

    let exec = match cmd_runner::run_tool(tool_path, &args, input_parent) {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return fail(request, format!("Failed to launch thstd: {e}"));
        }
    };

    if !exec.success {
        let _ = std::fs::remove_file(&temp_path);
        let code_str = exec
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "<signal>".to_string());
        let mut msg = format!("thstd exited with code {code_str}\n");
        msg.push_str(&exec.stdout);
        msg.push('\n');
        msg.push_str(&exec.stderr);
        return fail(request, msg);
    }

    // 读临时输出。thstd 文件里没有日文，正常就是纯 ASCII/UTF-8；
    // shift-jis 只作老文件兜底。
    let raw_dstd = match utils::read_text_file(&temp_path, "shift-jis") {
        Ok(s) => s,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return fail(
                request,
                format!("Failed to read thstd output {}: {e}", temp_path.display()),
            );
        }
    };
    let _ = std::fs::remove_file(&temp_path);

    // 拿不到语义不该让整个解包失败——用户仍然需要看到 ins_N 形式的内容。
    // 但必须说清楚为什么没有名字（例如 th13 及更早属于 formats_v1，尚未迁移）。
    let (semantics, semantics_note) = match super::map_parser::parse_std_semantics(&version) {
        Ok(s) => (Some(s), None),
        Err(e) => (None, Some(e)),
    };
    let readable = match &semantics {
        Some(s) => super::translator::dstd_to_readable(&raw_dstd, s, request.with_comments),
        None => raw_dstd.clone(),
    };

    if let Err(e) = utils::write_file_utf8(&output_path, &readable) {
        return fail(request, format!("Failed to write {output_path}: {e}"));
    }

    let stderr_trim = exec.stderr.trim();
    let mut message = if stderr_trim.is_empty() {
        format!("Decompiled {} → {}", request.input_path, output_path)
    } else {
        stderr_trim.to_string()
    };
    // 语义表缺失是"能用但没名字"，属于提示不是错误——必须让用户看见，
    // 否则他会以为这些 ins_N 就是本来的样子。
    if let Some(note) = semantics_note {
        message.push_str("\n\nℹ ");
        message.push_str(&note);
    }

    StdResult {
        success: true,
        tool: "thstd".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "std".to_string(),
        input_path: request.input_path.clone(),
        output_path: Some(output_path),
        message,
        diagnostics: Vec::new(),
    }
}

fn run_compile(tool_path: &str, request: &StdRequest) -> StdResult {
    let version = normalize_thstd_version(&request.version);
    let output_path = infer_output_path(request);

    // 1. 读源文件（见上，UTF-8 优先）
    let content = match utils::read_text_file(&request.input_path, "shift-jis") {
        Ok(c) => c,
        Err(e) => {
            return fail(
                request,
                format!("Failed to read {}: {e}", request.input_path),
            )
        }
    };

    // 2. 翻译回 raw dstd
    let semantics = match super::map_parser::parse_std_semantics(&version) {
        Ok(s) => s,
        Err(e) => return fail(request, format!("Failed to load std semantics: {e}")),
    };
    let raw_dstd = super::translator::readable_to_dstd(&content, &semantics);

    // 3. 写 UTF-8 临时文件
    let stem = Path::new(&request.input_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "input".to_string());
    let temp_path = unique_temp_path(&stem, "dstd");
    if let Err(e) = utils::write_file_utf8(&temp_path, &raw_dstd) {
        return fail(
            request,
            format!("Failed to write temp file {}: {e}", temp_path.display()),
        );
    }

    // 4. 跑 `thstd -c <ver> <temp> <out>`
    let temp_str = temp_path.to_string_lossy().to_string();
    let args: [&str; 4] = [
        "-c",
        version.as_str(),
        temp_str.as_str(),
        output_path.as_str(),
    ];
    let work_dir = Path::new(&output_path).parent();

    let exec = match cmd_runner::run_tool(tool_path, &args, work_dir) {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return fail(request, format!("Failed to launch thstd: {e}"));
        }
    };
    let _ = std::fs::remove_file(&temp_path); // best-effort cleanup

    let combined = format!("{}\n{}", exec.stdout, exec.stderr)
        .trim()
        .to_string();

    if !exec.success {
        let code_str = exec
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "<signal>".to_string());
        let mut msg = format!("thstd exited with code {code_str}\n");
        msg.push_str(&combined);
        return fail(request, msg);
    }

    let message = if combined.is_empty() {
        format!("Compiled {} → {}", request.input_path, output_path)
    } else {
        combined
    };

    StdResult {
        success: true,
        tool: "thstd".to_string(),
        mode: request.mode.as_str().to_string(),
        script_kind: "std".to_string(),
        input_path: request.input_path.clone(),
        output_path: Some(output_path),
        message,
        diagnostics: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(mode: StdMode, input: &str, output: Option<&str>) -> StdRequest {
        StdRequest {
            mode,
            version: "17".to_string(),
            input_path: input.to_string(),
            output_path: output.map(|s| s.to_string()),
            with_comments: true,
        }
    }

    #[test]
    fn infer_output_path_decompile_swaps_std_to_dstd() {
        let r = req(StdMode::Decompile, "/proj/stage01.std", None);
        assert_eq!(infer_output_path(&r), "/proj/stage01.dstd");
    }

    #[test]
    fn infer_output_path_compile_swaps_dstd_to_std() {
        let r = req(StdMode::Compile, "/proj/stage01.dstd", None);
        assert_eq!(infer_output_path(&r), "/proj/stage01.std");
    }

    #[test]
    fn normalize_thstd_version_strips_th_prefix() {
        assert_eq!(normalize_thstd_version("th17"), "17");
        assert_eq!(normalize_thstd_version("17"), "17");
        assert_eq!(normalize_thstd_version("  Th18 "), "18");
    }

    #[test]
    fn run_returns_failure_when_thstd_path_not_configured() {
        // 用默认 AppConfig(thtk_dir 空、tool_overrides 空)走 run(),应直接拿到 success:false。
        let cfg = AppConfig::default();
        let r = req(StdMode::Decompile, "/proj/stage01.std", None);
        let result = run(&cfg, &r);
        assert!(!result.success);
        assert!(
            result.message.contains("thstd") && result.message.contains("工具链设置"),
            "got message: {}",
            result.message
        );
        assert!(result.output_path.is_none());
        assert_eq!(result.tool, "thstd");
        assert_eq!(result.mode, "decompile");
        assert_eq!(result.script_kind, "std");
    }
}
