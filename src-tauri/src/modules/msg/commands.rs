use super::compiler;
use crate::app_state::AppState;
use crate::common::{project_config, toolchain};
use tauri::State;

/// 项目 game_version 优先,fallback 到全局 default_game_version。
/// 不复用 mcp 模块里的 effective_toolchain_config——那是 ECL 专用的,带 eclmap 字段,
/// 对 msg 完全无用。
fn effective_msg_version(config: &crate::config::AppConfig, project_root: Option<&str>) -> String {
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

fn ensure_thmsg_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thmsg", "thmsg.exe");
    if resolved.trim().is_empty() {
        return Err("thmsg path is not configured".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn decompile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    with_comments: Option<bool>,
) -> Result<compiler::MsgResult, String> {
    let config = state.config_manager.get_config();
    ensure_thmsg_configured(&config)?;
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let version = effective_msg_version(&config, root.as_deref());
    let req = compiler::MsgRequest {
        mode: compiler::MsgMode::Decompile,
        version,
        input_path,
        output_path,
        with_comments: with_comments.unwrap_or(true),
    };
    Ok(compiler::run(&config, &req))
}

#[tauri::command]
pub async fn compile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
) -> Result<compiler::MsgResult, String> {
    let config = state.config_manager.get_config();
    ensure_thmsg_configured(&config)?;
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let version = effective_msg_version(&config, root.as_deref());
    let req = compiler::MsgRequest {
        mode: compiler::MsgMode::Compile,
        version,
        input_path,
        output_path,
        with_comments: false,
    };
    Ok(compiler::run(&config, &req))
}
