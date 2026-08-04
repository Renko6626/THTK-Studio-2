//! 把跨前后端边界的结构体**序列化之后的字段名**钉死。
//!
//! 前端 `src/types/` 下的 TypeScript 类型是逐字段照着这些结构体写的，但两边没有
//! 任何自动同步机制。而本项目的序列化命名并不统一——`AppConfig` 是 snake_case、
//! `ProjectConfig` 是 camelCase、`FileNode` 混着来（`is_dir` 但 `isLeaf`）、
//! `EclMapInstructionParameter.type_name` 被单独 rename 成 `type`。这种不一致
//! 靠人记必然出错。
//!
//! 这些测试只断言 JSON 的键集合，不管值。改动任一结构体的字段名或 serde 属性时
//! 它们会失败，提醒同步修改 `src/types/` 里对应的类型。

use serde::Serialize;

/// 取序列化后的顶层键，排序以便稳定比较
fn json_keys<T: Serialize>(value: &T) -> Vec<String> {
    let json = serde_json::to_value(value).expect("结构体必须能序列化");
    let object = json.as_object().expect("期望序列化成 JSON 对象");
    let mut keys: Vec<String> = object.keys().cloned().collect();
    keys.sort();
    keys
}

fn sorted(items: &[&str]) -> Vec<String> {
    let mut keys: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    keys.sort();
    keys
}

#[test]
fn file_node_keeps_its_mixed_naming() {
    use crate::common::fs_utils::{FileCategory, FileNode};

    let node = FileNode {
        name: "st01.decl".to_string(),
        path: "/proj/st01.decl".to_string(),
        is_dir: false,
        size: Some(12),
        extension: Some("decl".to_string()),
        category: FileCategory::SourceScript,
        children: None,
        is_leaf: true,
        lossy: false,
    };

    // is_dir 是 snake_case 而 isLeaf 是 camelCase —— 这个不一致是历史遗留，
    // 前端 types/fs.ts 如实反映了它。谁想统一，得两边一起改。
    assert_eq!(
        json_keys(&node),
        sorted(&[
            "name", "path", "is_dir", "size", "extension", "category", "children", "isLeaf",
            "lossy"
        ])
    );

    let category = serde_json::to_value(FileCategory::SourceScript).unwrap();
    assert_eq!(category, serde_json::json!("sourceScript"));
}

#[test]
fn app_config_stays_snake_case() {
    let config = crate::config::AppConfig::default();

    assert_eq!(
        json_keys(&config),
        sorted(&[
            "thtk_dir",
            "thecl_path",
            "eclmap_path",
            "tool_overrides",
            "default_game_version",
            "theme",
            "mcp_port",
            "recent_projects"
        ])
    );
}

#[test]
fn project_config_is_camel_case() {
    use crate::common::project_config::ProjectConfig;

    assert_eq!(
        json_keys(&ProjectConfig::default()),
        sorted(&["gameVersion", "encoding", "mapPaths", "toolchain"])
    );

    let nested = serde_json::to_value(ProjectConfig::default()).unwrap();
    let toolchain = nested.get("toolchain").unwrap().as_object().unwrap();
    assert_eq!(toolchain.keys().collect::<Vec<_>>(), vec!["thtkDir"]);
}

#[test]
fn project_config_load_exposes_the_three_states() {
    use crate::common::project_config::{ProjectConfigLoad, ProjectConfigStatus};

    let load = ProjectConfigLoad {
        status: ProjectConfigStatus::Absent,
        value: None,
        error: None,
        path: "/proj/.thtk-project.json".to_string(),
    };

    assert_eq!(json_keys(&load), sorted(&["status", "value", "error", "path"]));

    // 三个状态的字面量前端要按字符串匹配
    for (status, expected) in [
        (ProjectConfigStatus::Absent, "absent"),
        (ProjectConfigStatus::Loaded, "loaded"),
        (ProjectConfigStatus::Invalid, "invalid"),
    ] {
        assert_eq!(serde_json::to_value(status).unwrap(), serde_json::json!(expected));
    }
}

#[test]
fn recent_project_shapes_differ_between_storage_and_view() {
    use crate::common::recent_projects::{to_views, RecentProject};

    let stored = RecentProject {
        path: "/proj".to_string(),
        name: "proj".to_string(),
        last_opened_at: 1,
    };
    assert_eq!(json_keys(&stored), sorted(&["path", "name", "lastOpenedAt"]));

    // 展示形态多一个现算的 available，它不落盘
    let view = to_views(&[stored]).remove(0);
    assert_eq!(
        json_keys(&view),
        sorted(&["path", "name", "lastOpenedAt", "available"])
    );
}

#[test]
fn toolchain_status_is_camel_case() {
    use crate::common::toolchain::ToolchainStatus;

    let status = ToolchainStatus {
        tool: "thecl".to_string(),
        label: "Enemy Script Compiler".to_string(),
        exe_name: "thecl.exe".to_string(),
        configured_path: String::new(),
        resolved_path: String::new(),
        available: false,
        version: String::new(),
        message: String::new(),
        supported_versions: vec![6, 20],
    };

    assert_eq!(
        json_keys(&status),
        sorted(&[
            "tool",
            "label",
            "exeName",
            "configuredPath",
            "resolvedPath",
            "available",
            "version",
            "message",
            "supportedVersions"
        ])
    );
}

#[test]
fn diagnostic_column_is_nullable() {
    use crate::modules::ecl::error_parser::Diagnostic;

    let diagnostic = Diagnostic {
        path: "/proj/st01.decl".to_string(),
        line: 12,
        column: None,
        severity: "error".to_string(),
        message: "unexpected token".to_string(),
    };

    assert_eq!(
        json_keys(&diagnostic),
        sorted(&["path", "line", "column", "severity", "message"])
    );

    // thecl 不总是给列号，前端类型必须是 number | null
    let json = serde_json::to_value(&diagnostic).unwrap();
    assert!(json.get("column").unwrap().is_null());
}

#[test]
fn thecl_request_and_result_are_camel_case() {
    use crate::modules::ecl::compiler::{TheclMode, TheclRequest};

    let request = TheclRequest {
        mode: TheclMode::Compile,
        version: "18".to_string(),
        input_path: "/proj/st01.decl".to_string(),
        output_path: None,
        map_paths: vec![],
        use_shift_jis: true,
        raw_dump: false,
        simple_creation: false,
        show_offsets: false,
    };

    assert_eq!(
        json_keys(&request),
        sorted(&[
            "mode",
            "version",
            "inputPath",
            "outputPath",
            "mapPaths",
            "useShiftJis",
            "rawDump",
            "simpleCreation",
            "showOffsets"
        ])
    );

    assert_eq!(
        serde_json::to_value(TheclMode::Decompile).unwrap(),
        serde_json::json!("decompile")
    );

    use crate::modules::ecl::compiler::EclResult;
    let result = EclResult {
        success: true,
        tool: "thecl".to_string(),
        mode: "compile".to_string(),
        script_kind: "ecl".to_string(),
        input_path: "/proj/st01.decl".to_string(),
        message: String::new(),
        diagnostics: vec![],
        output_path: None,
    };

    assert_eq!(
        json_keys(&result),
        sorted(&[
            "success",
            "tool",
            "mode",
            "scriptKind",
            "inputPath",
            "message",
            "diagnostics",
            "outputPath"
        ])
    );
}

#[test]
fn eclmap_parameter_type_field_is_renamed() {
    use crate::modules::ecl::map_parser::EclMapInstructionParameter;

    let param = EclMapInstructionParameter {
        name: "difficulty".to_string(),
        type_name: "int".to_string(),
    };

    // Rust 侧字段叫 type_name，但 #[serde(rename = "type")] 把它变成了 type。
    // 写成 typeName 会在运行时静默拿到 undefined。
    assert_eq!(json_keys(&param), sorted(&["name", "type"]));
}

#[test]
fn game_version_view_keys_are_pinned() {
    use crate::common::game_version_commands::GameVersionView;

    let view = GameVersionView {
        id: 18,
        code: "th18".to_string(),
        title: "东方虹龙洞".to_string(),
        tools: vec!["thecl".to_string()],
    };

    assert_eq!(json_keys(&view), sorted(&["id", "code", "title", "tools"]));
}
