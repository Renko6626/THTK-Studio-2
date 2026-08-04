use super::compiler;
use crate::app_state::AppState;
use crate::common::toolchain;
use tauri::State;

/// 本次调用生效的游戏文本编码：显式指定 > 项目配置 > shift-jis。
///
/// 解包和打包**各自传各自的**——常见工作流是按 shift-jis 解开原版日文，
/// 翻译后按 gbk 打包成汉化版，同一个文件的一次往返就要用两种编码。
fn effective_encoding(
    explicit: Option<String>,
    project: Option<&crate::common::project_config::ProjectConfig>,
) -> String {
    if let Some(value) = explicit {
        if !value.trim().is_empty() {
            return value.trim().to_lowercase();
        }
    }
    project
        .map(|pc| pc.encoding.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| "shift-jis".to_string())
}

fn ensure_thmsg_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thmsg", "thmsg.exe");
    if resolved.trim().is_empty() {
        return Err(toolchain::not_configured_message("thmsg"));
    }
    Ok(())
}

/// 解包 .msg → .dmsg。`encoding` 是**游戏文本**的编码（不是 .dmsg 文件的，
/// 那个始终是 UTF-8）；留空则用项目配置，再空则 shift-jis。
#[tauri::command]
pub async fn decompile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    with_comments: Option<bool>,
    encoding: Option<String>,
) -> Result<compiler::MsgResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
    ensure_thmsg_configured(&config)?;
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &config, "thmsg")?
            .to_string();
    let req = compiler::MsgRequest {
        mode: compiler::MsgMode::Decompile,
        version,
        input_path,
        output_path,
        with_comments: with_comments.unwrap_or(true),
        encoding: effective_encoding(encoding, ctx.project.as_ref()),
    };
    Ok(compiler::run(&config, &req))
}

/// 打包 .dmsg → .msg。`encoding` 与解包**各自独立**——把原版日文按 shift-jis
/// 解开、翻译成中文后按 gbk 打包，是同一个文件一次往返里的常规操作。
#[tauri::command]
pub async fn compile_msg_file(
    state: State<'_, AppState>,
    input_path: String,
    output_path: Option<String>,
    encoding: Option<String>,
) -> Result<compiler::MsgResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
    ensure_thmsg_configured(&config)?;
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &config, "thmsg")?
            .to_string();
    let req = compiler::MsgRequest {
        mode: compiler::MsgMode::Compile,
        version,
        input_path,
        output_path,
        with_comments: false,
        encoding: effective_encoding(encoding, ctx.project.as_ref()),
    };
    Ok(compiler::run(&config, &req))
}
