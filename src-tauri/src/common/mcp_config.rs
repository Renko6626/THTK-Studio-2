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

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize .mcp.json: {e}"))?;
    fs::write(&path, serialized).map_err(|e| format!("Failed to write .mcp.json: {e}"))
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
}
