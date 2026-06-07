# THTK-Studio 编辑器壳子现状

本文档只回答一件事：

当前项目作为"代码编辑器壳子"已经实现了什么，距离一个稳定、常用、像样的 IDE 外壳还缺什么。

它不替代长期规划，长期方向仍以 [project.md](./project.md) 为准。

---

## 1. 当前结论

目前项目已经具备一个基本可用、结构开始成形的桌面 IDE 工作台壳子。

**第一阶段 MVP 工程闭环已经接近完成。**

thecl 编译 → 错误诊断 → Monaco 波浪线 → 问题面板 → 点击跳转的完整链路已经打通。文件树已具备懒加载和展开状态持久化。外部文件变更检测已实现。

接下来应优先扩展 msg/std/anm 工具链，进入第二阶段。

---

## 2. 已实现的常用编辑器功能

### 2.1 工作区与文件管理

已实现：

- 打开工作区文件夹
- 文件树懒加载（只加载当前层级，展开时按需加载子目录）
- 刷新文件树（保留已展开目录的加载状态）
- 新建文件 / 文件夹
- 重命名文件或文件夹
- 删除文件或文件夹（同步关闭子文件标签）
- 复制文件路径
- 右键菜单基础操作
- 文件树多选
- 文件树内部剪切 / 复制 / 粘贴（自动处理重名）
- 系统文件剪贴板基础接入
- 文件和目录拖拽移动（含根目录拖放区域）
- 文件树展开状态持久化（按项目保存到 localStorage，切换项目自动恢复）
- 外部文件变更检测（Rust notify 监听 + 前端自动重载或提示）

### 2.2 编辑器主体

已实现：

- Monaco Editor 接入
- 多标签编辑 / 切换 / 关闭
- 脏状态跟踪 / 保存
- 当前文件内查找 / 替换
- 顶部菜单栏 / 状态栏
- VS Code 风格工作台布局

### 2.3 工作区视图体系

已实现：

- 文本文件走 Monaco 文本视图
- 真正的 `.ecl` 二进制文件进入专用"二进制脚本视图"（路径、大小、反编译入口）
- 主区域统一工作区视图宿主 + 注册表模式

### 2.4 会话与窗口行为

已实现：

- 工作区路径持久化
- 已打开标签恢复 / 当前激活标签恢复
- 脏标签草稿有限度持久化（200KB/文件，1MB 总量上限）
- 刷新后恢复工作区和标签
- 屏蔽 `F5` / `Ctrl+R` / `Ctrl+Shift+R`
- 关闭页面前未保存保护
- 草稿恢复时检测磁盘是否已变化

### 2.5 面板系统

已实现：

- 底部统一面板容器（输出 / 问题 / 终端）
- 输出面板：按任务分组，显示来源、操作和时间
- 问题面板：按严重度排序，点击跳转到源码位置（已修复时序问题）
- 统一 reports store 管理输出与问题数据

### 2.6 构建与工具链

已实现：

- `thecl` 统一请求模型（compile / decompile / header）
- 图形化构建配置弹窗（版本 / 输出路径 / thecl 选项）
- 编译结果 → Monaco 诊断标记（波浪线覆盖整个单词或行尾）
- 诊断路径自动规范化（Rust 端将相对路径解析为绝对路径）
- 构建后刷新文件树 / 自动打开产物
- 工具链状态检测（5 个工具的路径解析和版本查询）

### 2.7 ECL 语言支持

已实现：

- Monarch 语法高亮
- eclmap 驱动的语义数据（指令名、参数、枚举）
- 补全 / 悬停 / 转到定义 / 引用查找 / 签名帮助
- 文档符号（大纲 / 面包屑）
- 静态诊断（未定义跳转 / 重复定义等）
- 工具链诊断（thecl 编译错误 → Monaco markers）

### 2.8 Agent 通道

已实现进程内 MCP server，支持在终端内运行 claude code 并让 agent 直接操作工作区：

- **MCP server**：`src-tauri/src/modules/mcp/` — 基于 rmcp 1.7，Streamable HTTP，绑定 127.0.0.1 随机端口，Bearer token 每次启动轮换；阻塞工作走 `spawn_blocking`
- **六个工具**：`get_workspace_info`、`check_ecl`、`compile_ecl`、`decompile_ecl`、`lookup_ecl_semantics`、`report_to_user`
- **`.mcp.json` 自动接线**：打开项目时 Rust 端非破坏性合并 `.mcp.json`（`common/mcp_config.rs`），已有其他 MCP server 条目不被覆盖；token 每次启动更新
- **report_to_user 卡片**：agent 调用 `report_to_user` → 前端 `composables/useMcpBridge.js` 接收 → 输出面板弹出结构化卡片
- **AI 辅助包生成**：菜单"生成 AI 辅助包"产出 `.claude/skills/ecl-modding/`（SKILL.md 仅首次生成，重跑不覆盖；`references/` 从当前 eclmap 重新生成）
- **全局寄存器支持**：`map_parser` 新增 `!gvar_names`/`!gvar_types` 解析，为 `lookup_ecl_semantics` 提供完整词汇

注意：`.mcp.json` 含会话 Bearer token，用户若将其提交到 git 会短暂泄漏（token 每次启动轮换）；建议项目 `.gitignore` 加入 `.mcp.json`。

---

## 3. 已实现但需要明确边界的能力

### 3.1 真 PTY 终端（已实现）

底部终端面板现已接入完整的 PTY 终端，具备以下能力：

- **跨平台 PTY**：Rust 端使用 `portable-pty` crate，Windows 走 ConPTY，Linux/macOS 走系统 PTY；shell 自动探测（`$SHELL` → 系统默认）
- **xterm.js 前端**：`TerminalPanel.vue` 内嵌 xterm.js，支持多 tab、每 tab 独立会话
- **模块级会话运行时**：`src/services/terminal/sessionRuntime.js` 持有 xterm 实例，组件卸载后会话不丢失，切回 tab 内容完整恢复
- **输出合批**：Rust 端 16ms 窗口合批，减少前端渲染压力；chunk 按 lossy UTF-8 解码（多字节字符跨 chunk 时可能出现替换字符，已标注，出现问题再改累积解码）
- **scrollback 上限**：xterm.js scrollback 设为 5000 行，防止内存无限增长
- **退出检测**：ConPTY 安全的 waiter-thread 退出检测；shell 退出后 tab 显示退出码并变灰，无崩溃

旧的一次性命令面板（`src/api/modules/terminal.js`、`src-tauri/src/common/terminal.rs` 的 `run_shell_command`）已无消费方，待后续清理。

---

## 4. 还缺哪些基础功能

### 4.1 高优先级

- ~~Monaco diagnostics 标记~~ ✅ 已完成
- ~~工具链错误波浪线~~ ✅ 已完成
- ~~问题面板点击跳转~~ ✅ 已完成
- ~~外部文件变更检测~~ ✅ 已完成
- ~~文件树展开状态持久化~~ ✅ 已完成
- ~~文件树懒加载~~ ✅ 已完成
- 最近项目 / 欢迎页流程闭环
- `msg / std / anm` 的第一版工作区视图
- 项目配置文件格式（`.thtk-project.json`）

### 4.2 中优先级

- 更完整的标签关闭行为（关闭其他 / 关闭右侧）
- 资源管理器补齐（在系统中显示 / 复制相对路径 / 批量重命名）
- 右侧边栏真实面板（大纲、文件信息）
- Rust 错误处理改善（unwrap → 安全错误处理，添加 tracing 日志）
- 终端/输出面板内存限制（终端侧已解决：scrollback 上限 5000 行；输出面板仍无上限，长时间使用可能增长）

### 4.3 后续但不应过早开始

- ~~真正 PTY 终端~~ ✅ 已完成
- ~~AI 集成（agent 通道）~~ ✅ 已完成
- 语言服务深度扩展
- 符号索引
- ANM / MSG / STD / ECL 领域预览

---

## 5. 当前壳子的优势

- 工作台结构已经成型，不需要推翻重来
- 资源管理器已具备真正可用的基础操作 + 懒加载 + 持久化
- thecl 工作流已完整闭环（编辑 → 编译 → 诊断 → 波浪线 → 问题面板 → 跳转）
- 工作区视图体系已经开了正确的口
- 文件变更检测已到位，编辑体验更可靠
- 代码拆分合理（FileTree 已拆为 3 个 composable，职责清晰）

---

## 6. 当前壳子的风险点

- `msg / std / anm` 还没有进入统一的工作区视图体系
- 前端仍主要是 JavaScript，规划目标是 TypeScript
- Rust 端存在 8 处 unwrap/expect 调用，有 panic 风险
- 输出面板无内存上限，长时间使用可能内存增长（终端侧已通过 scrollback 上限解决）
- 路径比较逻辑在 Windows 上工作正常，跨平台兼容需额外处理
- `.mcp.json` 含会话 Bearer token，用户若误提交到 git 会短暂泄漏；建议加入项目 `.gitignore`
- PTY 输出按 chunk lossy UTF-8 解码，多字节字符跨 chunk 时可能出现替换字符（已注释，出现问题再改累积解码）
- `report_to_user` 的 warning 级别在输出面板被扁平化为 info（publishToolResult API 限制）
- eclmap 切换/重载后"提示重新生成 AI 辅助包"尚未实现（本期裁剪，references 需手动重跑菜单命令刷新）
- 旧一次性命令面板代码（`src/api/modules/terminal.js`、`src-tauri/src/common/terminal.rs`）已无消费方，待清理

---

## 7. 建议的下一步顺序

1. ~~把 `thecl` 结果接入 Monaco diagnostics~~ ✅
2. ~~补稳问题跳转~~ ✅
3. ~~做文件树展开状态持久化~~ ✅
4. ~~做外部文件变更检测~~ ✅
5. ~~文件树懒加载~~ ✅
6. ~~FileTree.vue 代码拆分~~ ✅
7. 给 `msg / std / anm` 接第一版工作区视图
8. 项目配置文件格式
9. 欢迎页 / 最近项目
10. ~~真正 PTY 终端~~ ✅ 已完成（portable-pty + xterm.js 多 tab）
11. ~~进程内 MCP server + agent 通道~~ ✅ 已完成（rmcp，六工具，.mcp.json 自动接线）

---

## 8. 总结

THTK-Studio 的第一阶段 MVP 工程闭环已经完成，并在此基础上完成了 Agent 通道（第二阶段重点）。

已完成的核心闭环：
- 编辑 → 编译 → 诊断 → Monaco 波浪线 → 问题面板 → 点击跳转
- 文件管理 → 懒加载 → 展开持久化 → 外部变更检测
- 会话恢复 → 草稿保护 → 脏状态跟踪
- 真 PTY 终端 → xterm.js 多 tab → 模块级会话运行时 → scrollback 上限
- 进程内 MCP server → 六工具 → .mcp.json 自动接线 → report_to_user 卡片 → AI 辅助包生成

下一阶段重点应从"稳定 agent 通道"转移到"扩展更多工具链（thmsg/thstd/thanm）"和"完善项目配置体系"。

---

## 9. 手动验收清单（待用户在带桌面的机器执行）

1. `npm run tauri dev` 启动，打开一个项目文件夹
2. 底部终端面板：新建终端 → 跑 `vim` / `python` REPL — 交互正常、resize 正常、多 tab 切换不丢内容
3. 项目根出现 `.mcp.json`，含 `thtk-studio` entry（端口 + Bearer token），已有自定义 entry 不被破坏
4. 终端里跑 `claude` → `/mcp` 列出 thtk-studio 六个工具
5. 让 agent 调 `check_ecl` 编译一个 `.decl` — 诊断与问题面板一致
6. 让 agent 调 `report_to_user` — 输出面板弹出卡片
7. 菜单"生成 AI 辅助包" — `.claude/skills/ecl-modding/` 生成；改 `SKILL.md` 后重跑不被覆盖，`references/` 刷新
8. 输入 `exit` 关闭 shell — 终端显示退出码，tab 变灰，无报错
