# 多客户端 MCP 自动注册实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox syntax.

**Goal:** 稳定端口 + PTY 环境注入(B)→ opencode/codex 配置托管写入(A)→ SKILL.md 自助注册兜底(C)。
**Spec:** `docs/superpowers/specs/2026-06-07-multi-client-mcp-registration-design.md`
**环境:** 所有 cargo 命令需 conda 环境(见 CLAUDE.md / 项目记忆):
`P=/data/sunyunbo/miniconda3/envs/tauri-dev; export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig LD_LIBRARY_PATH=$P/lib PATH=$P/bin:$PATH`
管道取真实退出码用 `${PIPESTATUS[0]}`。当前 Rust 测试基线:19 passed。

### Task 1: 稳定端口(AppConfig.mcp_port + 绑定回退)— TDD

**Files:** Modify `src-tauri/src/config.rs`(AppConfig 加 `#[serde(default = "default_mcp_port")] pub mcp_port: u16`,default fn 返回 39127;同步 Default impl)、`src-tauri/src/modules/mcp/server.rs`。

server.rs:把绑定逻辑抽成可测函数:

```rust
/// 优先绑定配置端口;被占用(多实例等)回退随机端口。
pub async fn bind_preferred(port: u16) -> Result<tokio::net::TcpListener, String> {
    if port != 0 {
        match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) => eprintln!("[mcp] port {port} unavailable ({e}), falling back to ephemeral"),
        }
    }
    tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("MCP server bind failed: {e}"))
}
```

`start()` 读 `config.mcp_port`(经 AppHandle→AppState→config_manager)调用之。
测试(`#[tokio::test]`):先占用一个端口再 `bind_preferred(该端口)` → 成功且 local_addr 端口不同;`bind_preferred(0)` → 成功。
验证:`cargo test --manifest-path src-tauri/Cargo.toml` → 21 passed。Commit: `feat: stable MCP port with ephemeral fallback`

### Task 2: PTY 环境注入 — TDD

**Files:** Modify `src-tauri/src/common/pty.rs`。

1. `PtyManager::create` 增参 `envs: Vec<(String, String)>`(`cwd` 之后);spawn 循环里对每个 candidate 的 CommandBuilder 逐个 `cmd.env(key, value)`(portable-pty CommandBuilder 有 `env(key, val)`)。
2. 既有 3 个测试调用处补 `Vec::new()`。新增 unix 测试 `env_injection_roundtrip`:`envs = [("THTK_TEST_ENV","env_ok_42")]`,写入 `echo $THTK_TEST_ENV\n`,断言输出含 `env_ok_42`。
3. `pty_create` 命令:从 `state.mcp_server` 读 info,Some 时注入 `THTK_MCP_URL=http://127.0.0.1:{port}/mcp` 与 `THTK_MCP_TOKEN={token}`,None 时空 vec。
验证:22 passed。Commit: `feat: inject MCP endpoint env vars into PTY sessions`

### Task 3: opencode 配置托管 + CLI 检测 — TDD

**Files:** Modify `src-tauri/src/common/mcp_config.rs`(同文件追加,模式照抄 upsert_mcp_entry)、`src-tauri/src/main.rs`(set_project_root 接线)。

1. `pub fn cli_available(name: &str) -> bool`:遍历 `PATH`(`std::env::split_paths`)找 `name`/`name.exe`/`name.cmd`(Windows bat/cmd 由 npm 包装常见)。测试:临时目录放可执行文件 + 改 PATH 环境跑(测试内 `std::env::set_var` 注意并发——用 `#[serial]`?无 serial_test 依赖 → 改为参数化:`fn cli_available_in(paths: &OsStr, name: &str)` 接受 PATH 字符串,公开薄包装读真实 PATH。测试只测参数化版本,不动全局 env)。
2. `pub fn upsert_opencode_entry(project_root: &str, port: u16) -> Result<(), String>`:upsert `opencode.json` 顶层 `mcp.thtk-studio` = spec §A.3 的 JSON(token 用字面量 `{env:THTK_MCP_TOKEN}` 字符串);非法 JSON 拒绝覆盖;尾换行。3 个测试(新建/保留其他键含已有 mcp 其他 server/拒绝非法)。
3. main.rs `set_project_root`:在 upsert_mcp_entry 之后,`if mcp_config::cli_available("opencode") { upsert_opencode_entry(...) }`,失败 eprintln。
验证:26 passed(+4)。Commit: `feat: managed opencode.json MCP entry (env-ref token, CLI-gated)`

### Task 4: codex 配置托管(toml_edit)+ trust 提示卡 — TDD

**Files:** Modify `src-tauri/Cargo.toml`(`toml_edit = "0.23"`,版本以 `cargo add toml_edit --dry-run` 实际为准)、`src-tauri/src/common/mcp_config.rs`、`src-tauri/src/main.rs`。

1. `pub fn upsert_codex_entry(project_root: &str, port: u16) -> Result<bool, String>`(返回"本次是否新建了 entry",供 trust 提示判断):
   - 路径 `{root}/.codex/config.toml`,目录不存在则建
   - `toml_edit::DocumentMut` 解析(不存在按空文档);解析失败 → Err 不覆盖
   - `doc["mcp_servers"]["thtk-studio"]["url"] = "http://127.0.0.1:{port}/mcp"`、`["bearer_token_env_var"] = "THTK_MCP_TOKEN"`;注意用 `implicit table` 方式避免顶层产生 `[mcp_servers]` 空表破坏已有格式(参考 toml_edit 文档 `Table::set_implicit`)
   - 保留文件中其他内容(toml_edit 保格式特性)
   - 3 个测试:新建(返回 true)/已有其他 server 与注释保留(返回 false,二次 upsert)/拒绝非法 TOML
2. main.rs:`if mcp_config::cli_available("codex")` → upsert;返回 Ok(true)(新建)时 `app_handle.emit("mcp://report", json!({"title":"已写入 codex 项目配置","body":"首次使用需在 codex 中信任本目录(.codex/config.toml 为项目级配置)。","level":"info","path":null}))`(set_project_root 已有 app_handle 参数)。
验证:29 passed(+3)。Commit: `feat: managed .codex/config.toml MCP entry with trust notice`

### Task 5: SKILL.md 模板自助注册节 + 文档

**Files:** Modify `src-tauri/assets/ecl-skill-template.md`、`CLAUDE.md`(Agent channel 节补两句)、`editor-shell-status.md`(§2.8 补多客户端注册;验收清单加 spec §验收 5 条)。

模板新增一节(放"注意事项"前):

```markdown
## 接入 thtk-studio MCP(若你的工具未配置)

终端环境变量 `THTK_MCP_URL` / `THTK_MCP_TOKEN` 是 IDE 提供的 MCP 接入点
(也可读项目根 `.mcp.json`)。若 `/mcp` 或工具列表里没有 thtk-studio:

- **codex**:`codex mcp add thtk-studio --url "$THTK_MCP_URL" --bearer-token-env-var THTK_MCP_TOKEN`
- **opencode**:把 thtk-studio 写入项目根 `opencode.json` 的 `mcp` 段
  (url 用 `$THTK_MCP_URL` 的值,headers.Authorization 用 `Bearer {env:THTK_MCP_TOKEN}`),
  然后提醒用户重启 opencode
- 任何改动都需要**新会话**才生效;完成后用 `report_to_user` 告知用户
```

验证:`cargo test`(模板被 include_str!,ai_pack 测试需仍过)+ `npm run build`。Commit: `docs: self-registration guide in skill template + status docs`

## 自审备注

- Task 2 改 `create` 签名会破坏 Task 1 之前的调用——顺序执行,实现者改全所有调用点
- `{env:THTK_MCP_TOKEN}` 在 opencode.json 里是**字面字符串**(opencode 自己展开),Rust 写入时不要做任何替换
- 端口稳定后 `.mcp.json` 仍每次启动重写(token 字面量轮换)——现状已如此,无需改动
