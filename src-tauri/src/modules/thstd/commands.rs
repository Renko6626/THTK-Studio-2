use crate::{
    app_state::AppState,
    common::toolchain,
    modules::truth::{
        compiler::{self, ScriptMode, ScriptRequest, ScriptResult},
        maps::TruthScriptKind,
    },
};
use tauri::State;
async fn execute(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    mode: ScriptMode,
) -> Result<ScriptResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    tauri::async_runtime::spawn_blocking(move || {
        compiler::run(
            &ctx.config,
            root.as_deref(),
            ctx.project.as_ref(),
            &ScriptRequest {
                mode,
                kind: TruthScriptKind::Std,
                input_path,
                output_path,
                encoding: None,
            },
        )
    })
    .await
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn decompile_std_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
) -> Result<ScriptResult, String> {
    execute(state, input_path, output_path, ScriptMode::Decompile).await
}
#[tauri::command]
pub async fn compile_std_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
) -> Result<ScriptResult, String> {
    execute(state, input_path, output_path, ScriptMode::Compile).await
}
