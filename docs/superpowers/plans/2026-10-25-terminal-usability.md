# 终端日常操作完善实施计划

> **For agentic workers:** 实施时使用 `superpowers:executing-plans` 或已明确选择的执行方式；项目 AGENTS.md 的范围、授权与测试约定优先。以下步骤是计划，尚未实施，不默认要求委派或逐任务提交。

**Goal:** 保留 xterm.js + Rust portable-pty，以官方插件和现有 API 补齐高频终端操作，减少自研代码。

**Architecture:** Rust 继续负责 PTY、进程和输入输出；`sessionRuntime.ts` 继续持有各会话的 xterm 与插件。Vue 面板负责查找条、菜单和重命名，Pinia 仅管理现有会话信息；本轮不改变 Rust command 或事件格式。

**Tech Stack:** Tauri v2、Vue 3、TypeScript、Pinia、Naive UI、`@xterm/xterm` 6.0.0、`@xterm/addon-fit` 0.11.0、Rust portable-pty 0.9。

**Spec:** 本文件第 1–4 节记录本轮需求、调研依据和范围决策；第 5 节为实施步骤。

**状态：** 用户已确认首轮范围并授权实施；代码已实现，自动检查通过，Windows 桌面交互尚未验收。

## Global Constraints

- 技术路线保持 xterm.js + Rust PTY；不引入另一套终端内核、Node sidecar 或终端框架。
- 不自研终端模拟、搜索或网址识别算法，不访问 xterm 私有 `_core` 等字段。
- 常规修改沿用现有模块边界；新增重要依赖和 PTY 接口变更须先确认。
- 不增加请求队列、自动重试、会话恢复状态机或额外数据库。
- 默认验证已有检查；永久测试只覆盖本项目的新行为和具体回归，不测试第三方库内部算法。
- UI 验证最多截图，不运行浏览器自动点击、输入或交互流程。
- 用户手动桌面验收与自动检查分开报告，后者不能代替 Windows 体验验收。

## Review Focus

- 终端焦点下 Ctrl+C 的“复制选区 / 中断进程”分工，不能把 Ctrl+R、Ctrl+W 等 shell 输入变成工作台操作；任务 1 覆盖纯分类逻辑。
- 查找时切换会话：不能在旧终端上继续查找或留下查找条的旧结果；任务 2 定义关闭查找条的行为，桌面手动确认。
- 退出后的回滚仍可复制和搜索，但不能向死会话继续粘贴；任务 1 定义运行时行为，任务 5 手动确认。
- 剪贴板权限拒绝：明确报错并保留系统原生粘贴入口，不能通过自动重试或新增 native 插件掩盖；任务 1 定义提示，任务 5 手动确认。
- 运行中的 CLI/TUI：隐藏面板、清屏和重命名不终止进程，不替换 PTY；任务 4 覆盖真实 store 状态，任务 5 手动确认。

## 1. 当前代码基线

- `src/services/terminal/sessionRuntime.ts`：模块级 xterm 容器；目前只加载 FitAddon；5000 行回滚、13px 字体、输入转发、输出和退出监听、resize、销毁。
- `src/stores/terminal.ts`：多会话、当前会话、新建计数、shell 失败回退和退出状态。会话快照为空；新会话从当前项目根启动。
- `src/components/ToolWindow/TerminalPanel.vue`：会话标签、关闭、新建和 shell 菜单；显示时 focus/fit，ResizeObserver 同步尺寸。
- `src/composables/useWorkbenchShortcuts.ts`：工作台快捷键。目前没有显式终端焦点分工；但 xterm 通常会取消自己处理的按键事件，不能仅凭全局监听断言具体快捷键已冲突。
- `src-tauri/src/common/pty.rs`：portable-pty，输出批处理和 Tauri 事件；输出批次目前由 `String::from_utf8_lossy` 解码。保留实现，本轮不顺带重构。
- 已有 opener 插件及 `opener:default` 权限，普通网页链接无需新增 Tauri 插件或放宽文件权限。

## 2. 调研结论与依赖选择

| 能力 | 采用方式 | 首轮处理 |
| --- | --- | --- |
| 搜索缓冲区 | 官方 `@xterm/addon-search` | 建议新增 0.16.0，首轮只接普通文本、区分大小写、前后查找和高亮 |
| 普通网址识别 | 官方 `@xterm/addon-web-links` | 建议新增 0.12.0，识别规则沿用官方默认 |
| 复制、粘贴、全选、清屏、滚到底部 | xterm 公共 API + 浏览器 Clipboard API | 不新增 npm/Rust 依赖 |
| 菜单、查找条、重命名 | 已有 Vue + Naive UI | 不新增 UI 库 |
| OSC 52 剪贴板协议 | `addon-clipboard` 实现的是此协议 | 首轮不需要，不因普通复制粘贴按钮安装它 |
| GPU 渲染、图片、Unicode 扩展 | 官方其他 addons 可选 | 暂不接入，尚无实际性能或兼容问题支持扩展 |

0.16.0 / 0.12.0 是上游 **6.0.0 发布标签**中的对应版本，不把 master、最新版或 prerelease 自动视为兼容。实施前确认 npm 可获取这两个版本；建议使用精确版本并提交 package-lock，保持核心 6.0.0 不升级。若版本无法取得，先报告差异，不临时选择其他主版本。

这两个插件为 MIT，属于本次待确认的新增依赖；本文件不等于已经安装或验收。实现时预计命令：

```bash
npm install --save-exact @xterm/addon-search@0.16.0 @xterm/addon-web-links@0.12.0
```

来源（调研日期 2026-10-25）：

- [SearchAddon 版本](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-search/package.json)、[类型和接口](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-search/typings/addon-search.d.ts)。
- [WebLinksAddon 版本](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-web-links/package.json)、[链接激活回调](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-web-links/typings/addon-web-links.d.ts)。
- [ClipboardAddon 源码](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-clipboard/src/ClipboardAddon.ts)：注册 OSC 52 handler，不提供终端复制粘贴菜单。
- [xterm Terminal API](https://xtermjs.org/docs/api/terminal/classes/terminal/)；实现时以本地 6.0.0 类型为准。
- [xterm 编码说明](https://xtermjs.org/docs/guides/encoding/)：Uint8Array 输入按 UTF-8 流解码，onBinary 是另一类输入事件。
- [Tauri opener](https://v2.tauri.app/plugin/opener/)；本地权限清单已包含默认 URL 权限。

## 3. 首轮产品行为

### 3.1 操作入口和快捷键

保留现有会话条，在右侧加入“查找”和“更多操作”，不再增加一整行常驻工具栏。更多菜单包含复制、粘贴、全选、清屏、滚到底部、重命名和关闭；顶部“终端”菜单复用相同动作入口。顶部菜单操作前先显示终端面板，等待 Vue nextTick 挂载完成再派发动作；没有活动会话时相应操作置灰，不因复制或查找隐式启动 shell。

- Windows/Linux：终端有选区时 Ctrl+C 复制；无选区时交给 xterm，保持 Ctrl+C 中断。Ctrl+Shift+C 明确复制；Ctrl+Shift+V 使用显式粘贴动作。
- macOS：Cmd+C 复制选区。普通 Cmd+V 沿用浏览器/xterm 原生粘贴事件；Windows/Linux 普通 Ctrl+V 同样保留原生路径。
- Ctrl/Cmd+F 打开终端查找条。只有查找输入框中的 Enter / Shift+Enter 用于前后查找，Escape 关闭并把焦点还给终端。
- 终端内 Ctrl/Cmd+反引号切换面板。Ctrl+R、Ctrl+W、Ctrl+O、Ctrl+K 等其余输入继续交给 xterm/shell；不把编辑器快捷键套进终端。
- 工作台全局处理器显式识别 `.xterm` 内目标后退出；上述需要工作台处理的动作由 xterm 的 `attachCustomKeyEventHandler` 转发。
- 自定义 handler 消费按键时须主动 `preventDefault`、`stopPropagation` 并返回 false；仅返回 false 不代表浏览器事件已被取消。未处理按键返回 true。

复制调用 `getSelection()` 后 `navigator.clipboard.writeText()`；无选区时不改剪贴板。菜单粘贴调用 `navigator.clipboard.readText()` 后 **`term.paste(text)`**，保留 xterm 的 bracketed paste 行为，不把文本直接写入 PTY。读取被拒绝时提示使用系统原生粘贴快捷键；首轮不新增 native 剪贴板插件。

清屏调用 `term.clear()`，其语义沿用 xterm：清理回滚/显示、保留当前行；不清除 shell 命令历史。选区清空、全选和滚动使用公共 API。退出会话仍保留可见输出，粘贴禁用，查找和复制可用。

### 3.2 搜索

每个 runtime 持有自己的 SearchAddon；Vue 只持有当前查找条的 query、caseSensitive 和 found 状态，不把 xterm 对象放入 Pinia。

- 查找条默认关闭；显示时占用一行，收起后归还高度并 fit 终端。
- 默认普通文本、不区分大小写。提供区分大小写、上一项、下一项和关闭。
- 使用 `findNext` / `findPrevious` 和官方 decorations，不扫描日志字符串、不保存第二份终端输出。
- 空查询清除 decorations、不显示“无匹配”；非空查询失败显示“无匹配”。首轮不做总数展示，避免混淆插件的高亮上限和完整匹配数。
- 切换、关闭会话或隐藏终端时收起查找条；只有本次存在搜索时才清理旧会话 decorations，避免误清除普通文本选区。查询文本同步清空，不维护每会话搜索快照。
- 搜索范围是当前 xterm 保留的缓冲区，不承诺恢复超过 5000 行的历史。保留 native TUI alternate buffer 规则。

### 3.3 网址链接

用默认 WebLinksAddon 识别 HTTP(S)。提供自定义 handler，经 `URL` 检查协议后，Ctrl+点击（macOS 为 Cmd+点击）调用现有 `openUrl` 在系统浏览器打开。显示简短的修饰键提示；打开失败走现有消息提示，不重试。

不使用插件默认 `window.open()` 创建应用内新窗口，不扩展到本地文件、任意协议或自定义路径正则。OSC 8 等现有协议行为不在本轮统一改造。

### 3.4 会话重命名

仅修改已有 `TerminalSessionInfo.title`，trim 后非空才生效；取消或空值保留原名，不改变 sessionId、活动会话和退出状态。使用 Naive UI 的输入/对话框即可。

本轮标题只在窗口会话内有效。不增加 customTitle、标题持久化或自动标题覆盖规则，也不通过重建 PTY 实现改名。

## 4. 文件与模块职责

| 文件 | 计划修改 |
| --- | --- |
| `package.json`、`package-lock.json` | 两个官方插件，保持现有核心及 Rust 依赖 |
| `src/services/terminal/sessionRuntime.ts` | xterm/plugins 生命周期、公共操作、搜索、键盘入口和错误传出 |
| `src/composables/useTerminalActionBridge.ts`（新增） | 与编辑器同样的 window CustomEvent 动作桥；纯快捷键分类；不访问 Pinia、PTY 或剪贴板 |
| `src/components/ToolWindow/TerminalPanel.vue` | 操作菜单、动作接收、查找条显示和消息提示 |
| `src/components/ToolWindow/TerminalFindBar.vue`（新增） | 独立查找 UI，只接收值并发出查询/前后/关闭事件 |
| `src/stores/terminal.ts` | 新增简单重命名动作，不存搜索缓存或 xterm 对象 |
| `src/components/Common/MenuBar.vue` | 顶部终端菜单转发到当前会话 |
| `src/composables/useWorkbenchShortcuts.ts` | 终端目标排除，避免工作台快捷键再次处理 |
| `tests/unit/terminalActions.spec.ts`（新增） | 本项目快捷键分工，调用真实生产分类函数 |
| `tests/unit/stores.spec.ts` | 在现有 store 测试中补重命名行为，避免新增独立测试工程 |

`useContextMenu.ts` 是文件树专用模块，不复用它，也不为这次更新抽取全局菜单框架；只复用 Naive UI 和现有事件桥的实现方式。

## 5. 实施任务

### Task 1：终端操作和焦点分工（不需要新依赖）

**接口：**

- bridge：`TerminalAction = 'copy' | 'paste' | 'selectAll' | 'clear' | 'scrollBottom' | 'find' | 'togglePanel'`。
- bridge：`dispatchTerminalAction(sessionId: number, action: TerminalAction): void`，发出 `thtk:terminal-action`，detail 为 `{ sessionId, action }`。
- bridge：`resolveTerminalShortcut(event: KeyboardEvent, platform: 'windows' | 'linux' | 'mac', hasSelection: boolean): TerminalAction | null`。
- runtime：`runTerminalAction(id: number, action: 'copy' | 'paste' | 'selectAll' | 'clear' | 'scrollBottom'): Promise<void>`；不可用/已退出会话的粘贴给出可读错误，菜单事先禁用。

步骤：

- [x] 在 `terminalActions.spec.ts` 覆盖 Ctrl+C 有/无选区、Ctrl+Shift+C/V、Cmd+C、Ctrl/Cmd+F、切换面板；Ctrl+R/W/O/K 及普通 Ctrl+V/Cmd+V 返回 null。不调用浏览器或模拟整个终端。
- [x] 运行 `npm test -- --run tests/unit/terminalActions.spec.ts`，确认尚无分类实现时失败；随后实现生产分类函数和 bridge，检查相应用例通过。
- [x] 在 runtime 注册 custom handler，转发带 sessionId 的动作；保留未处理按键和原生 paste 事件。为正常复制/粘贴接入公共 API，错误由调用方展示。
- [x] 在 TerminalPanel 订阅动作并在卸载时解除；按事件中的 sessionId 执行，不能把旧事件重定向到另一个活动会话。`find` / `togglePanel` 属于 UI 动作，runtime 不导入 store。
- [x] 增加终端操作菜单和顶部菜单入口；暂未接搜索时不显示空入口。隐藏或退出会话时的禁用状态符合第 3 节。
- [x] 修改全局快捷键的终端目标排除，保留终端以外的刷新保护和编辑器行为。

### Task 2：官方搜索插件和查找条

**接口：** runtime 新增 `findInSession(id: number, query: string, direction: 'next' | 'previous', caseSensitive: boolean): boolean` 和 `clearSessionSearch(id: number): void`；缺失会话安全返回 false/无操作，用于关闭时清理。

步骤：

- [x] 用户确认本计划中的新增依赖后，安装固定版 SearchAddon，更新锁文件；不升级 xterm 核心。
- [x] 每个 SessionRuntime 增加 SearchAddon，随 Terminal 一起释放；加载错误沿用现有创建失败清理路径。
- [x] 接入 `TerminalFindBar.vue` 和 Panel 局部状态，依据第 3.2 节处理按钮、Enter、Escape、大小写、空查询、切换和隐藏。
- [x] 复用插件搜索和高亮接口，首轮不做正则、全文备份、结果计数或自研匹配层。
- [x] 运行已有前端测试和类型检查；不编写只断言假 SearchAddon 调用次数的永久测试。真实搜索效果列入桌面手动验收。

### Task 3：官方网址插件

**接口：** 仍使用现有 `openUrl(url)`；无 Rust 接口变更。

步骤：

- [x] 用户确认新增依赖后安装固定版 WebLinksAddon，更新锁文件。
- [x] 每个 runtime 加载插件，handler 只响应约定的修饰键点击，校验 HTTP(S) 后调用现有 opener。协议判断与错误提示保持直接，不引入链接服务框架。
- [x] 使用默认识别规则；不为少数日志格式另写 URL/文件解析器。
- [x] 检查新插件随会话释放；验证未扩大 `capabilities/default.json` 的权限。

### Task 4：会话重命名

**接口：** store 新增 `renameSession(id: number, title: string): void`。

步骤：

- [x] 在现有 `stores.spec.ts` 以 mock 隔离 sessionRuntime 的 PTY/DOM 边界，设置两条会话，调用真实 store 动作，断言只改变目标 title，保留 id、activeSessionId、exited；空白标题不改变原名。
- [x] 运行该文件确认新增动作未实现时失败，然后实现 trim/非空检查与标题赋值。
- [x] Panel 的“重命名”菜单弹出带原名的输入框；确认调用 store，取消不改变状态。
- [x] 不添加重启、自动标题、shell/cwd 快照或持久化逻辑。

### Task 5：集中验证和记录

- [x] 完整改动集中运行 `npm test`、`npm run typecheck`、`npm run build`、`git diff --check`；局部失败修正后优先重跑受影响检查。
- [x] 首轮无 Rust 变更，不为形式完整反复运行 Rust 全量测试，也不新增 CI/E2E 工作流。
- [x] 若当前环境无法启动 Windows 桌面，明确记录未验收项；自动化只做相关单测和静态检查，最多截图，不自动跑交互流程。
- [x] 收尾时记录实际插件版本、验证命令、结果和未验证项；代码提交/推送按用户当时的授权范围执行。

供用户手动验收的精简清单（不是本次调研已执行的操作）：

1. PowerShell/cmd 中有选区 Ctrl+C 复制，无选区 Ctrl+C 中断；Ctrl+R/W 等行为不被工作台接管。
2. 普通系统粘贴与菜单粘贴都能输入；权限拒绝有明确提示；多行粘贴沿用 xterm/shell 的 bracketed paste。
3. 输出中查找文字，前后跳转、大小写、无匹配和关闭正常；切换会话后没有旧结果。
4. Ctrl/Cmd+点击 HTTP(S) 用系统浏览器打开，普通选择文本不打开链接。
5. 运行一个实际常用 CLI/TUI 后隐藏/恢复面板，复制、滚动和退出状态正常；重命名不重启进程。

## 6. 暂缓与独立决策

- **输出跨批次 UTF-8 问题：** 代码中确实有逐批 lossy 解码，尚未收到实际乱码复现。本轮不因理论风险改 PTY 协议；如确认要修，优先评估 `Vec<u8>` 事件到 `Uint8Array`，交给 xterm 流式解码。它是数据格式变更，需要另行确认和对应的拆分多字节回归，不能顺带加队列、重试或恢复机制。
- **重启终端：** 会终止正在运行的 shell/agent，而且当前 store 没保存实际 shell/cwd。先确认是否需要、重启是否沿用创建目录，不以重命名功能顺带加入。
- **分屏：** 需要新的布局状态和焦点行为，后续单独设计；先保留多标签。
- **shell profiles / 字体设置：** 会增加配置字段及持久化，后续按真实需求设计。
- **命令边界、当前目录、相对文件链接：** 依赖 shell integration，后续独立推进，不推测 shell 状态。
- **持久会话、刷新重连、远程、任务系统、OSC 52、图片协议：** 本轮不做。

## 7. 计划自查

- 首轮只改终端前端及两个 npm 依赖；没有 Rust/API/存储模型迁移。
- 快捷键桥只传动作与目标会话，不传终端实例；UI 局部状态不进入 PTY 生命周期。
- 自动搜索、网址识别和 paste 交给现成公共 API；剪贴板插件没有误作菜单组件。
- 不把搜索切换、权限失败、TUI 实际表现伪装成单元测试已验收；明确留给精简手动检查。
- 重启、分屏与传输修正没有混入首轮，后续决策点有具体理由。


文档验证：`git diff --check` 通过；来源链接均按本轮实际读取资料列出。未修改或运行终端实现，未安装依赖；未运行单元测试、构建或桌面交互，以上验证命令与手动清单均为后续实施要求。


## 8. 首轮实施记录

- 安装精确版本 `@xterm/addon-search` 0.16.0、`@xterm/addon-web-links` 0.12.0；锁文件中的 xterm 核心仍为 6.0.0。
- 实现操作菜单、终端查找条及官方高亮、Ctrl/Cmd+点击网址、会话重命名和快捷键分工。
- `SearchAddon` 的高亮需要公共 decoration API；按其要求开启 `allowProposedApi`，应用代码未访问 `_core` 等私有字段。
- 额外的 `thtk:terminal-error` 消息事件仅用于 runtime 向现有面板消息提示传递打开链接错误；未引入独立通知系统。
- 自动检查：`npm test` 175 passed；`npm run typecheck`、`npm run build`、`git diff --check` 通过。
- 测试只覆盖本项目快捷键分类和真实 store 的重命名；不以 mock 证明第三方插件的搜索/剪贴板效果。
- 只读代码审阅未发现高、中优先级正常流程问题。
- 测试仍出现 Monaco 随包 source map 缺失警告；构建仍有大 chunk 提示，均未作为本轮扩展工作处理。
- 未修改 Rust、Tauri 权限或 PTY 数据格式，未运行 Rust 全量测试。
- 未运行 Windows 桌面交互或浏览器自动交互；第 5 节手动清单仍未验收。实现检查完成不代表这份手动清单已执行。
- 代码与计划尚未提交或推送；`.qoder/` 本地配置未修改。
