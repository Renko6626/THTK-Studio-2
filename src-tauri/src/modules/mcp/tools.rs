use crate::common::{project_config, toolchain};
use crate::config::AppConfig;
use crate::modules::ecl::{commands, compiler, map_parser};
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

    let script_profiles: Vec<Value> = [
        crate::modules::truth::maps::TruthScriptKind::Msg,
        crate::modules::truth::maps::TruthScriptKind::Ending,
        crate::modules::truth::maps::TruthScriptKind::Std,
    ].into_iter().map(|kind| {
        let path = Path::new(project_root.unwrap_or(".")).join(format!("untitled.{}", kind.extension()));
        match crate::modules::truth::compiler::resolve_script_context(
            config, project_root, None, kind, &path.to_string_lossy(), None,
        ) {
            Ok(context) => {
                let available = context.tool_path.is_file();
                json!({"kind": kind, "gameVersion": context.game_version, "encoding": context.encoding,
                    "mapPaths": context.map_paths, "toolPath": context.tool_path, "available": available,
                    "message": if available { None } else { Some("工具文件不存在") }})
            }
            Err(message) => json!({"kind": kind, "gameVersion": null, "encoding": null,
                "mapPaths": [], "toolPath": null, "available": false, "message": message}),
        }
    }).collect();
    let checker = toolchain::bundled_checker_path().filter(|p| p.is_file());
    json!({
        "scriptProfiles": script_profiles,
        "checkerPath": checker,
        "referenceRoot": reference_root(project_root),
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

/// lookup_ecl_semantics:按名称子串或精确 opcode 查指令,同时匹配全局寄存器
pub fn lookup_semantics(map_path: &str, query: &str) -> Result<Value, String> {
    let data = map_parser::parse_ecl_map_file(map_path)?;
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

/// 解析当前应使用的 eclmap 路径(项目配置优先,其次应用配置,最后 thtk_dir/maps/{ver}.eclmap)
pub fn resolve_map_path(config: &AppConfig, project_root: Option<&str>) -> Result<String, String> {
    let effective = effective_toolchain_config(config, project_root);
    effective
        .map_paths
        .into_iter()
        .next()
        .ok_or_else(|| "No eclmap configured: set eclmap_path or thtk_dir in settings".to_string())
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
    fn script_lookup_supports_new_files_pragmas_pagination_and_complete_enums() {
        use crate::modules::truth::maps::TruthScriptKind;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        project_config::save_project_config(
            root,
            &project_config::ProjectConfig {
                game_version: "18".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let path = dir.path().join("new.stdspec");
        let path = path.to_str().unwrap();
        std::fs::write(
            dir.path().join("custom.stdm"),
            "!stdmap\n!ins_names\n3 projectCamera\n",
        )
        .unwrap();
        let config = AppConfig::default();
        let buffer = "#pragma mapfile \"custom.stdm\"\n";
        let custom = lookup_script(
            &config,
            Some(root),
            TruthScriptKind::Std,
            path,
            "projectCamera",
            Some(buffer),
            None,
            None,
        )
        .unwrap();
        assert_eq!(custom["total"], 1);
        assert_eq!(custom["instructions"][0]["opcode"], 3);
        assert!(custom["sourceContext"].as_str().unwrap().contains("传入"));
        assert_eq!(custom["instructions"][0]["referencePath"], Value::Null);
        let full = lookup_script(
            &config,
            Some(root),
            TruthScriptKind::Std,
            path,
            "",
            None,
            None,
            Some(200),
        )
        .unwrap();
        assert!(full["sourceContext"].as_str().unwrap().contains("尚未创建"));
        let mut collected = Vec::new();
        let mut offset = Some(0);
        while let Some(start) = offset {
            let page = lookup_script(
                &config,
                Some(root),
                TruthScriptKind::Std,
                path,
                "",
                None,
                Some(start),
                Some(7),
            )
            .unwrap();
            assert_eq!(page["total"], full["total"]);
            assert_eq!(page["enums"], full["enums"]);
            collected.extend(page["instructions"].as_array().unwrap().iter().cloned());
            offset = page["nextOffset"].as_u64().map(|n| n as usize);
        }
        assert_eq!(collected, *full["instructions"].as_array().unwrap());
        let end = lookup_script(
            &config,
            Some(root),
            TruthScriptKind::Std,
            path,
            "",
            None,
            Some(usize::MAX),
            None,
        )
        .unwrap();
        assert!(end["instructions"].as_array().unwrap().is_empty());
        assert!(end["nextOffset"].is_null());
        assert!(lookup_script(
            &config,
            Some(root),
            TruthScriptKind::Std,
            path,
            "",
            None,
            None,
            Some(201)
        )
        .is_err());
    }

    #[test]
    fn script_profiles_keep_kinds_separate_and_do_not_hide_unsupported_ending() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        project_config::save_project_config(
            root,
            &project_config::ProjectConfig {
                game_version: "18".into(),
                map_paths: vec!["ecl-only.eclm".into()],
                script_maps: project_config::ScriptMaps {
                    msg: vec!["dialog.msgm".into()],
                    ending: vec!["credits.endm".into()],
                    std: vec!["stage.stdm".into()],
                },
                ..Default::default()
            },
        )
        .unwrap();
        let info = workspace_info(&AppConfig::default(), Some(root));
        assert!(info.get("defaultGameVersion").is_some());
        assert_eq!(info["toolchains"].as_array().unwrap().len(), 5);
        let profiles = info["scriptProfiles"].as_array().unwrap();
        for (index, file) in ["dialog.msgm", "credits.endm", "stage.stdm"]
            .iter()
            .enumerate()
        {
            let paths = profiles[index]["mapPaths"].as_array().unwrap();
            assert_eq!(paths.len(), 2);
            assert!(paths[1].as_str().unwrap().ends_with(file));
        }
        project_config::save_project_config(
            root,
            &project_config::ProjectConfig {
                game_version: "8".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let info = workspace_info(&AppConfig::default(), Some(root));
        assert_eq!(info["scriptProfiles"][0]["gameVersion"], 8);
        assert_eq!(info["scriptProfiles"][1]["available"], false);
        assert!(info["scriptProfiles"][1]["message"]
            .as_str()
            .unwrap()
            .contains("8"));
        assert_eq!(info["scriptProfiles"][2]["gameVersion"], 8);
        assert!(info["referenceRoot"].is_null());
    }

    #[test]
    fn lookup_by_name_substring() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);

        let result = lookup_semantics(&map_path, "wai").expect("lookup");

        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0]["name"], "wait");
        assert_eq!(matches[0]["opcode"], 23);
    }

    #[test]
    fn lookup_by_opcode_and_register() {
        let dir = tempfile::tempdir().expect("tempdir");
        let map_path = write_sample_map(&dir);

        let result = lookup_semantics(&map_path, "10").expect("lookup");
        let matches = result["instructions"].as_array().expect("array");
        assert_eq!(matches[0]["name"], "jump");

        let result = lookup_semantics(&map_path, "I0").expect("lookup");
        assert_eq!(result["globals"].as_array().unwrap().len(), 1);
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

pub fn reference_root(root: Option<&str>) -> Option<std::path::PathBuf> {
    root.map(|r| Path::new(r).join(".claude/skills/touhou-truth-scripts/references"))
        .filter(|p| p.is_dir())
}

pub fn run_script(
    config: &AppConfig,
    root: Option<&str>,
    kind: crate::modules::truth::maps::TruthScriptKind,
    operation: &str,
    input: String,
    output: Option<String>,
    encoding: Option<String>,
    source_text: Option<String>,
) -> Result<Value, String> {
    use crate::modules::truth::{
        check,
        compiler::{self, ScriptMode, ScriptRequest},
    };
    if source_text.is_some() && operation != "check" {
        return Err("source_text 仅支持 check 操作".into());
    }
    let result = if operation == "check" {
        let text = match source_text {
            Some(text) => text,
            None => std::fs::read_to_string(&input).map_err(|e| format!("{input}: {e}"))?,
        };
        check::check_with_encoding(config, root, None, &input, kind, &text, encoding.as_deref())?
    } else {
        let mode = match operation {
            "compile" => ScriptMode::Compile,
            "decompile" => ScriptMode::Decompile,
            _ => return Err("Unknown script operation".into()),
        };
        compiler::run(
            config,
            root,
            None,
            &ScriptRequest {
                mode,
                kind,
                input_path: input,
                output_path: output,
                encoding,
            },
        )
    };
    let mut value = serde_json::to_value(result).map_err(|e| e.to_string())?;
    if operation == "check" {
        value["checkOnly"] = json!(true);
    }
    Ok(value)
}

pub fn lookup_script(
    config: &AppConfig,
    root: Option<&str>,
    kind: crate::modules::truth::maps::TruthScriptKind,
    path: &str,
    query: &str,
    source_text: Option<&str>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Value, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50);
    if !(1..=200).contains(&limit) {
        return Err("limit 必须为 1–200".into());
    }
    let context = crate::modules::truth::compiler::resolve_script_context(
        config, root, None, kind, path, None,
    )?;
    let (text, source_context) = match source_text {
        Some(text) => (text.to_string(), "使用传入的未保存源码及其 pragma map"),
        None => match std::fs::read_to_string(path) {
            Ok(text) => (text, "使用磁盘源码及其 pragma map"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
                String::new(),
                "源码尚未创建，使用项目定义，未解析文件 pragma",
            ),
            Err(e) => return Err(format!("{path}: {e}")),
        },
    };
    let mut data = crate::modules::truth::commands::source_definitions_for_context(
        &context, kind, path, &text,
    )?;
    let q = query.trim().to_lowercase();
    data.instructions.retain(|i| {
        q.is_empty()
            || i.opcode.to_string() == q
            || i.aliases.iter().any(|s| s.to_lowercase().contains(&q))
    });
    data.instructions.sort_by_key(|i| i.opcode);
    let total = data.instructions.len();
    data.instructions = data
        .instructions
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    let next = offset.saturating_add(data.instructions.len());
    let mut value = serde_json::to_value(data).map_err(|e| e.to_string())?;
    value["total"] = json!(total);
    value["nextOffset"] = if next < total {
        json!(next)
    } else {
        Value::Null
    };
    value["sourceContext"] = json!(source_context);
    let reference = reference_root(root)
        .map(|p| p.join(format!("{}.md", kind.as_str())))
        .filter(|p| p.is_file());
    for instruction in value["instructions"].as_array_mut().unwrap() {
        instruction["referencePath"] = json!(reference);
    }
    Ok(value)
}
