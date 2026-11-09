use script_check::*;
#[test]
fn snapshots_resolve_priority_and_order_without_rewriting_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(PROJECT_FILE),r#"{"gameVersion":"TH18","encoding":"gbk","scriptMaps":{"msg":["project.map"]},"toolchain":{"truthDir":"project-tools","thtkDir":"unrelated"},"mapPaths":[]}"#).unwrap();
    let project = load_project_snapshot(dir.path()).unwrap();
    let input = ContextInput {
        kind: ScriptKind::Msg,
        input_path: dir.path().join("new.msgspec"),
        project_root: Some(dir.path().into()),
        explicit_game_version: Some("th17".into()),
        explicit_encoding: Some("utf-8".into()),
        explicit_truth_dir: Some(dir.path().join("explicit-tools")),
        explicit_map_paths: vec![dir.path().join("call.map")],
        app: AppSnapshot {
            tool_overrides: [(
                "trumsg".into(),
                dir.path()
                    .join("custom-tool")
                    .to_string_lossy()
                    .into_owned(),
            )]
            .into(),
            ..Default::default()
        },
        project,
        bundled_truth_dir: dir.path().join("bundled"),
    };
    let ctx = resolve_context(&input).unwrap();
    assert_eq!(ctx.game_version, 17);
    assert_eq!(ctx.encoding.as_deref(), Some("utf-8"));
    assert_eq!(ctx.tool_path, dir.path().join("custom-tool"));
    assert_eq!(
        &ctx.map_paths[1..],
        &[dir.path().join("project.map"), dir.path().join("call.map")]
    );
    assert_eq!(
        discover_project_root(&input.input_path).unwrap().as_deref(),
        Some(dir.path())
    );
    std::fs::write(dir.path().join(PROJECT_FILE), "invalid").unwrap();
    assert!(load_project_snapshot(dir.path()).is_err());
}
