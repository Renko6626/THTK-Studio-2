use crate::app_state::AppState;
use crate::modules::mcp::tools;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData as McpError,
};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};

pub struct McpServerInfo {
    pub port: u16,
    pub token: String,
}

#[derive(Clone)]
pub struct ThtkMcp {
    app: AppHandle,
    tool_router: ToolRouter<Self>,
}

fn json_result(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// spec §6：工具错误统一带结构化 data {code, message, hint}
fn tool_error(message: String) -> McpError {
    let hint = if message.contains("configured") {
        "在 THTK-Studio 设置中配置 thtk 工具路径 (thtk_dir / thecl_path / eclmap_path)"
    } else {
        "检查文件路径是否为绝对路径，以及 get_workspace_info 中工具链是否可用"
    };
    McpError::internal_error(
        message.clone(),
        Some(serde_json::json!({
            "code": "tool_failed",
            "message": message,
            "hint": hint,
        })),
    )
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SourcePathParams {
    /// .decl 源文件的绝对路径
    pub source_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompileParams {
    /// .decl 源文件的绝对路径
    pub source_path: String,
    /// 可选的输出 .ecl 路径；省略时按约定推断（.decl → .ecl）
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DecompileParams {
    /// .ecl 二进制文件的绝对路径
    pub binary_path: String,
    /// 可选的输出 .decl 路径
    pub output_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupParams {
    /// 指令名子串（如 "wait"）、精确 opcode（如 "23"）或寄存器名（如 "I0"）
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReportParams {
    /// 报告标题（显示在输出面板分组上）
    pub title: String,
    /// 报告正文，支持多行
    pub body: String,
    /// "info" | "success" | "warning" | "error"
    pub level: Option<String>,
    /// 关联文件的绝对路径（可选）
    pub path: Option<String>,
}

#[tool_router]
impl ThtkMcp {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
        }
    }

    fn config(&self) -> crate::config::AppConfig {
        self.app.state::<AppState>().config_manager.get_config()
    }

    fn project_root(&self) -> Option<String> {
        self.app
            .state::<AppState>()
            .current_project_root
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    #[tool(
        description = "获取 THTK-Studio 工作区信息：项目根目录、默认游戏版本、五个 thtk 工具（thecl/thmsg/thanm/thstd/thdat）的可用状态与版本。"
    )]
    async fn get_workspace_info(&self) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        let value = tokio::task::spawn_blocking(move || {
            tools::workspace_info(&config, root.as_deref())
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join: {e}"), None))?;
        json_result(value)
    }

    #[tool(
        description = "编译检查一个 ECL 源文件（.decl）：运行 thecl 编译但不保留产物，返回结构化诊断列表（与 IDE 问题面板同源）。改完代码后用它验证。"
    )]
    async fn check_ecl(
        &self,
        Parameters(params): Parameters<SourcePathParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        let value = tokio::task::spawn_blocking(move || {
            tools::check_ecl(&config, root.as_deref(), &params.source_path)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join: {e}"), None))?
        .map_err(tool_error)?;
        json_result(value)
    }

    #[tool(description = "编译 ECL 源文件（.decl → .ecl），产物落盘，返回诊断与产物路径。")]
    async fn compile_ecl(
        &self,
        Parameters(params): Parameters<CompileParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        let value = tokio::task::spawn_blocking(move || {
            tools::compile_ecl(&config, root.as_deref(), &params.source_path, params.output_path)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join: {e}"), None))?
        .map_err(tool_error)?;
        json_result(value)
    }

    #[tool(description = "反编译 ECL 二进制（.ecl → .decl 文本），返回诊断与产物路径。")]
    async fn decompile_ecl(
        &self,
        Parameters(params): Parameters<DecompileParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        let value = tokio::task::spawn_blocking(move || {
            tools::decompile_ecl(&config, root.as_deref(), &params.binary_path, params.output_path)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join: {e}"), None))?
        .map_err(tool_error)?;
        json_result(value)
    }

    #[tool(
        description = "按指令名子串、opcode 或寄存器名查询当前 eclmap 语义数据（签名、参数、全局寄存器）。写 ECL 前先查签名。"
    )]
    async fn lookup_ecl_semantics(
        &self,
        Parameters(params): Parameters<LookupParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config();
        let root = self.project_root();
        let value = tokio::task::spawn_blocking(move || {
            let map_path =
                tools::resolve_map_path(&config, root.as_deref()).map_err(tool_error)?;
            tools::lookup_semantics(&map_path, &params.query).map_err(tool_error)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join: {e}"), None))??;
        json_result(value)
    }

    #[tool(
        description = "向 IDE 用户的输出面板推送一张结构化报告卡片。完成一项工作或发现重要问题时，用它向人类汇报结论。"
    )]
    async fn report_to_user(
        &self,
        Parameters(params): Parameters<ReportParams>,
    ) -> Result<CallToolResult, McpError> {
        self.app
            .emit(
                "mcp://report",
                serde_json::json!({
                    "title": params.title,
                    "body": params.body,
                    "level": params.level.unwrap_or_else(|| "info".to_string()),
                    "path": params.path,
                }),
            )
            .map_err(|e| McpError::internal_error(format!("emit failed: {e}"), None))?;
        json_result(serde_json::json!({ "delivered": true }))
    }
}

// rmcp 1.7 的 #[tool_handler] 默认每次调用都重建 Self::tool_router()，
// 这里显式指向构造时缓存的字段
#[tool_handler(router = self.tool_router)]
impl ServerHandler for ThtkMcp {
    fn get_info(&self) -> ServerInfo {
        // rmcp 1.7 的 InitializeResult 是 #[non_exhaustive]，只能走构造器
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "THTK-Studio：东方 Project 脚本魔改 IDE。\
                 工作流：decompile_ecl 把 .ecl 反编译为 .decl 文本 → 编辑 → check_ecl 验证 → compile_ecl 产出。\
                 写 ECL 指令前先 lookup_ecl_semantics 查签名；完成任务后 report_to_user 向用户汇报。",
            )
    }
}

/// 给任意 axum Router 套上 bearer token 校验（抽出为独立函数以便单测）。
fn with_bearer_auth(router: axum::Router, token: &str) -> axum::Router {
    use axum::response::IntoResponse;

    let expected = format!("Bearer {token}");
    router.layer(axum::middleware::from_fn(
        move |req: axum::extract::Request, next: axum::middleware::Next| {
            let expected = expected.clone();
            async move {
                let authorized = req
                    .headers()
                    .get(axum::http::header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    == Some(expected.as_str());
                if authorized {
                    next.run(req).await
                } else {
                    axum::http::StatusCode::UNAUTHORIZED.into_response()
                }
            }
        },
    ))
}

/// 优先绑定配置端口;被占用(多实例等)回退随机端口。
pub async fn bind_preferred(port: u16) -> Result<tokio::net::TcpListener, String> {
    if port != 0 {
        match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) => {
                eprintln!("[mcp] port {port} unavailable ({e}), falling back to ephemeral")
            }
        }
    }
    tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("MCP server bind failed: {e}"))
}

/// 绑定 127.0.0.1 随机端口，带 bearer token 校验，返回端口与 token。
/// serve 循环在 tauri 异步运行时中后台运行。
pub async fn start(app: AppHandle) -> Result<McpServerInfo, String> {
    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    };

    let token = uuid::Uuid::new_v4().simple().to_string();

    let service_app = app.clone();
    let service = StreamableHttpService::new(
        move || Ok(ThtkMcp::new(service_app.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );

    let router = with_bearer_auth(
        axum::Router::new().nest_service("/mcp", service),
        &token,
    );

    let mcp_port = app.state::<AppState>().config_manager.get_config().mcp_port;
    let listener = bind_preferred(mcp_port).await?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("MCP server addr failed: {e}"))?
        .port();

    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            eprintln!("[mcp] server stopped: {error}");
        }
    });

    Ok(McpServerInfo { port, token })
}

#[cfg(test)]
mod tests {
    use super::{bind_preferred, with_bearer_auth};
    use std::io::{Read, Write};

    #[tokio::test]
    async fn bind_preferred_falls_back_when_port_taken() {
        let blocker = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind blocker");
        let taken_port = blocker.local_addr().expect("addr").port();

        let listener = bind_preferred(taken_port).await.expect("fallback bind");
        let bound_port = listener.local_addr().expect("addr").port();

        assert_ne!(bound_port, taken_port, "must fall back to a different port");
    }

    #[tokio::test]
    async fn bind_preferred_uses_requested_port_when_free() {
        // 找空闲端口→释放→立刻请求,存在小概率被并发测试/其他进程抢占的竞态;
        // 重试若干轮把 flake 概率压到忽略不计(任意一轮成功即证明语义正确)。
        for attempt in 0..5 {
            let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("probe");
            let free_port = probe.local_addr().expect("addr").port();
            drop(probe);

            let listener = bind_preferred(free_port).await.expect("bind");
            if listener.local_addr().expect("addr").port() == free_port {
                return; // 语义验证成功
            }
            eprintln!("attempt {attempt}: port {free_port} got snatched, retrying");
        }
        panic!("bind_preferred never used the requested free port in 5 attempts");
    }

    fn raw_request(addr: std::net::SocketAddr, auth: Option<&str>) -> String {
        let mut stream = std::net::TcpStream::connect(addr).expect("connect");
        let auth_line = auth
            .map(|value| format!("Authorization: {value}\r\n"))
            .unwrap_or_default();
        write!(
            stream,
            "GET /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth_line}Connection: close\r\n\r\n"
        )
        .expect("write request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");
        response
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bearer_auth_rejects_missing_or_wrong_token() {
        let router = with_bearer_auth(
            axum::Router::new().route("/mcp", axum::routing::get(|| async { "ok" })),
            "secret-token",
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });

        let no_header = tokio::task::spawn_blocking(move || raw_request(addr, None))
            .await
            .expect("join");
        assert!(
            no_header.starts_with("HTTP/1.1 401"),
            "expected 401 without Authorization, got: {no_header}"
        );

        let wrong = tokio::task::spawn_blocking(move || {
            raw_request(addr, Some("Bearer wrong-token"))
        })
        .await
        .expect("join");
        assert!(
            wrong.starts_with("HTTP/1.1 401"),
            "expected 401 with wrong token, got: {wrong}"
        );

        let ok = tokio::task::spawn_blocking(move || {
            raw_request(addr, Some("Bearer secret-token"))
        })
        .await
        .expect("join");
        assert!(
            ok.starts_with("HTTP/1.1 200"),
            "expected 200 with correct token, got: {ok}"
        );
    }
}
