# Spec 1.5: 多客户端 MCP 自动注册(B 打底 + A 主干 + C 兜底)

日期:2026-06-07 状态:已批准

## 目标

claude code / codex / opencode 在 THTK-Studio 内嵌终端里使用时,尽可能零手动接入 IDE 的 MCP server。

## 背景事实(调研结论,决定设计)

- claude code:读项目根 `.mcp.json`;`${VAR}` 在 URL 可用但 **headers 展开有未修复 bug**(#51581)→ token 必须写字面量并每次启动刷新
- codex:`~/.codex/config.toml` 或项目级 `.codex/config.toml`(需用户 trust);env 引用是一等公民(`bearer_token_env_var`)
- opencode:项目根 `opencode.json` 的 `mcp` 段;headers 支持 `{env:VAR}` 替换
- 三家都不支持会话内热挂载 MCP(配置在会话启动时加载)

## B:稳定接入点 + PTY 环境注入(基础设施)

1. **稳定端口**:`AppConfig` 新增 `mcp_port: u16`(默认 39127)。server 启动时绑
   `127.0.0.1:mcp_port`;`AddrInUse`(多实例等)→ 回退 `:0` 随机端口并 eprintln。
   本轮不做设置 UI(改 settings 文件生效)。
2. **PTY 环境注入**:`pty_create` 向会话注入 `THTK_MCP_URL` 与 `THTK_MCP_TOKEN`
   (从 `AppState.mcp_server` 读;server 未启动则不注入)。
   `PtyManager::create` 增加 `envs: Vec<(String, String)>` 参数保持 Tauri 解耦。

## A:配置全托管写入(打开项目时,检测到 CLI 才写)

`set_project_root` 在写 `.mcp.json`(现状保留,token 字面量)之后:

3. **opencode**:PATH 检测到 `opencode` 时,非破坏 upsert 项目根 `opencode.json`:
   ```json
   { "mcp": { "thtk-studio": { "type": "remote", "enabled": true,
       "url": "http://127.0.0.1:{port}/mcp",
       "headers": { "Authorization": "Bearer {env:THTK_MCP_TOKEN}" } } } }
   ```
   token 走 env 引用(opencode 从我们的 PTY 启动天然带 env)→ 端口不变则写一次即永久有效。
4. **codex**:PATH 检测到 `codex` 时,用 `toml_edit`(保格式)upsert 项目级
   `.codex/config.toml`:
   ```toml
   [mcp_servers.thtk-studio]
   url = "http://127.0.0.1:{port}/mcp"
   bearer_token_env_var = "THTK_MCP_TOKEN"
   ```
   首次写入后经 `mcp://report` 事件向输出面板发卡片:
   "已写入 codex 项目配置,首次使用需在 codex 中信任本目录"。
5. 与 `.mcp.json` 同规则:文件非法(JSON/TOML 解析失败)→ 报错不覆盖;
   只动 `thtk-studio` 一个 entry;写入失败仅 eprintln,不影响打开项目。

## C:SKILL.md 自助注册兜底

6. `ecl-skill-template.md` 新增一节"接入 thtk-studio MCP":若所用工具未配置
   thtk-studio,从环境变量 `THTK_MCP_URL`/`THTK_MCP_TOKEN`(或项目根 `.mcp.json`)
   获取接入点;codex 用 `codex mcp add thtk-studio --url $THTK_MCP_URL
   --bearer-token-env-var THTK_MCP_TOKEN`;opencode 编辑 `opencode.json` 后提示
   用户重启;改动需要新会话才生效。

## 不做

- MCP server 按项目派生端口/运行时换绑(server 全局一个,稳定端口已够)
- 设置 UI、OAuth、会话内热挂载、写全局 `~/.codex/config.toml`

## 测试

- 端口:占用 39127 后启动 → 回退随机且 server 可用(集成测试改造现有 auth 测试套路)
- PTY env:unix 会话内 `echo $THTK_MCP_TOKEN` 回读(扩展现有 echo 测试)
- opencode/codex upsert:各 3 个测试(新建/保留他人内容/拒绝覆盖非法文件)
- CLI 检测:PATH 中不存在时不产生文件
- 模板:占位符替换后无残留

## 验收

1. 内嵌终端 `echo $THTK_MCP_URL $THTK_MCP_TOKEN` 有值
2. 装了 opencode 的机器:打开项目生成 `opencode.json`;opencode 启动即见六工具
3. 装了 codex 的机器:`.codex/config.toml` 生成 + trust 提示卡片;trust 后 codex 见六工具
4. 没装对应 CLI:不产生对应文件
5. 重启 IDE(端口不变):opencode/codex 配置无 diff;`.mcp.json` 仅 token 变
