use super::compiler;
use crate::app_state::AppState;
use crate::common::{project_config, toolchain};
use tauri::State;

/// Project game_version takes priority; fallback to global default_game_version.
fn effective_thdat_version(
    config: &crate::config::AppConfig,
    project_root: Option<&str>,
) -> String {
    let mut version = config.default_game_version.clone();
    if let Some(root) = project_root {
        if let Some(pc) = project_config::load_project_config(root) {
            if !pc.game_version.is_empty() {
                version = pc.game_version.clone();
            }
        }
    }
    version
}

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
    let config = state.config_manager.get_config();
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
    let config = state.config_manager.get_config();
    ensure_thdat_configured(&config)?;
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let version = effective_thdat_version(&config, root.as_deref());
    let req = compiler::ThdatRequest {
        mode: compiler::ThdatMode::Pack,
        version,
        archive_path,
        target_dir: source_dir,
    };
    Ok(compiler::run(&config, &req))
}
