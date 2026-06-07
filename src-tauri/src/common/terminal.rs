use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize)]
pub struct ShellCommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub cwd: String,
    pub shell: String,
}

#[tauri::command]
pub fn run_shell_command(
    shell: String,
    command: String,
    cwd: Option<String>,
) -> Result<ShellCommandResult, String> {
    let shell_kind = normalize_shell_kind(&shell);
    let working_dir = determine_working_directory(cwd.as_deref())?;

    let mut process = match shell_kind.as_str() {
        "cmd" => {
            let mut cmd = Command::new("cmd.exe");
            cmd.args(["/C", &command]);
            cmd
        }
        _ => {
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoLogo", "-NoProfile", "-Command", &command]);
            cmd
        }
    };

    process.current_dir(&working_dir);

    #[cfg(target_os = "windows")]
    {
        process.creation_flags(CREATE_NO_WINDOW);
    }

    let output = process
        .output()
        .map_err(|e| format!("Failed to spawn shell '{}': {}", shell_kind, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(ShellCommandResult {
        success: output.status.success(),
        stdout,
        stderr,
        exit_code: output.status.code(),
        cwd: path_to_string(&working_dir),
        shell: shell_kind,
    })
}

#[tauri::command]
pub fn resolve_directory(base_dir: Option<String>, target: String) -> Result<String, String> {
    let base = determine_working_directory(base_dir.as_deref())?;
    let trimmed = target.trim();

    let candidate = if trimmed.is_empty() || trimmed == "." {
        base
    } else {
        let target_path = PathBuf::from(trimmed);
        if target_path.is_absolute() {
            target_path
        } else {
            base.join(target_path)
        }
    };

    if !candidate.exists() || !candidate.is_dir() {
        return Err(format!("Directory not found: {}", path_to_string(&candidate)));
    }

    Ok(path_to_string(&candidate))
}

fn determine_working_directory(cwd: Option<&str>) -> Result<PathBuf, String> {
    match cwd {
        Some(path) if !path.trim().is_empty() => {
            let path_buf = PathBuf::from(path.trim());
            if !path_buf.exists() || !path_buf.is_dir() {
                return Err(format!("Invalid working directory: {}", path));
            }
            Ok(path_buf)
        }
        _ => std::env::current_dir().map_err(|e| e.to_string()),
    }
}

fn normalize_shell_kind(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "cmd" | "cmd.exe" => "cmd".to_string(),
        _ => "pwsh".to_string(),
    }
}

fn path_to_string(path: &Path) -> String {
    let display = path.to_string_lossy().to_string();
    if let Some(stripped) = display.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        display
    }
}
