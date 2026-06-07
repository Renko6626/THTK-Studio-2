use serde::Serialize;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize)]
pub struct FileClipboardPayload {
    pub paths: Vec<String>,
}

#[tauri::command]
pub fn set_file_clipboard(paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let json = serde_json::to_string(&paths).map_err(|e| e.to_string())?;
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$paths = $env:THTK_FILE_CLIPBOARD | ConvertFrom-Json
$collection = New-Object System.Collections.Specialized.StringCollection
foreach ($path in $paths) { [void]$collection.Add([string]$path) }
[System.Windows.Forms.Clipboard]::SetFileDropList($collection)
"#;

    run_powershell(script, &[("THTK_FILE_CLIPBOARD", json)])?;
    Ok(())
}

#[tauri::command]
pub fn get_file_clipboard() -> Result<FileClipboardPayload, String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
if ([System.Windows.Forms.Clipboard]::ContainsFileDropList()) {
  $items = @()
  foreach ($item in [System.Windows.Forms.Clipboard]::GetFileDropList()) { $items += [string]$item }
  $items | ConvertTo-Json -Compress
} else {
  '[]'
}
"#;

    let stdout = run_powershell(script, &[])?;
    let paths = serde_json::from_str::<Vec<String>>(stdout.trim()).map_err(|e| e.to_string())?;
    Ok(FileClipboardPayload { paths })
}

fn run_powershell(script: &str, envs: &[(&str, String)]) -> Result<String, String> {
    let mut command = Command::new("powershell.exe");
    command.args(["-NoLogo", "-NoProfile", "-STA", "-Command", script]);

    for (key, value) in envs {
        command.env(key, value);
    }

    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
