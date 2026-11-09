use script_check::*;
use std::{fs, path::PathBuf};
fn truth_dir() -> PathBuf {
    std::env::var_os("THTK_TEST_TRUTH_DIR")
        .map(PathBuf::from)
        .expect("set THTK_TEST_TRUTH_DIR to prepared host truth tools")
}
fn context(root: &std::path::Path, kind: ScriptKind, project: ProjectSnapshot) -> ScriptContext {
    resolve_context(&ContextInput {
        kind,
        input_path: root.join(format!("input.{}", kind.extension())),
        project_root: Some(root.into()),
        explicit_game_version: None,
        explicit_encoding: None,
        explicit_truth_dir: Some(truth_dir()),
        explicit_map_paths: vec![],
        app: AppSnapshot::default(),
        project: Some(project),
        bundled_truth_dir: PathBuf::new(),
    })
    .unwrap()
}
#[test]
#[ignore = "requires prepared host truth tools and THTK_TEST_TRUTH_DIR"]
fn compile_decompile_maps_gbk_and_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    for (kind, opcode, signature, args, magic) in [
        (
            ScriptKind::Msg,
            17,
            "z(bs=4;mask=0x77,7,16)",
            "\"汉\"",
            "msg",
        ),
        (
            ScriptKind::Ending,
            3,
            "z(bs=4;mask=0x77,7,16)",
            "\"汉\"",
            "end",
        ),
        (ScriptKind::Std, 3, "SSfff", "60, 0, 1.0, 2.0, 3.0", "std"),
    ] {
        fs::write(dir.path().join("project.map"), format!("!{magic}map\n!ins_names\n{opcode} projectName\n!ins_signatures\n{opcode} {signature}\n")).unwrap();
        fs::write(
            dir.path().join("source.map"),
            format!("!{magic}map\n!ins_names\n{opcode} sourceName\n"),
        )
        .unwrap();
        let mut project = ProjectSnapshot {
            game_version: "18".into(),
            encoding: "gbk".into(),
            ..Default::default()
        };
        let paths = vec!["project.map".into()];
        match kind {
            ScriptKind::Msg => project.script_maps.msg = paths,
            ScriptKind::Ending => project.script_maps.ending = paths,
            ScriptKind::Std => project.script_maps.std = paths,
        }
        let ctx = context(dir.path(), kind, project);
        let meta = if kind == ScriptKind::Std {
            "meta { unknown: 0, anm_path: \"stage.anm\", objects: {}, instances: [] }"
        } else {
            "meta { table: {0: {script: \"main\"}} }"
        };
        let source = format!(
            "#pragma mapfile \"source.map\"\n{meta}\nscript main {{\n sourceName({args});\n}}\n"
        );
        let input = dir.path().join(format!("input.{}", kind.extension()));
        fs::write(&input, &source).unwrap();
        let request = ScriptRequest {
            operation: ScriptOperation::Compile,
            input_path: input.clone(),
            output_path: Some("built.bin".into()),
            source_text: None,
        };
        let result = run(&ctx, &request).unwrap();
        assert!(result.success, "{}", result.message);
        if kind != ScriptKind::Std {
            assert!(fs::read(dir.path().join("built.bin"))
                .unwrap()
                .windows(4)
                .any(|w| w == [0xcd, 0xc4, 0x95, 0xbc]));
        }
        let decoded = run(
            &ctx,
            &ScriptRequest {
                operation: ScriptOperation::Decompile,
                input_path: result.output_path.unwrap().into(),
                output_path: Some(format!("decoded.{}", kind.extension()).into()),
                source_text: None,
            },
        )
        .unwrap();
        assert!(decoded.success, "{}", decoded.message);
        let text = fs::read_to_string(decoded.output_path.unwrap()).unwrap();
        assert!(text.contains("projectName("));
        if kind != ScriptKind::Std {
            assert!(text.contains("汉"));
        }
        fs::write(&input, source.replace(args, "")).unwrap();
        let failed = run(&ctx, &request).unwrap();
        assert!(!failed.success);
        assert!(
            failed
                .diagnostics
                .iter()
                .any(|d| d.line == 4 && d.severity == "error"),
            "{}",
            failed.message
        );
        assert!(run(
            &ctx,
            &ScriptRequest {
                output_path: Some(input.clone()),
                ..request
            }
        )
        .is_err());
    }
}
#[test]
#[ignore = "requires prepared host truth tools and THTK_TEST_TRUTH_DIR"]
fn buffers_preserve_file_relative_pragma_and_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dialog.msgspec");
    fs::write(&path, "original unsaved file").unwrap();
    fs::write(
        dir.path().join("custom.msgm"),
        "!msgmap\n!ins_names\n17 say\n",
    )
    .unwrap();
    let ctx = context(
        dir.path(),
        ScriptKind::Msg,
        ProjectSnapshot {
            game_version: "18".into(),
            encoding: "gbk".into(),
            ..Default::default()
        },
    );
    let text="#pragma mapfile \"custom.msgm\"\nmeta {table:{0:{script:\"main\"}}}\nscript main {\n say(\"汉\");\n}";
    let request = |input: PathBuf, text: &str| ScriptRequest {
        operation: ScriptOperation::Check,
        input_path: input,
        output_path: None,
        source_text: Some(text.into()),
    };
    let good = run(&ctx, &request(path.clone(), text)).unwrap();
    assert!(good.success, "{}", good.message);
    assert!(good.output_path.is_none());
    let bad = run(
        &ctx,
        &request(path.clone(), &text.replace("say(\"汉\")", "say()")),
    )
    .unwrap();
    assert!(!bad.success);
    assert_eq!(bad.diagnostics[0].path, path.to_str().unwrap());
    assert_eq!(bad.diagnostics[0].line, 4);
    assert_eq!(fs::read_to_string(&path).unwrap(), "original unsaved file");
    let new = dir.path().join("new.msgspec");
    assert!(run(&ctx, &request(new.clone(), text)).unwrap().success);
    assert!(!new.exists());
    // Some("") is a real empty buffer, not a request to read the invalid disk file.
    let empty = run(&ctx, &request(path.clone(), "")).unwrap();
    assert!(!empty.success);
    assert!(
        empty.message.contains("missing 'meta' section"),
        "{}",
        empty.message
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}
#[test]
#[ignore = "requires prepared host truth tools and THTK_TEST_TRUTH_DIR"]
fn location_free_tool_error_remains_readable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.msgspec");
    fs::write(
        &path,
        "#pragma mapfile \"missing.map\"\nmeta {table:{0:{script:\"main\"}}}\nscript main {}\n",
    )
    .unwrap();
    let ctx = context(
        dir.path(),
        ScriptKind::Msg,
        ProjectSnapshot {
            game_version: "18".into(),
            ..Default::default()
        },
    );
    let result = run(
        &ctx,
        &ScriptRequest {
            operation: ScriptOperation::Check,
            input_path: path,
            output_path: None,
            source_text: None,
        },
    )
    .unwrap();
    assert!(!result.success);
    assert!(result.message.contains("missing.map"), "{}", result.message);
    assert!(result.diagnostics.is_empty());
}
