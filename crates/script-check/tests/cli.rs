use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
#[test]
#[ignore = "requires prepared host truth tools and THTK_TEST_TRUTH_DIR"]
fn headless_json_exit_codes_and_new_stdin_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join(".thtk-project.json"), r#"{"gameVersion":"18"}"#).unwrap();
    let source = root.join("dialog.msgspec");
    let valid = "meta {table:{0:{script:\"main\"}}}\nscript main {}\n";
    std::fs::write(&source, valid).unwrap();
    let truth =
        PathBuf::from(std::env::var_os("THTK_TEST_TRUTH_DIR").expect("set THTK_TEST_TRUTH_DIR"));
    let command = |path: &std::path::Path| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_thtk-check"));
        cmd.env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env("XDG_CONFIG_HOME", root.join("config"));
        cmd.arg("check")
            .arg(path)
            .arg("--truth-dir")
            .arg(&truth)
            .arg("--json");
        cmd
    };
    let output = command(&source).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["context"]["gameVersion"], 18);
    assert_eq!(data["result"]["success"], true);
    assert!(output.stderr.is_empty());
    let missing_map = root.join("missing.msgm");
    let output = command(&source)
        .arg("--map")
        .arg(&missing_map)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["context"]["gameVersion"], 18);
    assert_eq!(
        data["context"]["mapPaths"][1],
        missing_map.to_str().unwrap()
    );
    assert_eq!(data["result"]["success"], false);
    assert!(data["result"]["message"]
        .as_str()
        .unwrap()
        .contains(missing_map.to_str().unwrap()));
    assert!(output.stderr.is_empty());
    std::fs::write(
        &source,
        "meta {table:{0:{script:\"main\"}}}\nscript main {\n unknown();\n}",
    )
    .unwrap();
    let output = command(&source).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["result"]["diagnostics"][0]["line"], 3);
    let new = root.join("new.msgspec");
    let mut child = command(&new)
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(valid.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(!new.exists());
    let output = command(&source)
        .arg("--truth-dir")
        .arg(root.join("missing"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data.as_object().unwrap().len(), 2);
    assert_eq!(data["context"]["gameVersion"], 18);
    assert_eq!(data["result"]["success"], false);
    assert!(data["result"]["message"]
        .as_str()
        .unwrap()
        .contains("trumsg"));
}
