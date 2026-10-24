use super::compiler;
use super::map_parser;
use crate::app_state::AppState;
use crate::common::dependencies;
use crate::common::toolchain;
use std::path::{Path, PathBuf};
use tauri::State;

fn ensure_thecl_configured(config: &crate::config::AppConfig) -> Result<(), String> {
    let resolved = toolchain::resolve_tool_path(config, "thecl", "thecl.exe");
    if resolved.trim().is_empty() {
        return Err(toolchain::not_configured_message("thecl"));
    }
    Ok(())
}

/// 在某个目录里按统一命名找该版本的 map：
/// `th{version}.eclm` / `{version}.eclm` / `{version}.eclmap`
/// （内置基线用 th06–th09、th095 的补零命名，见 `dependencies::map_file_names`）。
fn find_map_in_dir(dir: &Path, game_ver: &str) -> Option<PathBuf> {
    dependencies::map_file_names(game_ver)
        .into_iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// 解析本次 thecl 调用实际使用的 map。
///
/// 优先级：显式 `map_paths` > 全局 `eclmap_path` > IDE 托管 map
/// （`deps/maps/bundled/`）> `thtk_dir/maps/` 推导。
/// 显式列表原样返回——调用方已经明确要哪几份 map，不该被默认值顶掉。
pub(crate) fn resolve_default_maps(
    config: &crate::config::AppConfig,
    thtk_dir: &str,
    game_ver: &str,
    maps: Vec<String>,
) -> Vec<String> {
    let managed_map = dependencies::DependencyManager::for_current_user()
        .ok()
        .and_then(|manager| manager.managed_map_path(game_ver));

    resolve_default_maps_with(config, thtk_dir, game_ver, maps, managed_map.as_deref())
}

/// [`resolve_default_maps`] 的可测试内核：托管 map 由调用方解析好再传入，
/// 单测因此不需要碰真实用户目录。
pub(crate) fn resolve_default_maps_with(
    config: &crate::config::AppConfig,
    thtk_dir: &str,
    game_ver: &str,
    mut maps: Vec<String>,
    managed_map: Option<&Path>,
) -> Vec<String> {
    if !maps.is_empty() {
        return maps;
    }

    if !config.eclmap_path.trim().is_empty() {
        maps.push(config.eclmap_path.trim().to_string());
        return maps;
    }

    if let Some(path) = managed_map {
        maps.push(path.to_string_lossy().to_string());
        return maps;
    }

    if !thtk_dir.trim().is_empty() {
        if let Some(path) = find_map_in_dir(&Path::new(thtk_dir).join("maps"), game_ver) {
            maps.push(path.to_string_lossy().to_string());
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
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
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
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
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
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let config = ctx.config;
    let thtk_dir = config.thtk_dir.clone();

    ensure_thecl_configured(&config)?;

    if request.version.is_empty() {
        request.version = config.default_game_version.clone();
    }

    request.map_paths = resolve_default_maps(&config, &thtk_dir, &request.version, request.map_paths);

    Ok(compiler::run(&config, &request))
}

#[tauri::command]
pub async fn get_ecl_map_semantics(path: String) -> Result<map_parser::EclMapSemanticData, String> {
    map_parser::parse_ecl_map_file(&path)
}

#[tauri::command]
pub async fn generate_ai_assist_pack(
    state: State<'_, AppState>,
    force: bool,
) -> Result<super::ai_pack::AiPackResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("No project root set")?;
    // 必须过项目级覆盖：否则辅助包的 references 会按全局 thtk_dir 的 eclmap 生成，
    // 而 check_ecl / compile_ecl 用的是项目级的，agent 拿到的语义和实际编译不是一回事
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), Some(&root));
    let config = ctx.config;

    let map_path = crate::modules::mcp::tools::resolve_map_path(&config, Some(&root))?;
    let semantics = map_parser::parse_ecl_map_file(&map_path)?;
    super::ai_pack::generate(&root, &semantics, force)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use std::fs;
    use tempfile::TempDir;

    fn config_with_eclmap(path: &str) -> AppConfig {
        AppConfig {
            eclmap_path: path.to_string(),
            ..AppConfig::default()
        }
    }

    fn dependency_manager(base: &TempDir) -> dependencies::DependencyManager {
        dependencies::DependencyManager::with_base_dir(base.path())
    }

    fn write_map(dir: &Path, name: &str) -> PathBuf {
        fs::create_dir_all(dir).expect("mkdir maps");
        let path = dir.join(name);
        fs::write(&path, "!eclmap\n").expect("write map");
        path
    }

    fn write_bundled_map(manager: &dependencies::DependencyManager, name: &str) -> PathBuf {
        write_map(&manager.bundled_maps_dir(), name)
    }

    #[test]
    fn explicit_map_paths_win_over_every_default() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        write_bundled_map(&manager, "th18.eclm");

        let maps = resolve_default_maps_with(
            &config_with_eclmap("/global/20.eclmap"),
            "/thtk",
            "18",
            vec!["/project/custom.eclm".to_string()],
            manager.managed_map_path("18").as_deref(),
        );

        assert_eq!(maps, vec!["/project/custom.eclm"]);
    }

    #[test]
    fn global_eclmap_wins_over_managed_maps() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        write_bundled_map(&manager, "th18.eclm");

        let maps = resolve_default_maps_with(
            &config_with_eclmap(" /global/18.eclmap "),
            "/thtk",
            "18",
            Vec::new(),
            manager.managed_map_path("18").as_deref(),
        );

        // 与旧行为一致：trim 之后原样作为唯一 map
        assert_eq!(maps, vec!["/global/18.eclmap"]);
    }

    #[test]
    fn managed_map_wins_over_thtk_dir_derivation() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        let managed_map = write_bundled_map(&manager, "th18.eclm");

        let thtk = TempDir::new().expect("thtk");
        write_map(&thtk.path().join("maps"), "18.eclmap");

        let maps = resolve_default_maps_with(
            &AppConfig::default(),
            &thtk.path().to_string_lossy(),
            "18",
            Vec::new(),
            manager.managed_map_path("18").as_deref(),
        );

        assert_eq!(maps, vec![managed_map.to_string_lossy().to_string()]);
    }

    #[test]
    fn managed_lookup_accepts_zero_padded_baseline_names() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        let th06 = write_bundled_map(&manager, "th06.eclm");
        let th095 = write_bundled_map(&manager, "th095.eclm");

        // 配置里的版本是纯数字 6 / 95，内置文件名是 th06 / th095
        let maps = resolve_default_maps_with(
            &AppConfig::default(),
            "",
            "6",
            Vec::new(),
            manager.managed_map_path("6").as_deref(),
        );
        assert_eq!(maps, vec![th06.to_string_lossy().to_string()]);

        let maps = resolve_default_maps_with(
            &AppConfig::default(),
            "",
            "th95",
            Vec::new(),
            manager.managed_map_path("th95").as_deref(),
        );
        assert_eq!(maps, vec![th095.to_string_lossy().to_string()]);
    }

    #[test]
    fn thtk_dir_derivation_supports_eclm_and_eclmap_names() {
        for (version, file_name) in [("17", "th17.eclm"), ("16", "16.eclm"), ("15", "15.eclmap")] {
            let thtk = TempDir::new().expect("thtk");
            let expected = write_map(&thtk.path().join("maps"), file_name);

            let maps = resolve_default_maps_with(
                &AppConfig::default(),
                &thtk.path().to_string_lossy(),
                version,
                Vec::new(),
                None,
            );

            assert_eq!(
                maps,
                vec![expected.to_string_lossy().to_string()],
                "{file_name} 应被解析"
            );
        }
    }

    #[test]
    fn falls_back_to_thtk_dir_when_managed_lacks_the_version() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        write_bundled_map(&manager, "th18.eclm");

        let thtk = TempDir::new().expect("thtk");
        let fallback = write_map(&thtk.path().join("maps"), "13.eclmap");

        let maps = resolve_default_maps_with(
            &AppConfig::default(),
            &thtk.path().to_string_lossy(),
            "13",
            Vec::new(),
            manager.managed_map_path("13").as_deref(),
        );

        assert_eq!(maps, vec![fallback.to_string_lossy().to_string()]);
    }

    #[test]
    fn unknown_or_empty_version_yields_no_default_map() {
        let base = TempDir::new().expect("managed");
        let manager = dependency_manager(&base);
        write_bundled_map(&manager, "th18.eclm");
        let thtk = TempDir::new().expect("thtk");
        write_map(&thtk.path().join("maps"), "18.eclmap");

        for version in ["", "bogus", "18.5"] {
            let maps = resolve_default_maps_with(
                &AppConfig::default(),
                &thtk.path().to_string_lossy(),
                version,
                Vec::new(),
                manager.managed_map_path(version).as_deref(),
            );
            assert!(maps.is_empty(), "{version:?} 不该拼出默认 map");
        }
    }
}
