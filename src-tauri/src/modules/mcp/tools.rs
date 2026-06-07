use crate::common::toolchain;
use crate::config::AppConfig;
use crate::modules::ecl::{commands, compiler, map_parser};
use serde_json::{json, Value};
use std::path::Path;

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
        "defaultGameVersion": config.default_game_version,
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

fn resolved_maps(config: &AppConfig, version: &str) -> Vec<String> {
    commands::resolve_default_maps(config, config.thtk_dir.as_str(), version, Vec::new())
}

fn base_request(
    mode: compiler::TheclMode,
    config: &AppConfig,
    input_path: &str,
    output_path: Option<String>,
) -> compiler::TheclRequest {
    let version = config.default_game_version.clone();
    compiler::TheclRequest {
        mode,
        version: version.clone(),
        input_path: input_path.to_string(),
        output_path,
        map_paths: resolved_maps(config, &version),
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    }
}

/// check_ecl:编译到临时文件并删除产物,只为拿诊断
pub fn check_ecl(config: &AppConfig, source_path: &str) -> Result<Value, String> {
    let file_name = Path::new(source_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("Invalid source path: {source_path}"))?;
    let temp_output = std::env::temp_dir().join(format!("thtk-check-{file_name}.ecl"));

    let request = base_request(
        compiler::TheclMode::Compile,
        config,
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
    source_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    Ok(run_thecl(
        config,
        &base_request(compiler::TheclMode::Compile, config, source_path, output_path),
    ))
}

/// decompile_ecl
pub fn decompile_ecl(
    config: &AppConfig,
    binary_path: &str,
    output_path: Option<String>,
) -> Result<Value, String> {
    Ok(run_thecl(
        config,
        &base_request(
            compiler::TheclMode::Decompile,
            config,
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

/// 解析当前应使用的 eclmap 路径(配置优先,其次 thtk_dir/maps/{ver}.eclmap)
pub fn resolve_map_path(config: &AppConfig) -> Result<String, String> {
    let maps = resolved_maps(config, &config.default_game_version);
    maps.into_iter().next().ok_or_else(|| {
        "No eclmap configured: set eclmap_path or thtk_dir in settings".to_string()
    })
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
}
