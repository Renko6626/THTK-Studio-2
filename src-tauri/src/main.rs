#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_state;
mod common;
mod config;
mod utils;
#[cfg(test)]
mod wire_format_tests;

use app_state::AppState;
use common::file_watcher;
use common::fs_utils;
use common::project_config;
use common::recent_projects;
use common::toolchain;
use common::system_clipboard;
use config::AppConfig;
use tauri::State;

mod modules; // 确保 src-tauri/src/modules/mod.rs 存在且包含 pub mod ecl;
use modules::ecl::commands::{
    compile_ecl_file, decompile_ecl_file, generate_ai_assist_pack, get_ecl_map_semantics,
    run_thecl_operation,
};
use modules::msg::commands::{compile_msg_file, decompile_msg_file};
use modules::thdat::commands::{extract_dat_file, pack_dat_file};
use modules::thstd::commands::{compile_std_file, decompile_std_file};

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
    state.config_manager.update_user_settings(config)
}

/// 取当前项目根，供 toolchain::effective_config 应用项目级覆盖。
fn current_project_root(state: &State<AppState>) -> Option<String> {
    state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

// 状态查询同样要过项目级覆盖，否则设置界面显示的"已解析路径"会和
// 实际编译时用的 exe 不是同一个。
#[tauri::command]
fn get_toolchain_status(state: State<AppState>, tool: String) -> Result<toolchain::ToolchainStatus, String> {
    let root = current_project_root(&state);
    let config = toolchain::effective_config(&state.config_manager.get_config(), root.as_deref());
    toolchain::get_toolchain_status(&config, &tool)
}

#[tauri::command]
fn get_toolchain_statuses(state: State<AppState>) -> Vec<toolchain::ToolchainStatus> {
    let root = current_project_root(&state);
    let config = toolchain::effective_config(&state.config_manager.get_config(), root.as_deref());
    toolchain::get_all_toolchain_statuses(&config)
}

// ----------------------------------------------------------------
// File System Commands (支持多标签页的基础)
// ----------------------------------------------------------------

// 读取文件内容
// 前端根据文件后缀决定怎么渲染，后端负责把字节流变成字符串
// 守卫:path 必须在 project_root 之内,防止 webview 越界读盘。
#[tauri::command]
fn read_file(state: State<AppState>, path: String) -> Result<String, String> {
    fs_ops::guard_path(&state, &path)?;
    utils::read_text_file(&path).map_err(|e| e.to_string())
}

// 保存文件
// is_source: true 表示这是 .decl/.dmsg (保存为 UTF-8)
// is_source: false 表示这是原始 txt (保存为 Shift-JIS)
// 守卫:path 必须在 project_root 之内。
#[tauri::command]
fn save_file(
    state: State<AppState>,
    path: String,
    content: String,
    is_source: bool,
) -> Result<(), String> {
    fs_ops::guard_path(&state, &path)?;
    if is_source {
        utils::write_file_utf8(&path, &content).map_err(|e| e.to_string())
    } else {
        utils::write_file_sjis(&path, &content).map_err(|e| e.to_string())
    }
}

// ----------------------------------------------------------------
// Project / Workspace Commands
// ----------------------------------------------------------------

/// 打开项目的返回值。一次调用把前端需要的三件事一起给出，
/// 避免"设根目录 → 取文件树 → 读配置"三步中途失败留下半个状态。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectOpenResult {
    root_path: String,
    files: Vec<fs_utils::FileNode>,
    project_config: project_config::ProjectConfigLoad,
}

fn report_card(app_handle: &tauri::AppHandle, title: &str, body: String, level: &str) {
    use tauri::Emitter;
    let _ = app_handle.emit(
        "mcp://report",
        serde_json::json!({
            "title": title,
            "body": body,
            "level": level,
            "path": null,
        }),
    );
}

fn start_project_watcher(state: &State<AppState>, app_handle: &tauri::AppHandle, path: &str) {
    if let Err(error) = file_watcher::start_watching(&state.file_watcher, app_handle, path) {
        report_card(
            app_handle,
            "文件监听不可用",
            format!("文件系统监听器启动失败,外部文件变更不会自动反映到 IDE。可通过文件树刷新按钮手动同步。\n\n错误:{error}"),
            "error",
        );
    }
}

fn register_project_mcp_clients(
    state: &State<AppState>,
    app_handle: &tauri::AppHandle,
    path: &str,
) {
    // 取出端口/token 后立即放锁,避免持锁做文件 IO(pty_create 等会争用这把锁)
    let endpoint = {
        let mcp = state.mcp_server.lock().unwrap_or_else(|e| e.into_inner());
        mcp.as_ref().map(|info| (info.port, info.token.clone()))
    };

    let Some((port, token)) = endpoint else {
        return;
    };

    // 项目根就绪后,把 MCP server 接入信息写进各客户端配置(非破坏性)。
    for card in common::mcp_config::register_clients(path, port, &token) {
        report_card(app_handle, &card.title, card.body, &card.level);
    }
}

/// 事务式打开项目。
///
/// 目录验证与首层扫描都成功之后，才提交项目根、切换 watcher、注册 MCP 客户端
/// 并记录最近项目；任何一步失败都不会动到当前工作区、监听器或最近项目列表。
/// 配置文件损坏**不**阻止打开——用户仍然要能看到项目内容再决定怎么修，
/// 因此以 invalid 状态返回，由前端在覆盖前二次确认。
#[tauri::command]
fn open_project(
    state: State<AppState>,
    path: String,
    app_handle: tauri::AppHandle,
) -> Result<ProjectOpenResult, String> {
    let root_path = fs_utils::validate_project_dir(&path)?;
    let files = fs_utils::get_file_tree(&root_path).map_err(|e| e.to_string())?;

    {
        let mut root = state
            .current_project_root
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *root = Some(root_path.clone());
    }

    start_project_watcher(&state, &app_handle, &root_path);
    register_project_mcp_clients(&state, &app_handle, &root_path);

    // 记录失败只是最近项目列表没更新，不该让整个打开动作失败，但要让用户知道
    let updated = recent_projects::record(
        &state.config_manager.get_config().recent_projects,
        &root_path,
        recent_projects::now_millis(),
    );
    if let Err(error) = state.config_manager.set_recent_projects(updated) {
        report_card(
            &app_handle,
            "最近项目未能保存",
            format!("项目已打开，但最近项目列表写入失败，重启后可能看不到这一条。\n\n错误:{error}"),
            "warning",
        );
    }

    let project_config = project_config::load_project_config_detailed(&root_path);

    Ok(ProjectOpenResult {
        root_path,
        files,
        project_config,
    })
}

// ----------------------------------------------------------------
// Recent Projects Commands
// ----------------------------------------------------------------

#[tauri::command]
fn list_recent_projects(state: State<AppState>) -> Vec<recent_projects::RecentProjectView> {
    recent_projects::to_views(&state.config_manager.recent_projects())
}

#[tauri::command]
fn remove_recent_project(
    state: State<AppState>,
    path: String,
) -> Result<Vec<recent_projects::RecentProjectView>, String> {
    let updated =
        recent_projects::remove(&state.config_manager.get_config().recent_projects, &path);
    state.config_manager.set_recent_projects(updated)?;
    Ok(recent_projects::to_views(
        &state.config_manager.recent_projects(),
    ))
}

#[tauri::command]
fn clear_recent_projects(state: State<AppState>) -> Result<(), String> {
    state.config_manager.set_recent_projects(Vec::new())
}

// ----------------------------------------------------------------
// Project Config Commands
// ----------------------------------------------------------------

/// 返回三态结果（absent / loaded / invalid），让前端能区分"还没配置"和
/// "配置文件坏了"——后者不确认就覆盖会毁掉用户手写的内容。
#[tauri::command]
fn load_project_config(state: State<AppState>) -> Result<project_config::ProjectConfigLoad, String> {
    let root = state.current_project_root.lock().unwrap_or_else(|e| e.into_inner());
    let root_path = root.as_deref().ok_or("No project root set")?;
    Ok(project_config::load_project_config_detailed(root_path))
}

/// `expected_root` 是 UI 打开对话框时所编辑的那个项目根。
///
/// 写入目标取自后端的当前项目根，两者必须一致：项目设置对话框是非模态于快捷键的
/// （Ctrl+O 的全局 keydown 不被 naive-ui 拦截），A 项目开着设置框时切到 B，
/// 表单里还是 A 的值。不校验的话这一保存会把 A 的配置写进 B 的
/// `.thtk-project.json`，而且因为 B 的配置是合法的，连损坏确认都不会弹。
#[tauri::command]
fn save_project_config_cmd(
    state: State<AppState>,
    config: project_config::ProjectConfig,
    expected_root: String,
) -> Result<(), String> {
    let root = state.current_project_root.lock().unwrap_or_else(|e| e.into_inner());
    let root_path = root.as_deref().ok_or("No project root set")?;

    if !recent_projects::paths_equal(root_path, &expected_root) {
        return Err(format!(
            "项目已切换到 {root_path}，未写入 {expected_root}。请重新打开项目设置。"
        ));
    }

    // 数据契约要求保存前校验项目根状态：目录若已被外部删除，
    // 否则失败会以"写入临时文件失败: No such file or directory"这种形式冒出来
    fs_utils::validate_project_dir(root_path)?;

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
            common::game_version_commands::list_game_versions,
            read_file,
            save_file,
            open_project,
            list_recent_projects,
            remove_recent_project,
            clear_recent_projects,
            load_project_config,
            save_project_config_cmd,
            get_file_tree,
            get_dir_children,
            compile_ecl_file,
            decompile_ecl_file,
            get_ecl_map_semantics,
            run_thecl_operation,
            decompile_msg_file,
            compile_msg_file,
            decompile_std_file,
            compile_std_file,
            extract_dat_file,
            pack_dat_file,
            generate_ai_assist_pack,
            fs_ops::create_directory,
            fs_ops::create_file,
            fs_ops::rename_entry,
            fs_ops::copy_entry,
            fs_ops::delete_entry,
            fs_ops::stat_entry,
            system_clipboard::set_file_clipboard,
            system_clipboard::get_file_clipboard,
            common::pty::pty_create,
            common::pty::pty_write,
            common::pty::pty_resize,
            common::pty::pty_kill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
