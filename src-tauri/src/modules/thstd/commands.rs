use super::compiler;
use crate::app_state::AppState;
use crate::common::toolchain;
use tauri::State;

fn ensure_thstd_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thstd", "thstd.exe");
    if resolved.trim().is_empty() {
        return Err(toolchain::not_configured_message("thstd"));
    }
    Ok(())
}

#[tauri::command]
pub async fn decompile_std_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    with_comments: Option<bool>,
) -> Result<compiler::StdResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
    ensure_thstd_configured(&config)?;
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &config, "thstd")?
            .to_string();
    let req = compiler::StdRequest {
        mode: compiler::StdMode::Decompile,
        version,
        input_path,
        output_path,
        with_comments: with_comments.unwrap_or(true),
    };
    Ok(compiler::run(&config, &req))
}

#[tauri::command]
pub async fn compile_std_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
) -> Result<compiler::StdResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
    ensure_thstd_configured(&config)?;
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &config, "thstd")?
            .to_string();
    let req = compiler::StdRequest {
        mode: compiler::StdMode::Compile,
        version,
        input_path,
        output_path,
        with_comments: false,
    };
    Ok(compiler::run(&config, &req))
}
