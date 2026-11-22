use crate::common::{game_version, project_config, toolchain};
use crate::config::AppConfig;
use crate::modules::ecl::{commands, compiler, instruction_reference, map_parser};
use serde_json::{json, Value};
use std::path::Path;

/// 与前端 useTheclActions.applyProjectDefaults 同语义:
/// .thtk-project.json 的 gameVersion / mapPaths 覆盖应用级默认,保证 agent 与问题面板诊断同源。
pub struct EffectiveToolchainConfig {
    pub version: String,
    pub map_paths: Vec<String>,
}

pub fn effective_toolchain_config(
    config: &AppConfig,
    project_root: Option<&str>,
) -> EffectiveToolchainConfig {
    let mut version = config.default_game_version.clone();
    let mut map_paths: Vec<String> = Vec::new();

    if let Some(root) = project_root {
        if let Some(pc) = project_config::load_project_config(root) {
            // 前端: request.version || pc.gameVersion || ''(非空项目值覆盖默认)
            if !pc.game_version.is_empty() {
                version = pc.game_version.clone();
            }
            // 前端: request.mapPaths?.length ? request.mapPaths : (pc.mapPaths || [])
            // 项目内相对路径按项目根解析为绝对路径
            if !pc.map_paths.is_empty() {
                map_paths = project_config::resolve_map_paths(root, &pc.map_paths);
            }
        }
    }

    // 覆盖之后再兜底:resolve_default_maps 对非空 maps 原样返回
    let map_paths =
        commands::resolve_default_maps(config, config.thtk_dir.as_str(), &version, map_paths);

    EffectiveToolchainConfig { version, map_paths }
}

/// get_workspace_info:项目根 + 五个工具链状态
pub fn workspace_info(config: &AppConfig, project_root: Option<&str>) -> Value {
    let toolchains: Vec<Value> = toolchain::get_all_toolchain_statuses(config)
        .into_iter()
        .map(|status| {
            json!({
                "tool": status.tool,
                "available": status.available,
                "version": status.version,
                "resolvedPath": status.resolved_path,
                "message": status.message,
            })
        })
        .collect();

    json!({
        "projectRoot": project_root,
        "defaultGameVersion": effective_toolchain_config(config, project_root).version,
        "toolchains": toolchains,
    })
}

fn run_thecl(config: &AppConfig, request: &compiler::TheclRequest) -> Value {
    let result = compiler::run(config, request);
    json!({
        "success": result.success,
        "mode": result.mode,
        "inputPath": result.input_path,
        "outputPath": result.output_path,
        "diagnostics": result.diagnostics,
        "message": result.message,
    })
}

fn base_request(
    mode: compiler::TheclMode,
    config: &AppConfig,
    project_root: Option<&str>,
    input_path: &str,
    output_path: Option<String>,
) -> compiler::TheclRequest {
    let effective = effective_toolchain_config(config, project_root);
    compiler::TheclRequest {
        mode,
        version: effective.version,
        input_path: input_path.to_string(),
        output_path,
        map_paths: effective.map_paths,
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    }
}

/// check_ecl:编译到临时文件并删除产物,只为拿诊断
pub fn check_ecl(
    config: &AppConfig,
    project_root: Option<&str>,
    source_path: &str,
) -> Result<Value, String> {
    let file_name = Path::new(source_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("Invalid source path: {source_path}"))?;
    // 每次调用唯一的临时路径:MCP server 进程内并发,多个 agent 同时 check
    // 同名文件时不能共享路径(process::id 不够,需调用级计数器)。
    static CHECK_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = CHECK_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_output = std::env::temp_dir().join(format!(
        "thtk-check-{}-{seq}-{file_name}.ecl",
        std::process::id()
    ));

    let request = base_request(
        compiler::TheclMode::Compile,
        config,
        project_root,
        source_path,
        Some(temp_output.to_string_lossy().to_string()),
    );

    let mut value = run_thecl(config, &request);
    let _ = std::fs::remove_file(&temp_output);
    // 检查模式不暴露产物路径
    if let Some(object) = value.as_object_mut() {
        object.insert("outputPath".to_string(), Value::Null);
        object.insert("checkOnly".to_string(), json!(true));
    }
    Ok(value)
}

/// compile_ecl:真编译,产物落盘
pub fn compile_ecl(
    config: &AppConfig,
    project_root: Option<&str>,
    source_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    Ok(run_thecl(
        config,
        &base_request(
            compiler::TheclMode::Compile,
            config,
            project_root,
            source_path,
            output_path,
        ),
    ))
}

/// decompile_ecl
pub fn decompile_ecl(
    config: &AppConfig,
    project_root: Option<&str>,
    binary_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    Ok(run_thecl(
        config,
        &base_request(
            compiler::TheclMode::Decompile,
            config,
            project_root,
            binary_path,
            output_path,
        ),
    ))
}

pub fn lookup_ecl_semantics(
    config: &AppConfig,
    project_root: Option<&str>,
    query: &str,
    game_version_override: Option<&str>,
) -> Result<Value, String> {
    let effective = effective_toolchain_config(config, project_root);
    let current_game = game_version::parse(&effective.version).ok();
    let game = game_version_override
        .map(game_version::parse)
        .transpose()?
        .or(current_game);
    let map_paths = match game {
        Some(game) if Some(game) != current_game => {
            // 自定义 map 属于当前配置，跨作品只查按版本选出的 map。
            let mut target_config = config.clone();
            target_config.eclmap_path.clear();
            commands::resolve_default_maps(
                &target_config,
                &target_config.thtk_dir,
                &game.to_string(),
                Vec::new(),
            )
        }
        _ => effective.map_paths,
    };
    let references = match game {
        Some(game) => instruction_reference::for_game(game)?,
        None => None,
    };
    let map_result = lookup_semantics(&map_paths, query);
    let mut result = match (map_result, references) {
        (Ok(result), _) => result,
        (Err(error), Some(_)) => json!({
            "mapPath": map_paths.first(),
            "version": game.map(|id| id.to_string()).unwrap_or(effective.version),
            "query": query,
            "instructions": [],
            "globals": [],
            "mapError": error,
        }),
        (Err(error), None) => return Err(error),
    };
    let query_lower = query.trim().to_lowercase();
    let opcode_query = query_lower.parse::<u32>().ok();
    let map_opcodes: std::collections::HashSet<_> = result["instructions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|instruction| instruction["opcode"].as_u64())
        .collect();
    let matches: Vec<_> = references
        .unwrap_or_default()
        .iter()
        .filter(|entry| {
            entry.matches(&query_lower, opcode_query, game)
                || map_opcodes.contains(&u64::from(entry.opcode))
        })
        .take(50)
        .collect();
    result["gameVersion"] = json!(game);
    result["mapPaths"] = json!(map_paths);
    result["instructionReferences"] = json!(matches);
    Ok(result)
}

/// 在实际 map 中按名称子串或精确编号查询指令与全局寄存器。
fn lookup_semantics(map_paths: &[String], query: &str) -> Result<Value, String> {
    let (map_path, overlays) = map_paths
        .split_first()
        .ok_or_else(|| NO_ECL_MAP.to_string())?;
    let data = map_parser::parse_ecl_map_files(map_path, overlays)?;
    let query_lower = query.trim().to_lowercase();
    let opcode_query = query_lower.parse::<i64>().ok();

    let instructions: Vec<Value> = data
        .instructions
        .iter()
        .filter(|ins| {
            ins.name.to_lowercase().contains(&query_lower)
                || opcode_query == Some(ins.opcode as i64)
        })
        .take(50)
        .map(|ins| {
            json!({
                "opcode": ins.opcode,
                "name": ins.name,
                "section": ins.section,
                "signature": ins.signature,
                "params": ins.params,
            })
        })
        .collect();

    let globals: Vec<Value> = data
        .globals
        .iter()
        .filter(|g| {
            g.name.to_lowercase().contains(&query_lower) || opcode_query == Some(g.id as i64)
        })
        .take(50)
        .map(|g| json!({ "id": g.id, "name": g.name, "type": g.var_type }))
        .collect();

    Ok(json!({
        "mapPath": map_path,
        "version": data.version,
        "query": query,
        "instructions": instructions,
        "globals": globals,
    }))
}

const NO_ECL_MAP: &str = "No eclmap configured: set eclmap_path or thtk_dir in settings";

/// 解析当前应使用的 eclmap 路径(项目配置优先,其次应用配置,最后 thtk_dir/maps/{ver}.eclmap)
pub fn resolve_map_path(config: &AppConfig, project_root: Option<&str>) -> Result<String, String> {
    let effective = effective_toolchain_config(config, project_root);
    effective
        .map_paths
        .into_iter()
        .next()
        .ok_or_else(|| NO_ECL_MAP.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SAMPLE_MAP: &str = "!eclmap\n!ins_names\n10 jump\n23 wait\n!ins_signatures\n10 ot\n23 S\n!gvar_names\n-9985 I0\n!gvar_types\n-9985 $\n";

    fn write_sample_map(dir: &tempfile::TempDir) -> String {
        let path = dir.path().join("th17.eclm");
        let mut file = std::fs::File::create(&path).expect("create map");
        file.write_all(SAMPLE_MAP.as_bytes()).expect("write map");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn lookup_by_name_substring() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);

        let result = lookup_semantics(&[map_path], "wai").expect("lookup");

        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0]["name"], "wait");
        assert_eq!(matches[0]["opcode"], 23);
    }

    #[test]
    fn lookup_by_opcode_and_register() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_paths = [write_sample_map(&dir)];

        let result = lookup_semantics(&map_paths, "10").expect("lookup");
        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches[0]["name"], "jump");

        let result = lookup_semantics(&map_paths, "I0").expect("lookup");
        assert_eq!(result["globals"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn lookup_keeps_map_fields_and_uses_the_projects_reference() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy();
        let map_path = dir.path().join("custom.eclm");
        std::fs::write(&map_path, SAMPLE_MAP).expect("write map");
        let project = project_config::ProjectConfig {
            game_version: "th13".to_owned(),
            map_paths: vec![map_path.to_string_lossy().into_owned()],
            ..Default::default()
        };
        project_config::save_project_config(&root, &project).expect("save project config");

        let config = AppConfig::default();
        let result = lookup_ecl_semantics(&config, Some(&root), "10", None).expect("lookup");
        assert_eq!(result["gameVersion"], 13);
        assert_eq!(result["instructions"][0]["name"], "jump");
        assert_eq!(result["instructions"][0]["signature"], "ot");
        assert_eq!(result["instructionReferences"][0]["zero318Name"], "ret");
        assert_eq!(result["instructionReferences"][0]["signature"], "");

        let custom_name = lookup_ecl_semantics(&config, Some(&root), "jump", None).expect("lookup");
        assert!(custom_name["instructionReferences"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["opcode"] == 10 && entry["zero318Name"] == "ret"));

        let globals = lookup_ecl_semantics(&config, Some(&root), "I0", None).expect("lookup");
        assert_eq!(globals["globals"][0]["id"], -9985);
    }

    #[test]
    fn lookup_cross_game_uses_target_reference_without_the_current_custom_map() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);
        let config = AppConfig {
            default_game_version: "17".to_owned(),
            eclmap_path: map_path.clone(),
            ..AppConfig::default()
        };
        for query in ["12", "JuMp"] {
            let result = lookup_ecl_semantics(&config, None, query, Some("TH13")).expect("lookup");
            assert_ne!(result["mapPath"], map_path);
            assert_eq!(result["gameVersion"], 13);
            assert!(result["instructionReferences"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["opcode"] == 12 && entry["zero318Name"] == "jump"));
        }
        let alias = lookup_ecl_semantics(&config, None, "stackFREE", Some("TH13")).expect("lookup");
        assert!(alias["instructionReferences"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["opcode"] == 41 && entry["priw8Name"] == "stackFree"));
    }

    #[test]
    fn lookup_combines_signatures_and_names_from_project_map_layers() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy();
        let common = dir.path().join("common.eclm");
        let names = dir.path().join("names.eclm");
        std::fs::write(
            &common,
            "!eclmap\n!ins_signatures\n10 S\n12 ot\n!ins_names\n10 projectJmp\n!gvar_types\n-9985 $",
        )
        .unwrap();
        std::fs::write(
            &names,
            "!eclmap\n!ins_names\n12 projectJmp\n!gvar_names\n-9985 I0\n",
        )
        .unwrap();
        let project = project_config::ProjectConfig {
            game_version: "13".to_owned(),
            map_paths: vec!["common.eclm".to_owned(), "names.eclm".to_owned()],
            ..Default::default()
        };
        project_config::save_project_config(&root, &project).expect("save project config");

        let config = AppConfig::default();
        let result =
            lookup_ecl_semantics(&config, Some(&root), "projectJmp", None).expect("lookup");
        assert_eq!(result["mapPath"], common.to_string_lossy().as_ref());
        assert_eq!(result["mapPaths"].as_array().unwrap().len(), 2);
        assert_eq!(result["instructions"].as_array().unwrap().len(), 1);
        assert_eq!(result["instructions"][0]["opcode"], 12);
        assert_eq!(result["instructions"][0]["signature"], "ot");
        assert_eq!(result["instructionReferences"][0]["zero318Name"], "jump");

        let globals = lookup_ecl_semantics(&config, Some(&root), "I0", None).expect("lookup");
        assert_eq!(globals["globals"][0]["type"], "int");

        std::fs::remove_file(&names).unwrap();
        let result = lookup_ecl_semantics(&config, Some(&root), "12", None).expect("lookup");
        assert!(result["mapError"].as_str().unwrap().contains("names.eclm"));
        assert_eq!(result["instructions"], json!([]));
        assert_eq!(result["instructionReferences"][0]["opcode"], 12);
    }

    #[test]
    fn lookup_keeps_reference_available_without_a_readable_map() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = AppConfig {
            default_game_version: "20".to_owned(),
            eclmap_path: dir
                .path()
                .join("missing.eclm")
                .to_string_lossy()
                .into_owned(),
            ..AppConfig::default()
        };
        let result = lookup_ecl_semantics(&config, None, "12", None).expect("lookup");
        assert_eq!(result["gameVersion"], 20);
        assert_eq!(result["instructions"], json!([]));
        assert_eq!(result["globals"], json!([]));
        assert!(result["mapError"].is_string());
        assert_eq!(result["instructionReferences"][0]["opcode"], 12);
        assert_eq!(
            result["instructionReferences"][0]["stockCount"],
            Value::Null
        );

        let by_original_name =
            lookup_ecl_semantics(&config, None, "__debug_unknown_B", None).expect("lookup");
        let entry = &by_original_name["instructionReferences"][0];
        assert_eq!(entry["opcode"], 31);
        assert_eq!(entry["zero318SourceGame"], 20);
        assert!(entry["zero318Name"].is_null());
        assert!(entry["signature"].is_null());
    }

    #[test]
    fn workspace_info_reports_root_and_tools() {
        let config = AppConfig::default();
        let info = workspace_info(&config, Some("/tmp/proj"));
        assert_eq!(info["projectRoot"], "/tmp/proj");
        assert_eq!(info["toolchains"].as_array().expect("array").len(), 5);
    }

    #[test]
    fn effective_config_applies_project_overrides() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        // 项目内相对路径的 eclmap
        let maps_dir = dir.path().join("maps");
        std::fs::create_dir_all(&maps_dir).expect("mkdir maps");
        let map_file = maps_dir.join("custom.eclm");
        std::fs::write(&map_file, SAMPLE_MAP).expect("write map");

        let project = crate::common::project_config::ProjectConfig {
            game_version: "17".to_string(),
            map_paths: vec!["maps/custom.eclm".to_string()],
            ..Default::default()
        };
        crate::common::project_config::save_project_config(&root, &project)
            .expect("save project config");

        let config = AppConfig::default();
        let effective = effective_toolchain_config(&config, Some(&root));

        assert_eq!(effective.version, "17");
        assert_eq!(
            effective.map_paths,
            vec![map_file.to_string_lossy().to_string()]
        );
    }

    #[test]
    fn effective_config_falls_back_to_app_config_without_project_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        let config = AppConfig::default();
        let effective = effective_toolchain_config(&config, Some(&root));

        assert_eq!(effective.version, config.default_game_version);
        // 默认配置下无 eclmap_path / thtk_dir → 没有 map
        assert!(effective.map_paths.is_empty());
    }
}

pub fn run_script(
    config: &AppConfig,
    root: Option<&str>,
    kind: crate::modules::truth::maps::TruthScriptKind,
    operation: &str,
    input: String,
    output: Option<String>,
    encoding: Option<String>,
) -> Result<Value, String> {
    use crate::modules::truth::compiler::{self, ScriptMode, ScriptRequest};
    let ctx = toolchain::effective_context(config, root);
    let check = operation == "check";
    let mode = match operation {
        "compile" | "check" => ScriptMode::Compile,
        "decompile" => ScriptMode::Decompile,
        _ => return Err("Unknown script operation".into()),
    };
    let temp = std::env::temp_dir().join(format!("thtk-truth-check-{}", uuid::Uuid::new_v4()));
    let mut result = compiler::run(
        &ctx.config,
        root,
        ctx.project.as_ref(),
        &ScriptRequest {
            mode,
            kind,
            input_path: input,
            output_path: if check {
                Some(temp.to_string_lossy().into_owned())
            } else {
                output
            },
            encoding,
        },
    );
    if check {
        if temp.exists() {
            std::fs::remove_file(&temp).map_err(|e| e.to_string())?;
        }
        result.output_path = None;
    }
    let mut value = serde_json::to_value(result).map_err(|e| e.to_string())?;
    if check {
        value["checkOnly"] = serde_json::json!(true);
    }
    Ok(value)
}

pub fn lookup_script(
    config: &AppConfig,
    root: Option<&str>,
    kind: crate::modules::truth::maps::TruthScriptKind,
    path: &str,
    query: &str,
) -> Result<Value, String> {
    let ctx = toolchain::effective_context(config, root);
    let game =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &ctx.config, kind.tool())?;
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut data = crate::modules::truth::commands::source_definitions(
        game,
        kind,
        root,
        ctx.project.as_ref(),
        path,
        &text,
    )?;
    let q = query.trim().to_lowercase();
    data.instructions.retain(|i| {
        q.is_empty()
            || i.opcode.to_string() == q
            || i.aliases.iter().any(|s| s.to_lowercase().contains(&q))
    });
    data.instructions.truncate(50);
    serde_json::to_value(data).map_err(|e| e.to_string())
}
