use crate::common::{cmd_runner, toolchain};
use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ThdatMode {
    Extract,
    Pack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThdatRequest {
    pub mode: ThdatMode,
    /// Extract: ignored (uses `-xd` auto-detect); Pack: required, normalized
    pub version: String,
    pub archive_path: String,
    pub target_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThdatResult {
    pub success: bool,
    pub tool: String,         // "thdat"
    pub mode: String,         // "extract" | "pack"
    pub archive_path: String,
    pub target_dir: String,
    pub message: String,
    /// Always empty; frontend panel reuses ECL Diagnostic shape for compatibility.
    pub diagnostics: Vec<crate::modules::ecl::error_parser::Diagnostic>,
    pub file_count: Option<usize>,
}

/// Strip "th" prefix, lowercase, trim — same convention as ecl/msg/std.
pub fn normalize_thdat_version(version: &str) -> String {
    let v = version.trim().to_lowercase();
    v.strip_prefix("th").unwrap_or(&v).to_string()
}

/// Windows CreateProcess command-line limit is 32 KB; leave 4 KB headroom.
const MAX_CMDLINE_BYTES: usize = 28000;

pub fn build_thdat_extract_args(archive: &str, target: &str) -> Vec<String> {
    vec![
        "-xd".to_string(),
        archive.to_string(),
        "-C".to_string(),
        target.to_string(),
    ]
}

/// Build pack args. Returns Err if the total estimated command-line length
/// would exceed MAX_CMDLINE_BYTES (batching is not implemented this release).
pub fn build_thdat_pack_args(
    version: &str,
    archive: &str,
    target_dir: &str,
    files: &[String],
) -> Result<Vec<String>, String> {
    let normalized = normalize_thdat_version(version);
    if normalized.is_empty() {
        return Err("thdat pack version is empty".to_string());
    }
    let mut args = vec![
        format!("-c{normalized}"),
        archive.to_string(),
        "-C".to_string(),
        target_dir.to_string(),
    ];
    for f in files {
        args.push(f.clone());
    }
    let total: usize = args.iter().map(|s| s.len() + 1).sum();
    if total > MAX_CMDLINE_BYTES {
        return Err(format!(
            "too many files for thdat ({} bytes, max {}); batching not implemented",
            total, MAX_CMDLINE_BYTES
        ));
    }
    Ok(args)
}

pub fn run(config: &AppConfig, request: &ThdatRequest) -> ThdatResult {
    let tool_path = toolchain::resolve_tool_path(config, "thdat", "thdat.exe");
    if tool_path.trim().is_empty() {
        return failure(request, crate::common::toolchain::not_configured_message("thdat"), None);
    }
    match request.mode {
        ThdatMode::Extract => run_extract(&tool_path, request),
        ThdatMode::Pack => run_pack(&tool_path, request),
    }
}

fn mode_str(m: &ThdatMode) -> &'static str {
    match m {
        ThdatMode::Extract => "extract",
        ThdatMode::Pack => "pack",
    }
}

fn run_extract(tool_path: &str, request: &ThdatRequest) -> ThdatResult {
    // Ensure target directory exists before invoking tool.
    if let Err(e) = fs::create_dir_all(&request.target_dir) {
        return failure(request, format!("failed to create target dir: {e}"), None);
    }
    let args = build_thdat_extract_args(&request.archive_path, &request.target_dir);
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = cmd_runner::run_tool(tool_path, &arg_refs, None).unwrap_or_else(|err| {
        cmd_runner::CommandResult {
            success: false,
            stdout: String::new(),
            stderr: format!("failed to launch thdat: {err}"),
            exit_code: None,
        }
    });
    let combined = format!("{}\n{}", result.stdout, result.stderr)
        .trim()
        .to_string();
    if !result.success {
        return failure(request, combined, None);
    }
    // Count files at top level of target_dir for UI reporting.
    let file_count = count_files_shallow(&request.target_dir);
    ThdatResult {
        success: true,
        tool: "thdat".to_string(),
        mode: "extract".to_string(),
        archive_path: request.archive_path.clone(),
        target_dir: request.target_dir.clone(),
        message: if combined.is_empty() {
            format!("Extracted {} files", file_count.unwrap_or(0))
        } else {
            combined
        },
        diagnostics: vec![],
        file_count,
    }
}

fn run_pack(tool_path: &str, request: &ThdatRequest) -> ThdatResult {
    // List only top-level files from the source directory (thdat does not support nested dirs).
    let files = match list_files_shallow(&request.target_dir) {
        Ok(f) => f,
        Err(e) => return failure(request, format!("failed to list source dir: {e}"), None),
    };
    if files.is_empty() {
        return failure(
            request,
            "source directory is empty".to_string(),
            Some(0),
        );
    }
    let file_count = files.len();
    let args = match build_thdat_pack_args(
        &request.version,
        &request.archive_path,
        &request.target_dir,
        &files,
    ) {
        Ok(a) => a,
        Err(e) => return failure(request, e, Some(file_count)),
    };
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = cmd_runner::run_tool(tool_path, &arg_refs, None).unwrap_or_else(|err| {
        cmd_runner::CommandResult {
            success: false,
            stdout: String::new(),
            stderr: format!("failed to launch thdat: {err}"),
            exit_code: None,
        }
    });
    let combined = format!("{}\n{}", result.stdout, result.stderr)
        .trim()
        .to_string();
    if !result.success {
        return failure(request, combined, Some(file_count));
    }
    ThdatResult {
        success: true,
        tool: "thdat".to_string(),
        mode: "pack".to_string(),
        archive_path: request.archive_path.clone(),
        target_dir: request.target_dir.clone(),
        message: if combined.is_empty() {
            format!("Packed {} files", file_count)
        } else {
            combined
        },
        diagnostics: vec![],
        file_count: Some(file_count),
    }
}

fn failure(request: &ThdatRequest, message: String, file_count: Option<usize>) -> ThdatResult {
    ThdatResult {
        success: false,
        tool: "thdat".to_string(),
        mode: mode_str(&request.mode).to_string(),
        archive_path: request.archive_path.clone(),
        target_dir: request.target_dir.clone(),
        message,
        diagnostics: vec![],
        file_count,
    }
}

/// Return sorted list of filenames (not paths) for files at the top level of `dir`.
fn list_files_shallow(dir: &str) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {dir}: {e}"))?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            files.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    files.sort(); // deterministic ordering
    Ok(files)
}

fn count_files_shallow(dir: &str) -> Option<usize> {
    let entries = fs::read_dir(dir).ok()?;
    let n = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .count();
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_th_prefix() {
        assert_eq!(normalize_thdat_version("th17"), "17");
        assert_eq!(normalize_thdat_version("17"), "17");
        assert_eq!(normalize_thdat_version("  TH18 "), "18");
    }

    #[test]
    fn extract_args_shape() {
        let args = build_thdat_extract_args("/p/th17.dat", "/p/th17");
        assert_eq!(args, vec!["-xd", "/p/th17.dat", "-C", "/p/th17"]);
    }

    #[test]
    fn pack_args_include_version_and_files_sorted() {
        let files = vec!["a.ecl".to_string(), "b.anm".to_string()];
        let args =
            build_thdat_pack_args("th17", "/p/out.dat", "/p/src", &files).expect("ok");
        assert_eq!(args[0], "-c17");
        assert_eq!(args[1], "/p/out.dat");
        assert_eq!(args[2], "-C");
        assert_eq!(args[3], "/p/src");
        assert_eq!(&args[4..], &["a.ecl", "b.anm"]);
    }

    #[test]
    fn pack_fails_when_file_args_too_long() {
        // Manufacture a file list that pushes past MAX_CMDLINE_BYTES.
        let big: Vec<String> = (0..1000)
            .map(|i| format!("filename_with_some_padding_{i:04}.bin"))
            .collect();
        let err =
            build_thdat_pack_args("17", "/p/out.dat", "/p/src", &big).unwrap_err();
        assert!(err.contains("too many files"), "got: {err}");
    }

    #[test]
    fn pack_fails_when_version_empty() {
        let err =
            build_thdat_pack_args("", "/p/out.dat", "/p/src", &[]).unwrap_err();
        assert!(err.contains("version"), "got: {err}");
    }

    #[test]
    fn run_returns_failure_when_thdat_path_not_configured() {
        let config = crate::config::AppConfig::default();
        let req = ThdatRequest {
            mode: ThdatMode::Extract,
            version: String::new(),
            archive_path: "/tmp/x.dat".to_string(),
            target_dir: "/tmp/out".to_string(),
        };
        let result = run(&config, &req);
        assert!(!result.success);
        assert!(
            result.message.contains("thdat") && result.message.contains("工具链设置"),
            "got: {}",
            result.message
        );
    }
}
