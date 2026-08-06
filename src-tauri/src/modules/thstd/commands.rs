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

/// 导出不含方言声明的原始 .dstd（`thstd` 可直接编译的形式）。
///
/// 磁盘上的 .dstd 是 IDE 方言：指令名由本 IDE 映射，`thstd` 只认 `ins_N`。
/// 需要脱离 IDE 走命令行 / CI 时用这个入口。
#[tauri::command]
pub async fn export_raw_dstd(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
) -> Result<String, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &ctx.config, "thstd")?
            .to_string();
    let semantics = super::map_parser::parse_std_semantics(&version)?;

    let content = crate::utils::read_text_file(&input_path, "utf-8")
        .map_err(|e| format!("读取 {input_path} 失败: {e}"))?;
    // 方言头必须先剥掉——它不是指令，thstd 会把它当语法错误
    let stripped: String = content
        .lines()
        .filter(|line| !line.trim_start().starts_with(super::translator::DIALECT_MARKER))
        .collect::<Vec<_>>()
        .join("\n");
    let raw = super::translator::readable_to_dstd(&stripped, &semantics);
    crate::utils::write_file_utf8(&output_path, &raw)
        .map_err(|e| format!("写入 {output_path} 失败: {e}"))?;
    Ok(output_path)
}
