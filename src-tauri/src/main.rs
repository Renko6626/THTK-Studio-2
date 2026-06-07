#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_state;
mod common;
mod config;
mod utils;

use app_state::AppState;
use common::file_watcher;
use common::fs_utils;
use common::project_config;
use common::toolchain;
use common::terminal;
use common::system_clipboard;
use config::AppConfig;
use tauri::State;

mod modules; // 确保 src-tauri/src/modules/mod.rs 存在且包含 pub mod ecl;
use modules::ecl::commands::{
    compile_ecl_file, decompile_ecl_file, generate_ai_assist_pack, get_ecl_map_semantics,
    get_thecl_status, run_thecl_operation,
};

use common::fs_ops;
// ----------------------------------------------------------------
// Config Commands
// ----------------------------------------------------------------

#[tauri::command]
fn get_settings(state: State<AppState>) -> AppConfig {
    state.config_manager.get_config()
}

#[tauri::command]
fn save_settings(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    state.config_manager.update_config(config)
}

#[tauri::command]
fn get_toolchain_status(state: State<AppState>, tool: String) -> Result<toolchain::ToolchainStatus, String> {
    let config = state.config_manager.get_config();
    toolchain::get_toolchain_status(&config, &tool)
}

#[tauri::command]
fn get_toolchain_statuses(state: State<AppState>) -> Vec<toolchain::ToolchainStatus> {
    let config = state.config_manager.get_config();
    toolchain::get_all_toolchain_statuses(&config)
}

// ----------------------------------------------------------------
// File System Commands (支持多标签页的基础)
// ----------------------------------------------------------------

// 读取文件内容
// 前端根据文件后缀决定怎么渲染，后端负责把字节流变成字符串
#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    utils::read_text_file(&path).map_err(|e| e.to_string())
}

// 保存文件
// is_source: true 表示这是 .decl/.dmsg (保存为 UTF-8)
// is_source: false 表示这是原始 txt (保存为 Shift-JIS)
#[tauri::command]
fn save_file(path: String, content: String, is_source: bool) -> Result<(), String> {
    if is_source {
        utils::write_file_utf8(&path, &content).map_err(|e| e.to_string())
    } else {
        utils::write_file_sjis(&path, &content).map_err(|e| e.to_string())
    }
}

// ----------------------------------------------------------------
// Project / Workspace Commands
// ----------------------------------------------------------------

// 设置当前项目根目录，并启动文件变更监听
#[tauri::command]
fn set_project_root(state: State<AppState>, path: String, app_handle: tauri::AppHandle) {
    let mut root = state.current_project_root.lock().unwrap_or_else(|e| e.into_inner());
    *root = Some(path.clone());
    drop(root);

    file_watcher::start_watching(&state.file_watcher, &app_handle, &path);

    // 取出端口/token 后立即放锁,避免持锁做文件 IO(pty_create 等会争用这把锁)
    let endpoint = {
        let mcp = state.mcp_server.lock().unwrap_or_else(|e| e.into_inner());
        mcp.as_ref().map(|info| (info.port, info.token.clone()))
    };

    if let Some((port, token)) = endpoint {
        // 项目根就绪后,把 MCP server 接入信息写进各客户端配置(非破坏性)。
        let cards = common::mcp_config::register_clients(&path, port, &token);
        use tauri::Emitter;
        for card in cards {
            let _ = app_handle.emit(
                "mcp://report",
                serde_json::json!({
                    "title": card.title,
                    "body": card.body,
                    "level": card.level,
                    "path": null,
                }),
            );
        }
    }
}

// ----------------------------------------------------------------
// Project Config Commands
// ----------------------------------------------------------------

#[tauri::command]
fn load_project_config(state: State<AppState>) -> Option<project_config::ProjectConfig> {
    let root = state.current_project_root.lock().unwrap_or_else(|e| e.into_inner());
    let root_path = root.as_deref()?;
    project_config::load_project_config(root_path)
}

#[tauri::command]
fn save_project_config_cmd(
    state: State<AppState>,
    config: project_config::ProjectConfig,
) -> Result<(), String> {
    let root = state.current_project_root.lock().unwrap_or_else(|e| e.into_inner());
    let root_path = root.as_deref().ok_or("No project root set")?;
    project_config::save_project_config(root_path, &config)
}

// ----------------------------------------------------------------
// File Tree Commands
// ----------------------------------------------------------------

#[tauri::command]
fn get_file_tree(path: String) -> Result<Vec<fs_utils::FileNode>, String> {
    fs_utils::get_file_tree(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dir_children(path: String) -> Result<Vec<fs_utils::FileNode>, String> {
    fs_utils::get_dir_children(&path).map_err(|e| e.to_string())
}

// ----------------------------------------------------------------
// Main Entry
// ----------------------------------------------------------------

fn main() {
    // 初始化应用状态
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // 注册 State
        .manage(app_state)
        .setup(|app| {
            // MCP server：绑定很快，同步等待拿到端口；失败不阻止应用启动
            use tauri::Manager;
            let handle = app.handle().clone();
            match tauri::async_runtime::block_on(modules::mcp::server::start(handle)) {
                Ok(info) => {
                    let state = app.state::<AppState>();
                    *state.mcp_server.lock().unwrap_or_else(|e| e.into_inner()) = Some(info);
                }
                Err(error) => {
                    eprintln!("[mcp] failed to start MCP server, agent tools unavailable: {error}");
                }
            }
            Ok(())
        })
        // 注册所有命令
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_toolchain_status,
            get_toolchain_statuses,
            read_file,
            save_file,
            set_project_root,
            load_project_config,
            save_project_config_cmd,
            get_file_tree,
            get_dir_children,
            compile_ecl_file,
            decompile_ecl_file,
            get_thecl_status,
            get_ecl_map_semantics,
            run_thecl_operation,
            generate_ai_assist_pack,
            fs_ops::create_directory,
            fs_ops::create_file,
            fs_ops::rename_entry,
            fs_ops::copy_entry,
            fs_ops::delete_entry,
            system_clipboard::set_file_clipboard,
            system_clipboard::get_file_clipboard,
            terminal::run_shell_command,
            terminal::resolve_directory,
            common::pty::pty_create,
            common::pty::pty_write,
            common::pty::pty_resize,
            common::pty::pty_kill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
