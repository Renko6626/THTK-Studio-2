use super::{MsgResult, MsgScriptKind};
use crate::{
    app_state::AppState,
    common::toolchain,
    modules::truth::compiler::{self, ScriptMode, ScriptRequest},
};
use tauri::State;

async fn execute(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    encoding: Option<String>,
    script_kind: Option<MsgScriptKind>,
    mode: ScriptMode,
) -> Result<MsgResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let kind = script_kind
        .unwrap_or_else(|| MsgScriptKind::infer_from_path(&input_path))
        .truth_kind();
    tauri::async_runtime::spawn_blocking(move || {
        compiler::run(
            &ctx.config,
            root.as_deref(),
            ctx.project.as_ref(),
            &ScriptRequest {
                mode,
                kind,
                input_path,
                output_path,
                encoding,
            },
        )
    })
    .await
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn decompile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    encoding: Option<String>,
    script_kind: Option<MsgScriptKind>,
) -> Result<MsgResult, String> {
    execute(
        state,
        input_path,
        output_path,
        encoding,
        script_kind,
        ScriptMode::Decompile,
    )
    .await
}
#[tauri::command]
pub async fn compile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    encoding: Option<String>,
    script_kind: Option<MsgScriptKind>,
) -> Result<MsgResult, String> {
    execute(
        state,
        input_path,
        output_path,
        encoding,
        script_kind,
        ScriptMode::Compile,
    )
    .await
}
