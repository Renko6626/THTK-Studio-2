# Spec 1: Agent 通道 — 真 PTY 终端 + IDE 作为 MCP 服务

日期:2026-06-07
状态:已与用户确认设计,待实现

## 0. 背景与定位转向

THTK-Studio 从"纯面向人类的东方脚本编辑器"转向**双主体 IDE**:

- 人类在 Monaco 编辑器里干活
- AI agent(claude code / codex 等现成 CLI)在内嵌终端里干活
- 双方共享同一套诊断、eclmap 语义、thtk 工具链服务

整体转向拆为三个 spec 依次实施:

1. **Spec 1(本文)**:Agent 通道 — 真 PTY 终端 + MCP server
2. Spec 2:诊断管线 — 深度语义检查(真解析器)、自动后台编译、诊断增量推送
3. Spec 3:AI 辅助 — 双协议 provider 抽象(Anthropic 原生 + OpenAI 兼容)、Monaco 内联补全、AI 增强诊断

选型决策(已确认):不自建 agent loop(不用 Agent SDK 内置面板),复用成熟 CLI;
MCP server 跑在 Tauri 主进程内(进程内 HTTP,方案 A),不做 sidecar(方案 B)、
不做文件协议(方案 C)。理由:核心资产(结构化诊断、eclmap 语义、thecl 封装)
全在主进程 Rust 里,进程内是唯一零复制、零转发的方案。

## 1. 总体架构

```
┌─ Tauri 主进程 (Rust) ─────────────────────────┐
│  AppState (已有: config/project_root/watcher)  │
│  ├── common/pty/        新: PTY 会话管理        │
│  └── modules/mcp/       新: MCP server (rmcp)  │
│       └── 直接调用已有 ecl::compiler/error_parser/map_parser
└──────┬───────────────────────┬────────────────┘
       │ Tauri events (流式)    │ Streamable HTTP (127.0.0.1:随机端口)
┌──────▼──────────┐    ┌───────▼──────────┐
│ 前端 xterm.js    │    │ claude code      │
│ TerminalPanel   │◄───│ (跑在 PTY 终端里,  │
│ (用户也在这里看)  │    │  经 .mcp.json 连回 IDE) │
└─────────────────┘    └──────────────────┘
```

核心原则:**MCP 工具和问题面板调用同一批 Rust 函数**,人和 agent 看到的诊断严格同源。

新增 Rust 依赖:`portable-pty`(PTY,Windows 走 ConPTY / Unix 走 openpty)、
`rmcp`(官方 Rust MCP SDK,Streamable HTTP server 特性)、`tokio`/`axum`
(rmcp 传输层需要;Tauri v2 本身跑在 tokio 上,无冲突)。

新增前端依赖:`@xterm/xterm` + `@xterm/addon-fit`。

## 2. PTY 终端(替换现有命令面板)

现状问题:`common/terminal.rs` 是一次性命令执行(spawn → 收集输出 → 返回),
且硬编码 `powershell.exe`/`cmd.exe`,在 Linux 上不可用。project.md §4.2.1 已明确
禁止在该模型上叠加交互能力。

### Rust 侧(`common/pty/`)

- `PtyManager` 挂在 `AppState`,内部 `HashMap<SessionId, PtySession>`
- Tauri 命令:
  - `pty_create(shell?, cwd?, cols, rows) -> session_id`
  - `pty_write(session_id, data)`
  - `pty_resize(session_id, cols, rows)`
  - `pty_kill(session_id)`
- 每会话一个 reader 线程,输出以 `pty://output/{id}` 事件流式推送;
  进程退出时发 `pty://exit/{id}`(带退出码)
- Shell 探测跨平台:Windows → `pwsh` → `powershell` → `cmd` 依次;
  Unix → `$SHELL` → `bash`
- 默认 cwd = 当前项目根;无项目时为用户主目录

### 前端侧

- `TerminalPanel.vue` 用 `@xterm/xterm` + fit addon 重写,支持多终端 tab
  (新建/关闭/切换)
- `stores/terminal.js` 改为会话模型(`sessions[]` / `activeSessionId`)
- xterm scrollback 上限 5000 行(解决 editor-shell-status.md §6 记录的
  "终端 store 无内存上限"风险)
- resize 经 fit addon → `pty_resize` 同步到 PTY

旧 `run_shell_command` 保留给现有内部调用方,但终端面板不再使用;
后续在 Spec 2 中评估是否彻底移除。

## 3. MCP server 与工具清单 v1

`modules/mcp/`:rmcp Streamable HTTP server,应用启动时绑定 `127.0.0.1:0`
(OS 分配随机端口),持有 AppState 句柄。模块形状仿照 `modules/ecl/`
(`mod.rs` + `server.rs` + `tools.rs`)。

工具六个,全部返回结构化 JSON:

| 工具 | 作用 | 复用 |
|---|---|---|
| `get_workspace_info` | 项目根、5 个 thtk 工具的路径与版本状态 | `common/toolchain.rs` |
| `check_ecl` | 编译检查(不留产物),返回结构化诊断列表 | `ecl::compiler` + `error_parser` |
| `compile_ecl` | 真编译,产物落盘,返回诊断 + 产物路径 | 同上 |
| `decompile_ecl` | 反编译 .ecl → 文本 | 同上 |
| `lookup_ecl_semantics` | 按指令名/opcode 查 eclmap 语义(签名、参数、枚举) | `ecl::map_parser` |
| `report_to_user` | agent 主动向人类汇报:推结构化卡片到输出/问题面板(经 `mcp://report` 事件 → `workbenchReports` store) | 前端 reports store |

`report_to_user` 是"双主体"的关键件:agent 干完活不只在终端里打字,
还能把结论结构化地放进人类常驻的面板。

v1 刻意**不做**"读取编辑器打开文件/光标位置"等需要前端状态回传的工具,留给 v2。

## 3.5 AI 辅助包生成(skill 脚手架 + eclmap 导出)

背景:ECL 是小众 DSL,模型训练数据中几乎不存在。agent 需要两层知识:

1. **语言知识**(sub/timeline 概念、难度分支、寄存器变量、工作流)——
   稳定、跨版本,放进项目级 skill,agent 启动即懂
2. **逐版本指令表**(每作 eclmap 不同,数百条签名)——量大且按需,
   由 IDE 从 eclmap 自动生成 reference 文件(progressive disclosure),
   精准点查仍走 `lookup_ecl_semantics` 工具

新增 Tauri 命令 `generate_ai_assist_pack`(菜单入口),在项目根生成:

```
.claude/skills/ecl-modding/
├── SKILL.md                     ← 内置模板:语言概念 + 工作流 +
│                                   何时读 references、何时调 MCP 工具
└── references/
    ├── {version}-instructions.md ← 从当前加载的 eclmap 生成(map_parser)
    └── {version}-enums.md
```

规则:

- `SKILL.md` 仅在不存在时写入(用户可能手改过,不覆盖)
- `references/` 每次生成都重写(保证与编译器实际使用的 eclmap 永不漂移)
- eclmap 切换/重载后,提示用户可重新生成

## 4. 接线与安全

- 打开项目时(`set_project_root`),IDE 在项目根**非破坏性合并** `.mcp.json`:
  只增/改 `thtk-studio` 一个 entry,不动用户已有的其他 MCP server 配置:

  ```json
  {
    "mcpServers": {
      "thtk-studio": {
        "type": "http",
        "url": "http://127.0.0.1:{port}/mcp",
        "headers": { "Authorization": "Bearer {token}" }
      }
    }
  }
  ```

- 每次应用启动随机生成 bearer token,server 校验所有请求;只绑 127.0.0.1
- 端口每次启动会变 → 每次打开项目都重写 entry,陈旧端口自然失效

## 5. 明确不在本 spec 内

- 深度语义检查、自动后台编译、诊断增量推送(Spec 2)
- provider 抽象、Monaco 内联补全、AI 增强诊断(Spec 3)
- 内置 agent 面板(Agent SDK)— 观察 PTY+MCP 够不够用再决策
- thmsg/thanm/thstd 工具链扩展(原路线图工作,与本转向并行)

## 6. 错误处理与测试

- MCP 工具错误统一返回结构化 `{code, message, hint}`,不裸抛字符串
- PTY 会话崩溃 → `pty://exit` 带退出码,前端展示并允许重开
- `cargo test`:
  - MCP 工具 handler 直调测试(不经 HTTP)
  - `.mcp.json` 非破坏性合并逻辑测试
  - PTY echo 往返测试(Unix 下可跑)
- 前端无测试设施,沿用"构建即验证" + 手动验收:在内嵌终端跑 `claude`,
  让它调 `check_ecl` 看到与问题面板一致的真实诊断

## 7. 验收标准

1. Linux 与 Windows 上内嵌终端可运行交互式程序(vim、Python REPL、claude code)
2. 在内嵌终端启动 `claude`,无需手动配置即可发现 `thtk-studio` MCP server
3. agent 调用 `check_ecl` 返回的诊断与问题面板显示一致
4. agent 调用 `report_to_user` 后,卡片出现在输出面板
5. `.mcp.json` 中用户手写的其他 server 配置在 IDE 重写后保持原样
6. `generate_ai_assist_pack` 生成的 references 与 eclmap 内容一致;
   重复执行不覆盖用户已修改的 SKILL.md,但 references 总是刷新
