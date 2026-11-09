//! Thin desktop adapter for the shared headless checker.
use super::{
    compiler::{self, ScriptResult},
    maps::TruthScriptKind,
};
use crate::{common::project_config::ProjectConfig, config::AppConfig};
pub fn check(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    input_path: &str,
    kind: TruthScriptKind,
    text: &str,
) -> Result<ScriptResult, String> {
    check_with_encoding(config, root, project, input_path, kind, text, None)
}
pub fn check_with_encoding(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    input_path: &str,
    kind: TruthScriptKind,
    text: &str,
    encoding: Option<&str>,
) -> Result<ScriptResult, String> {
    let context =
        compiler::resolve_script_context(config, root, project, kind, input_path, encoding)?;
    script_check::run(
        &context,
        &script_check::ScriptRequest {
            operation: script_check::ScriptOperation::Check,
            input_path: input_path.into(),
            output_path: None,
            source_text: Some(text.into()),
        },
    )
    .map(compiler::adapt_result)
}
#[tauri::command]
pub async fn check_truth_source(
    state: tauri::State<'_, crate::app_state::AppState>,
    input_path: String,
    script_kind: TruthScriptKind,
    source_text: String,
) -> Result<ScriptResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let config = state.config_manager.get_config();
    tauri::async_runtime::spawn_blocking(move || {
        check(
            &config,
            root.as_deref(),
            None,
            &input_path,
            script_kind,
            &source_text,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
