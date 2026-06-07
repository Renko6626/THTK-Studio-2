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

    let entry = mcp_object
        .entry("thtk-studio")
        .or_insert_with(|| json!({
            "type": "remote",
            "enabled": true,
            "url": "",
            "headers": {}
        }));
    let entry_obj = entry.as_object_mut().ok_or_else(|| {
        "opencode.json mcp.thtk-studio is not an object, refusing to overwrite".to_string()
    })?;
    entry_obj.insert("url".to_string(), json!(format!("http://127.0.0.1:{port}/mcp")));
    // Merge headers: preserve user-set headers, only update Authorization.
    let headers = entry_obj
        .entry("headers")
        .or_insert_with(|| json!({}));
    let headers_obj = headers.as_object_mut().ok_or_else(|| {
        "opencode.json mcp.thtk-studio.headers is not an object".to_string()
    })?;
    headers_obj.insert("Authorization".to_string(), json!("Bearer {env:THTK_MCP_TOKEN}"));

    let mut serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize opencode.json: {e}"))?;
    serialized.push('\n'); // POSIX 尾换行,避免 git diff 噪音
    fs::write(&path, serialized).map_err(|e| format!("Failed to write opencode.json: {e}"))
}

/// 在项目级 .codex/config.toml 写入/更新 [mcp_servers.thtk-studio](保格式,
/// 不动用户已有内容与注释)。返回 Ok(true) 表示本次新建了 entry(供 trust 提示)。
/// token 走 bearer_token_env_var = "THTK_MCP_TOKEN"(PTY 注入),不落盘。
pub fn upsert_codex_entry(project_root: &str, port: u16) -> Result<bool, String> {
    let dir = Path::new(project_root).join(".codex");
    let path = dir.join("config.toml");

    let content = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read .codex/config.toml: {e}"))?
    } else {
        String::new()
    };

    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!(".codex/config.toml is not valid TOML, refusing to overwrite: {e}"))?;

    // Obtain a mutable reference to the mcp_servers table, creating it as an implicit
    // table if absent, or returning Err if it exists but is not a table.
    let servers = match doc.as_table_mut().entry("mcp_servers") {
        toml_edit::Entry::Occupied(occ) => {
            let item = occ.into_mut();
            match item.as_table_mut() {
                Some(t) => t,
                None => return Err(
                    ".codex/config.toml mcp_servers is not a table, refusing to overwrite".to_string()
                ),
            }
        }
        toml_edit::Entry::Vacant(vac) => {
            let mut t = toml_edit::Table::new();
            t.set_implicit(true);
            vac.insert(toml_edit::Item::Table(t)).as_table_mut().expect("just inserted table")
        }
    };
    servers.set_implicit(true);

    // Determine whether the thtk-studio entry already exists BEFORE or_insert_with.
    let created = servers.get("thtk-studio").is_none();

    // Get or create the thtk-studio sub-table, updating only our two managed keys,
    // preserving any user-added keys (e.g. startup_timeout_sec, tool_timeout_sec).
    let entry = servers
        .entry("thtk-studio")
        .or_insert_with(|| toml_edit::Item::Table(toml_edit::Table::new()));
    let entry_table = entry.as_table_mut().ok_or_else(|| {
        ".codex/config.toml mcp_servers.thtk-studio is not a table, refusing to overwrite".to_string()
    })?;
    entry_table["url"] = toml_edit::value(format!("http://127.0.0.1:{port}/mcp"));
    entry_table["bearer_token_env_var"] = toml_edit::value("THTK_MCP_TOKEN");

    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create .codex dir: {e}"))?;
    fs::write(&path, doc.to_string())
        .map_err(|e| format!("Failed to write .codex/config.toml: {e}"))?;
    Ok(created)
}

/// 推送到输出面板的注册结果卡片。无 Tauri 依赖,便于单测。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationCard {
    pub title: String,
    pub body: String,
    pub level: String, // "info" | "error"
}

/// 在项目根注册所有 MCP 客户端配置。无 Tauri 依赖,返回需要推送到输出面板的卡片。
/// .mcp.json 总是写;opencode.json / .codex/config.toml 仅在检测到 CLI 时写。
pub fn register_clients(project_root: &str, port: u16, token: &str) -> Vec<RegistrationCard> {
    let mut cards = Vec::new();

    if let Err(e) = upsert_mcp_entry(project_root, port, token) {
        cards.push(RegistrationCard {
            title: "更新 .mcp.json 失败".to_string(),
            body: e,
            level: "error".to_string(),
        });
    }

    if cli_available("opencode") {
        if let Err(e) = upsert_opencode_entry(project_root, port) {
            cards.push(RegistrationCard {
                title: "更新 opencode.json 失败".to_string(),
                body: e,
                level: "error".to_string(),
            });
        }
    }

    if cli_available("codex") {
        match upsert_codex_entry(project_root, port) {
            Ok(true) => cards.push(RegistrationCard {
                title: "已写入 codex 项目配置".to_string(),
                body: ".codex/config.toml 已生成 thtk-studio MCP entry。codex 的项目级配置仅在受信目录生效:首次在本项目使用 codex 时,请在其提示中信任本目录。".to_string(),
                level: "info".to_string(),
            }),
            Ok(false) => {}
            Err(e) => cards.push(RegistrationCard {
                title: "更新 .codex/config.toml 失败".to_string(),
                body: e,
                level: "error".to_string(),
            }),
        }
    }

    cards
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

    // ---- upsert_codex_entry tests ----

    #[test]
    fn codex_creates_when_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        let result = upsert_codex_entry(&root, 39127);
        assert!(result.is_ok(), "upsert failed: {:?}", result.err());
        assert_eq!(result.unwrap(), true, "should return true (new entry)");

        let config_path = dir.path().join(".codex").join("config.toml");
        let content = fs::read_to_string(&config_path).expect("read config.toml");

        // Parse and verify the values
        let doc: toml_edit::DocumentMut = content.parse().expect("valid toml");
        let url = doc["mcp_servers"]["thtk-studio"]["url"]
            .as_str()
            .expect("url is string");
        assert!(url.contains("39127"), "url should contain port 39127, got: {url}");
        assert_eq!(url, "http://127.0.0.1:39127/mcp");

        let btev = doc["mcp_servers"]["thtk-studio"]["bearer_token_env_var"]
            .as_str()
            .expect("bearer_token_env_var is string");
        assert_eq!(btev, "THTK_MCP_TOKEN");

        // Assert no stray empty [mcp_servers] header — entry should appear as [mcp_servers.thtk-studio]
        assert!(
            content.contains("[mcp_servers.thtk-studio]"),
            "file should contain [mcp_servers.thtk-studio], got:\n{content}"
        );
        assert!(
            !content.contains("[mcp_servers]\n[mcp_servers.thtk-studio]"),
            "file should NOT have stray [mcp_servers] header before [mcp_servers.thtk-studio]"
        );
    }

    #[test]
    fn codex_preserves_content_and_comments() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let seed = "# my comment\nmodel = \"o3\"\n\n[mcp_servers.other]\nurl = \"http://x/mcp\"\n";
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("create .codex dir");
        fs::write(codex_dir.join("config.toml"), seed).expect("seed");

        // First call: thtk-studio absent → should return true (created)
        let result = upsert_codex_entry(&root, 39127);
        assert!(result.is_ok(), "first upsert failed: {:?}", result.err());
        assert_eq!(result.unwrap(), true, "first call should return true (new entry)");

        let content = fs::read_to_string(codex_dir.join("config.toml")).expect("read");
        assert!(content.contains("# my comment"), "comment should be preserved");
        assert!(content.contains("model = \"o3\""), "model key should be preserved");
        assert!(content.contains("[mcp_servers.other]"), "[mcp_servers.other] should be preserved");

        // Second call: thtk-studio now exists → should return false (updated)
        let content_before = fs::read_to_string(codex_dir.join("config.toml")).expect("read before 2nd");
        let result2 = upsert_codex_entry(&root, 39127);
        assert!(result2.is_ok(), "second upsert failed: {:?}", result2.err());
        assert_eq!(result2.unwrap(), false, "second call should return false (entry existed)");

        // File content should be functionally identical for same port
        let content_after = fs::read_to_string(codex_dir.join("config.toml")).expect("read after 2nd");
        // Parse both and check the key values are the same
        let doc_before: toml_edit::DocumentMut = content_before.parse().expect("valid toml before");
        let doc_after: toml_edit::DocumentMut = content_after.parse().expect("valid toml after");
        assert_eq!(
            doc_before["mcp_servers"]["thtk-studio"]["url"].as_str(),
            doc_after["mcp_servers"]["thtk-studio"]["url"].as_str(),
        );
    }

    #[test]
    fn codex_refuses_invalid_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let bad_content = "not [ valid";
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("create .codex dir");
        fs::write(codex_dir.join("config.toml"), bad_content).expect("seed");

        let result = upsert_codex_entry(&root, 39127);
        assert!(result.is_err(), "should return Err for invalid TOML");

        // File should be byte-identical
        let content_after = fs::read_to_string(codex_dir.join("config.toml")).expect("read");
        assert_eq!(content_after, bad_content, "file should be unchanged after error");
    }

    #[test]
    fn codex_refuses_non_table_mcp_servers() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let seed = "mcp_servers = 1\n";
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("create .codex dir");
        fs::write(codex_dir.join("config.toml"), seed).expect("seed");

        let result = upsert_codex_entry(&root, 39127);
        assert!(result.is_err(), "should return Err when mcp_servers is not a table");

        // File should be byte-identical (not written)
        let content_after = fs::read_to_string(codex_dir.join("config.toml")).expect("read");
        assert_eq!(content_after, seed, "file should be unchanged after error");
    }

    #[test]
    fn opencode_preserves_user_enabled_false() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        fs::write(
            dir.path().join("opencode.json"),
            r#"{"mcp":{"thtk-studio":{"type":"remote","enabled":false,"url":"http://old/mcp","headers":{"Authorization":"Bearer x","X-Custom":"keep"}}}}"#,
        )
        .expect("seed");

        upsert_opencode_entry(&root, 55555).expect("upsert");

        let value: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("opencode.json")).unwrap())
                .unwrap();
        // enabled must remain false (not reverted to true)
        assert_eq!(value["mcp"]["thtk-studio"]["enabled"], false, "enabled should be preserved as false");
        // url updated to new port
        assert_eq!(
            value["mcp"]["thtk-studio"]["url"],
            "http://127.0.0.1:55555/mcp",
            "url should be updated"
        );
        // Authorization updated
        assert_eq!(
            value["mcp"]["thtk-studio"]["headers"]["Authorization"],
            "Bearer {env:THTK_MCP_TOKEN}",
            "Authorization should be updated"
        );
        // user-set custom header preserved
        assert_eq!(
            value["mcp"]["thtk-studio"]["headers"]["X-Custom"],
            "keep",
            "X-Custom header should be preserved"
        );
    }

    #[test]
    fn opencode_refuses_non_object_entry() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let bad_content = r#"{"mcp":{"thtk-studio":5}}"#;
        fs::write(dir.path().join("opencode.json"), bad_content).expect("seed");

        let result = upsert_opencode_entry(&root, 55555);

        assert!(result.is_err(), "should return Err when thtk-studio entry is not an object");
        // File should be byte-identical (not written)
        assert_eq!(
            fs::read_to_string(dir.path().join("opencode.json")).unwrap(),
            bad_content,
            "file should be unchanged after error"
        );
    }

    #[test]
    fn codex_preserves_user_keys_in_entry() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        let seed = "[mcp_servers.thtk-studio]\nurl = \"http://old/mcp\"\nstartup_timeout_sec = 60\n";
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).expect("create .codex dir");
        fs::write(codex_dir.join("config.toml"), seed).expect("seed");

        let result = upsert_codex_entry(&root, 55555);
        assert!(result.is_ok(), "upsert failed: {:?}", result.err());
        assert_eq!(result.unwrap(), false, "entry existed so should return false");

        let content = fs::read_to_string(codex_dir.join("config.toml")).expect("read");
        let doc: toml_edit::DocumentMut = content.parse().expect("valid toml");

        // url is updated to contain the new port
        let url = doc["mcp_servers"]["thtk-studio"]["url"]
            .as_str()
            .expect("url is string");
        assert!(url.contains("55555"), "url should contain new port 55555, got: {url}");

        // bearer_token_env_var is written
        let btev = doc["mcp_servers"]["thtk-studio"]["bearer_token_env_var"]
            .as_str()
            .expect("bearer_token_env_var is string");
        assert_eq!(btev, "THTK_MCP_TOKEN");

        // user key startup_timeout_sec is preserved
        let timeout = doc["mcp_servers"]["thtk-studio"]["startup_timeout_sec"]
            .as_integer()
            .expect("startup_timeout_sec is integer");
        assert_eq!(timeout, 60, "startup_timeout_sec should be preserved");
    }

    // ---- register_clients tests ----

    #[test]
    fn register_clients_writes_mcp_json_always() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        let cards = register_clients(&root, 39127, "tok");

        // .mcp.json 必须被写入并包含我们的 entry
        let content = fs::read_to_string(dir.path().join(".mcp.json")).expect("read .mcp.json");
        let value: Value = serde_json::from_str(&content).expect("json");
        assert_eq!(
            value["mcpServers"]["thtk-studio"]["url"],
            "http://127.0.0.1:39127/mcp"
        );
        // 不依赖 CI 是否有 opencode/codex:只断言没有针对 .mcp.json 的错误卡片
        assert!(
            !cards
                .iter()
                .any(|c| c.level == "error" && c.title.contains(".mcp.json")),
            "should not report a .mcp.json error, got: {cards:?}"
        );
    }

    #[test]
    fn register_clients_reports_error_on_unwritable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        // 非法 JSON 强制 upsert_mcp_entry 走拒绝覆盖的 Err 路径
        fs::write(dir.path().join(".mcp.json"), "{not json").expect("seed");

        let cards = register_clients(&root, 39127, "tok");

        assert!(
            cards
                .iter()
                .any(|c| c.level == "error" && c.title.contains(".mcp.json")),
            "should report a .mcp.json error card, got: {cards:?}"
        );
    }
}
