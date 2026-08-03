use super::compiler;
use crate::app_state::AppState;
use crate::common::toolchain;
use tauri::State;

fn ensure_thdat_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thdat", "thdat.exe");
    if resolved.trim().is_empty() {
        return Err("thdat path is not configured".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn extract_dat_file(
    state: State<'_, AppState>,
    archive_path: String,
    target_dir: String,
) -> Result<compiler::ThdatResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let config = toolchain::effective_config(&state.config_manager.get_config(), root.as_deref());
    ensure_thdat_configured(&config)?;
    // Extract uses -xd (auto-detect); version field is ignored.
    let req = compiler::ThdatRequest {
        mode: compiler::ThdatMode::Extract,
        version: String::new(),
        archive_path,
        target_dir,
    };
    Ok(compiler::run(&config, &req))
}

#[tauri::command]
pub async fn pack_dat_file(
    state: State<'_, AppState>,
    source_dir: String,
    archive_path: String,
) -> Result<compiler::ThdatResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let config = toolchain::effective_config(&state.config_manager.get_config(), root.as_deref());
    ensure_thdat_configured(&config)?;
    let version =
        crate::common::game_version::resolve(&config, root.as_deref(), "thdat")?
            .to_string();
    let req = compiler::ThdatRequest {
        mode: compiler::ThdatMode::Pack,
        version,
        archive_path,
        target_dir: source_dir,
    };
    Ok(compiler::run(&config, &req))
}
