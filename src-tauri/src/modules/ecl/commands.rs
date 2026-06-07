use super::compiler;
use super::map_parser;
use crate::app_state::AppState;
use crate::common::toolchain;
use std::path::Path;
use tauri::State;

fn ensure_thecl_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thecl", "thecl.exe");
    if resolved.trim().is_empty() {
        return Err("thecl path is not configured".to_string());
    }
    Ok(())
}

fn resolve_default_maps(
    config: &crate::config::AppConfig,
    thtk_dir: &str,
    game_ver: &str,
    mut maps: Vec<String>,
) -> Vec<String> {
    if maps.is_empty() {
        if !config.eclmap_path.trim().is_empty() {
            maps.push(config.eclmap_path.trim().to_string());
            return maps;
        }

        if !thtk_dir.trim().is_empty() {
            let default_map = Path::new(thtk_dir)
                .join("maps")
                .join(format!("{}.eclmap", game_ver));

            if default_map.exists() {
                maps.push(default_map.to_string_lossy().to_string());
            }
        }
    }

    maps
}

fn build_legacy_request(
    mode: compiler::TheclMode,
    version: String,
    input_path: String,
    map_paths: Vec<String>,
) -> compiler::TheclRequest {
    compiler::TheclRequest {
        mode,
        version,
        input_path,
        output_path: None,
        map_paths,
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    }
}

#[tauri::command]
pub async fn compile_ecl_file(
    state: State<'_, AppState>,
    source_path: String,
    map_paths: Vec<String>,
) -> Result<compiler::EclResult, String> {
    let config = state.config_manager.get_config();
    let thtk_dir = config.thtk_dir.clone();
    let game_ver = config.default_game_version.clone();

    ensure_thecl_configured(&config)?;

    let maps = resolve_default_maps(&config, &thtk_dir, &game_ver, map_paths);
    let request = build_legacy_request(compiler::TheclMode::Compile, game_ver, source_path, maps);

    Ok(compiler::run(&config, &request))
}

#[tauri::command]
pub async fn decompile_ecl_file(
    state: State<'_, AppState>,
    binary_path: String,
    map_paths: Vec<String>,
) -> Result<compiler::EclResult, String> {
    let config = state.config_manager.get_config();
    let thtk_dir = config.thtk_dir.clone();
    let game_ver = config.default_game_version.clone();

    ensure_thecl_configured(&config)?;

    let maps = resolve_default_maps(&config, &thtk_dir, &game_ver, map_paths);
    let request =
        build_legacy_request(compiler::TheclMode::Decompile, game_ver, binary_path, maps);

    Ok(compiler::run(&config, &request))
}

#[tauri::command]
pub async fn run_thecl_operation(
    state: State<'_, AppState>,
    mut request: compiler::TheclRequest,
) -> Result<compiler::EclResult, String> {
    let config = state.config_manager.get_config();
    let thtk_dir = config.thtk_dir.clone();

    ensure_thecl_configured(&config)?;

    if request.version.is_empty() {
        request.version = config.default_game_version.clone();
    }

    request.map_paths = resolve_default_maps(&config, &thtk_dir, &request.version, request.map_paths);

    Ok(compiler::run(&config, &request))
}

#[tauri::command]
pub async fn get_thecl_status(
    state: State<'_, AppState>,
) -> Result<toolchain::ToolchainStatus, String> {
    let config = state.config_manager.get_config();
    toolchain::get_toolchain_status(&config, "thecl")
}

#[tauri::command]
pub async fn get_ecl_map_semantics(path: String) -> Result<map_parser::EclMapSemanticData, String> {
    map_parser::parse_ecl_map_file(&path)
}
