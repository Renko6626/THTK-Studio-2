use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// 在项目根的 .mcp.json 中写入/更新 thtk-studio 这一个 server entry,
/// 不动用户已有的其他 server 和顶层键。文件不是合法 JSON 时报错而非覆盖。
pub fn upsert_mcp_entry(project_root: &str, port: u16, token: &str) -> Result<(), String> {
    let path = Path::new(project_root).join(".mcp.json");

    let mut root: Value = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read .mcp.json: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!(".mcp.json is not valid JSON, refusing to overwrite: {e}"))?
    } else {
        json!({})
    };

    let root_object = root
        .as_object_mut()
        .ok_or_else(|| ".mcp.json top level is not an object, refusing to overwrite".to_string())?;

    let servers = root_object
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    let servers_object = servers
        .as_object_mut()
        .ok_or_else(|| ".mcp.json mcpServers is not an object".to_string())?;

    servers_object.insert(
        "thtk-studio".to_string(),
        json!({
            "type": "http",
            "url": format!("http://127.0.0.1:{port}/mcp"),
            "headers": { "Authorization": format!("Bearer {token}") }
        }),
    );

    let mut serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize .mcp.json: {e}"))?;
    serialized.push('\n'); // POSIX 尾换行,避免 git diff 噪音
    fs::write(&path, serialized).map_err(|e| format!("Failed to write .mcp.json: {e}"))
}

/// 在给定的 PATH 字符串中查找可执行文件(name、name.exe、name.cmd)。
/// 测试用参数化版本;运行时用 cli_available 读真实 PATH。
pub fn cli_available_in(path_var: &std::ffi::OsStr, name: &str) -> bool {
    for dir in std::env::split_paths(path_var) {
        for candidate in [
            name.to_string(),
            format!("{name}.exe"),
            format!("{name}.cmd"),
        ] {
            if dir.join(&candidate).is_file() {
                return true;
            }
        }
    }
    false
}

pub fn cli_available(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| cli_available_in(&path, name))
        .unwrap_or(false)
}

/// 在项目根 opencode.json 写入/更新 mcp.thtk-studio entry(非破坏)。
/// token 用 {env:THTK_MCP_TOKEN} 引用(opencode 启动时自行展开;
/// 我们的 PTY 会话注入了该变量),端口不变则文件不再每次启动变化。
pub fn upsert_opencode_entry(project_root: &str, port: u16) -> Result<(), String> {
    let path = Path::new(project_root).join("opencode.json");

    let mut root: Value = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read opencode.json: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("opencode.json is not valid JSON, refusing to overwrite: {e}"))?
    } else {
        json!({})
    };

    let root_object = root
        .as_object_mut()
        .ok_or_else(|| "opencode.json top level is not an object, refusing to overwrite".to_string())?;

    let mcp = root_object
        .entry("mcp")
        .or_insert_with(|| json!({}));
    let mcp_object = mcp
        .as_object_mut()
        .ok_or_else(|| "opencode.json mcp is not an object".to_string())?;

    mcp_object.insert(
        "thtk-studio".to_string(),
        json!({
            "type": "remote",
            "enabled": true,
            "url": format!("http://127.0.0.1:{port}/mcp"),
            "headers": { "Authorization": "Bearer {env:THTK_MCP_TOKEN}" }
        }),
    );

    let mut serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize opencode.json: {e}"))?;
    serialized.push('\n'); // POSIX 尾换行,避免 git diff 噪音
    fs::write(&path, serialized).map_err(|e| format!("Failed to write opencode.json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_mcp_json_when_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        upsert_mcp_entry(&root, 12345, "tok-abc").expect("upsert");

        let content = fs::read_to_string(dir.path().join(".mcp.json")).expect("read");
        let value: Value = serde_json::from_str(&content).expect("json");
        assert_eq!(
            value["mcpServers"]["thtk-studio"]["url"],
            "http://127.0.0.1:12345/mcp"
        );
        assert_eq!(
            value["mcpServers"]["thtk-studio"]["headers"]["Authorization"],
            "Bearer tok-abc"
        );
        assert_eq!(value["mcpServers"]["thtk-studio"]["type"], "http");
    }

    #[test]
    fn preserves_existing_servers_and_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"my-tool":{"type":"stdio","command":"foo"}},"custom":1}"#,
        )
        .expect("seed");

        upsert_mcp_entry(&root, 999, "t").expect("upsert");

        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(value["mcpServers"]["my-tool"]["command"], "foo");
        assert_eq!(value["custom"], 1);
        assert_eq!(value["mcpServers"]["thtk-studio"]["type"], "http");
    }

    #[test]
    fn refuses_to_clobber_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(dir.path().join(".mcp.json"), "{not json").expect("seed");

        let result = upsert_mcp_entry(&root, 1, "t");

        assert!(result.is_err());
        // 原文件保持原样
        assert_eq!(
            fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            "{not json"
        );
    }

    // ---- cli_available_in tests ----

    #[test]
    fn cli_available_in_finds_executable_by_name() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Create a file named "opencode" in the temp dir
        fs::write(dir.path().join("opencode"), "").expect("create file");

        let path_os = std::env::join_paths([dir.path()]).expect("join_paths");
        assert!(
            cli_available_in(&path_os, "opencode"),
            "should find 'opencode' in temp dir"
        );
        assert!(
            !cli_available_in(&path_os, "codex"),
            "should NOT find 'codex' in temp dir"
        );
    }

    // ---- upsert_opencode_entry tests ----

    #[test]
    fn opencode_creates_when_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        upsert_opencode_entry(&root, 39127).expect("upsert");

        let content = fs::read_to_string(dir.path().join("opencode.json")).expect("read");
        let value: Value = serde_json::from_str(&content).expect("json");
        assert_eq!(
            value["mcp"]["thtk-studio"]["url"],
            "http://127.0.0.1:39127/mcp"
        );
        assert_eq!(
            value["mcp"]["thtk-studio"]["headers"]["Authorization"],
            "Bearer {env:THTK_MCP_TOKEN}"
        );
        assert_eq!(value["mcp"]["thtk-studio"]["type"], "remote");
        assert_eq!(value["mcp"]["thtk-studio"]["enabled"], true);
    }

    #[test]
    fn opencode_preserves_existing_keys_and_servers() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(
            dir.path().join("opencode.json"),
            r#"{"mcp":{"other":{"type":"local"}},"theme":"x"}"#,
        )
        .expect("seed");

        upsert_opencode_entry(&root, 9999).expect("upsert");

        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("opencode.json")).unwrap())
                .unwrap();
        assert_eq!(value["mcp"]["other"]["type"], "local");
        assert_eq!(value["theme"], "x");
        assert_eq!(value["mcp"]["thtk-studio"]["type"], "remote");
    }

    #[test]
    fn opencode_refuses_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let bad_content = "{not json";
        fs::write(dir.path().join("opencode.json"), bad_content).expect("seed");

        let result = upsert_opencode_entry(&root, 1);

        assert!(result.is_err());
        // 原文件保持原样
        assert_eq!(
            fs::read_to_string(dir.path().join("opencode.json")).unwrap(),
            bad_content
        );
    }
}
