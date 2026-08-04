# 第一阶段 MVP 项目工作流收尾计划

**日期：** 2026-07-13  
**目标：** 让用户从启动应用、选择最近项目、打开或切换工作区，到编辑项目配置形成完整且可恢复的项目级工作流。  
**范围：** 项目配置 UI、最近项目、欢迎页、项目切换保护、相关测试与文档。  
**不在本轮：** ANM 工具链、MSG / STD 语言服务、全局索引、大规模 TypeScript 迁移、通用 UI 重做。

## 实施状态（2026-08-03 更新）

| 任务 | 状态 | 说明 |
| --- | --- | --- |
| Task 1 收紧后端契约 | ✅ 完成 | 三态配置 + 原子保存 + 事务式 `open_project` |
| Task 2 最近项目存储 | ✅ 完成 | 见下方偏差说明 |
| Task 3 统一打开 / 切换动作 | ✅ 完成 | |
| Task 4 欢迎页与最近项目视图 | ✅ 完成 | |
| Task 5 项目配置对话框 | ✅ 完成 | |
| Task 6 测试门禁与容量修复 | 基本完成 | vitest 45 个测试 + `typecheck` 门禁 + 容量上限；**未做** Playwright smoke，见下 |
| Task 7 文档与桌面验收 | ✅ 完成 | 文档已同步；Windows 手动验收于 2026-08-03 执行通过，见下方记录 |

与计划的偏差：

- **新增前端边界用 `.js` 而非 `.ts`。** 仓库当前没有 `typescript` 依赖、没有 `tsconfig.json`、也没有 typecheck 命令（这些属于 Task 6）。孤立的 `.ts` 只会被 esbuild 转译、永不做类型检查，反而不如与其余 store 保持一致。等 Task 6 建起门禁后统一迁移。
- **额外做了项目级 `thtk` 目录覆盖的后端接线。** 计划要求配置表单包含该字段，但 `toolchain.thtkDir` 此前是死数据（无人读取），直接加输入框会是假功能。已新增 `toolchain::effective_config` 并接入 ECL / MSG / STD / DAT / MCP、`get_toolchain_status(es)` 与 `generate_ai_assist_pack`。
- **没有暴露 `record_recent_project` 命令。** 计划列了四个命令，但记录动作由 `open_project` 成功后在 Rust 侧完成，再对前端开一个入口就是没有调用方的死代码。实际提供 `list` / `remove` / `clear` 三个。
- **额外拆出 `WorkbenchRoot.vue`。** naive-ui 的 `useMessage` / `useDialog` 只能在 provider 后代中调用，而 `App.vue` 自己的 setup 在它自己的 provider 外面——这挡住了让 Ctrl+O 走统一打开流程。工作台整体下沉一层，`App.vue` 只留 provider。
- **额外修了 `save_settings` 的整体替换问题。** 它用完整 `AppConfig` 做载荷，而设置表单只填其中一部分，缺失字段取 serde 默认值；不修的话保存一次工具链设置就会清空刚加的 `recent_projects`。
- **未做 Playwright smoke（Task 6）。** 该流程要驱动真实 Tauri 窗口（需 tauri-driver + WebDriver），而开发机没有 DISPLAY 也没有 xvfb，写出来无法运行也无法验证。组件渲染层同理：要么依赖 naive-ui 的完整 provider 链，要么依赖真实 Tauri。这两块统一交给下方的 Windows 手动验收清单。vitest 的覆盖范围因此限定为 store / composable / service 这层领域逻辑。
- **测试迁移到 TS 的范围仅限新增边界。** `tsconfig.json` 只 `include` `.ts` 文件，`allowJs` 打开但 `checkJs` 关闭。既有的 20 个 `.vue` 和 58 个 `.js` 不在门禁内——全量迁移是独立的一批工作。

## 一轮多 agent 审阅（2026-07-28）

Task 1–5 完成后做过一次四路并行代码审阅（Rust / 前端状态 / UI / 对抗性回归），发现并已修复四个 Critical：会话恢复失败销毁未保存草稿、serde 忽略未知键导致拼错的配置键被静默丢弃、项目设置对话框可写入已切换的另一个项目、欢迎页裸 `<button>` 因仓库无 CSS reset 渲染成系统原生外观。细节见 `git log`。

仍未处理的已知问题：`open_project` 是同步命令会阻塞主线程；工具链每次调用重复读两次项目配置；配置 `invalid` 时前端置空 `projectConfig` 导致 ECL 路径与 msg/std/dat 行为不一致；`uno.config..js` 文件名笔误使 UnoCSS preflight 从未生效（改名会改变全局样式，需单独一批并实机比对）。

## 现状与约束

- Rust 已定义并读写 `.thtk-project.json`，前端 `project` store 也已加载和保存该结构，但没有 UI 调用保存动作。
- 当前只持久化一个 `rootPath`。应用启动时会直接恢复它，没有最近项目列表，也无法从应用内移除失效项目。
- `projectStore.loadProject()` 会吞掉加载错误，菜单仍可能显示“打开成功”；必须先修正这个契约，再让欢迎页复用。
- 切换项目会影响标签页、草稿、终端工作目录、文件监听和 MCP 客户端配置，不能只替换文件树路径。
- 项目路径检查、目录探测和最近项目持久化属于 Rust / Tauri 层；Vue 负责展示、表单和用户确认。
- 新增的前后端边界使用明确结构，不在组件间传递临时对象。

## 数据契约

### RecentProject

Rust 向前端返回 camelCase 结构：

```text
RecentProject {
  path: String,
  name: String,
  lastOpenedAt: u64
}
```

- 最多保留 10 项，按 `lastOpenedAt` 降序。
- 路径使用平台原生绝对路径；比较时沿用项目现有的跨分隔符规范化策略。
- 记录项目时去重并移动到顶部。
- 列表读取不自动删除失效路径，前端需要显示“不可用”并允许用户移除；这样不会因临时磁盘离线丢失记录。

### ProjectConfig

继续使用现有结构：

```text
ProjectConfig {
  gameVersion: String,
  encoding: "shift-jis" | "utf-8",
  mapPaths: String[],
  toolchain: { thtkDir: String }
}
```

- 保存前由 Rust 校验 encoding、项目根状态与 map path 字符串。
- 相对 map path 以项目根为基准；UI 同时允许绝对路径。
- JSON 不存在与 JSON 损坏必须返回不同结果，损坏时不得静默当作默认配置覆盖。

### ProjectOpenResult

项目打开使用一个事务式 Tauri command，不再由前端依次“设置根目录 → 获取文件树 → 加载配置”：

```text
ProjectOpenResult {
  rootPath: String,
  files: FileNode[],
  projectConfig: {
    status: "absent" | "loaded" | "invalid",
    value: ProjectConfig | null,
    error: String | null
  }
}
```

- 目录验证与首层文件扫描成功后，后端才提交当前项目根、切换 watcher、注册 MCP 客户端并记录最近项目。
- 配置文件损坏不阻止用户查看项目，但以 `invalid` 状态返回并禁止无确认覆盖。
- 目录验证或首层扫描失败时不修改当前项目、watcher、最近项目或前端状态。

## Task 1：收紧项目加载与配置后端契约

**修改：** `src-tauri/src/common/project_config.rs`、`src-tauri/src/main.rs`、`src/api/modules/config.js`、`src/api/modules/fs.js`、`src/stores/project.js`  
**测试：** `src-tauri/src/common/project_config.rs`

- 将项目目录验证提取为可测试函数：必须存在、必须是目录、能够规范化为绝对路径。
- 新增事务式 `open_project` command，返回 `ProjectOpenResult`；现有 `set_project_root` 仅保留兼容期或在迁移完成后移除。
- 将项目配置加载结果从含糊的 `Option` 改为能区分“不存在 / 有效 / 损坏”的稳定状态。
- 为 `ProjectConfig` 增加字段校验；保存采用同目录临时文件 + rename，避免中断时留下半截 JSON。
- `projectStore.loadProject(path)` 只消费一次 `open_project` 结果并提交 `rootPath/files/projectConfig`；失败时保留原项目并抛出错误。
- 增加 Rust 测试：不存在目录、普通文件、无配置、有效配置、损坏 JSON、无效 encoding、原子保存往返。

**验收：** 打开无效目录不会改变当前工作区；损坏配置会给出明确错误且文件内容不被覆盖。

## Task 2：实现最近项目后端存储

**修改：** `src-tauri/src/config.rs`、`src-tauri/src/common/recent_projects.rs`、`src-tauri/src/common/mod.rs`、`src-tauri/src/main.rs`、`src/api/modules/config.js`  
**测试：** `src-tauri/src/common/recent_projects.rs`

- 在应用配置中增加带 serde 默认值的 `recent_projects`，保持旧 `settings.json` 兼容。
- `recent_projects` 由 Rust 专用命令管理；现有 `save_settings` 必须合并并保留该字段，避免工具链设置表单提交旧快照时清空最近项目。
- 新增命令：`list_recent_projects`、`record_recent_project`、`remove_recent_project`、`clear_recent_projects`。
- 由 `open_project` 成功提交后记录；失败打开不得污染列表。
- 去重、排序和 10 项上限全部在 Rust 侧完成。
- 保存错误必须返回前端并进入输出 / 错误提示，不能只写 `console.error`。
- 测试去重、排序、容量、旧配置缺字段兼容和移除行为。

**验收：** 重启应用后最近项目仍存在；重复打开只更新时间；第 11 个项目会淘汰最旧记录。

## Task 3：建立统一的项目打开与切换动作

**新增：** `src/composables/useProjectActions.ts`  
**修改：** `src/components/Common/MenuBar.vue`、`src/stores/editor.js`、`src/stores/project.js`、`src/composables/useWorkbenchSession.js`

- 所有入口统一调用 `openProject(path)`，菜单、欢迎页和最近项目不得各写一套加载逻辑。
- 若存在脏标签，切换前提供“保存并切换 / 放弃并切换 / 取消”；保存失败时停止切换。
- 成功切换后清理旧项目标签、引用选择和仅属于旧项目的 UI 状态，再更新终端默认工作目录。
- 当前已运行的 PTY 会话不强制终止；只影响后续新建终端，并在计划验收中明确这一行为。
- 会话自动恢复失败时进入欢迎页并保留最近项目，不循环重试失效路径。
- 为新领域边界直接使用 TypeScript；不在本任务迁移既有 store。

**验收：** 菜单、欢迎页、恢复流程具有一致错误和确认行为；取消切换不会丢标签或草稿。

## Task 4：实现欢迎页与最近项目视图

**新增：** `src/components/Welcome/WelcomeView.vue`、`src/stores/recentProjects.ts`  
**修改：** `src/App.vue`、`src/components/Editor/WorkbenchEditorHost.vue`、`src/components/Editor/EmptyEditorState.vue`

- 无工作区时主区域显示真正的欢迎视图，而不是编辑器空状态。
- 首屏提供“打开文件夹”主操作和最近项目列表；不做营销 Hero 或说明卡片堆叠。
- 最近项目显示名称、完整路径、最近打开时间、不可用状态与移除操作。
- 有工作区但没有打开标签时继续显示轻量编辑器空状态，不重复欢迎页。
- 加载状态禁用重复操作；路径和错误文本必须在窄窗口内换行或截断，不破坏布局。
- 菜单中的“打开文件夹”改为复用 `useProjectActions`。

**验收：** 1024×640 与 1440×900 下无溢出；鼠标和键盘都能打开最近项目；失效项目可辨识、可移除。

## Task 5：实现项目配置对话框

**新增：** `src/components/Dialogs/ProjectSettingsDialog.vue`、`src/stores/projectSettings.ts`  
**修改：** `src/App.vue`、`src/components/Common/MenuBar.vue`、`src/stores/project.js`

- 在“文件”菜单加入“项目设置…”，无工作区时禁用。
- 表单覆盖游戏版本、编码、map path 列表和项目级 thtk 目录覆盖。
- 游戏版本用可输入选择器，编码用二段选择控件，路径通过原生 picker 添加并支持删除 / 调整顺序。
- 第一次保存创建 `.thtk-project.json`；取消不修改 store；保存成功后立即刷新 ECL 语义与工具链有效配置。
- 后端返回损坏配置时展示修复入口，但覆盖前必须向用户显示原文件路径并二次确认。
- 组件只处理交互和表单状态，校验结果与保存错误来自稳定 API / store。

**验收：** 新建、重开、修改配置均能往返；配置改变后下一次 ECL / MSG / STD / DAT 操作使用新的项目级版本或路径。

## Task 6：补测试门禁与容量修复

**修改：** `package.json`、`src/stores/workbenchReports.js`  
**新增：** 与项目动作、最近项目 store 对应的前端单元测试；`tests/project-workflow.spec.js`

- 引入与当前 Vite/Vue 版本兼容的最小测试配置；不顺带迁移全部前端代码。
- 单元测试覆盖最近项目展示状态、项目打开失败不提交状态、脏标签切换分支和配置表单归一化。
- Playwright 覆盖欢迎页 → 打开项目 → 修改设置 → 重启恢复的 smoke 流程；Tauri 原生 dialog 无法自动化的部分通过可注入动作边界替代。
- 给输出 / 问题 store 增加明确容量上限，并测试淘汰最旧条目，处理当前已知的长期内存增长风险。
- 保持 `npm run build`，新增可独立执行的 `test` / `typecheck`（仅覆盖新增 TS 边界）命令。

**验收：** 前端测试、生产构建和 Rust 全量测试全部通过。

## Task 7：文档与桌面验收

**修改：** `README.md`、`editor-shell-status.md`、`docs/review-notes.md`、本计划

- 完成后把项目配置 UI、欢迎页和最近项目标为已完成，并记录实际测试数。
- 删除“项目配置格式未实现”“终端仍是假终端”等残留说法。
- 更新 Windows 手动验收清单，不把 Linux 单元测试等同于真实 thtk 工具链验收。

Windows 手动验收：

1. 首次启动显示欢迎页，打开文件夹后进入工作台。
2. 重启后恢复上次项目；上次路径失效时回到欢迎页且无错误循环。
3. 最近项目顺序、去重、移除、清空和失效状态正确。
4. 有未保存标签时切换项目，三个确认分支行为正确。
5. 创建项目配置后重启，游戏版本、编码、map paths 和 thtk 目录覆盖保持一致。
6. 故意破坏 `.thtk-project.json`，IDE 明确报告损坏且不会静默覆盖。
7. 项目级设置确实影响 ECL、MSG、STD、DAT 的下一次工具调用。
8. 1024×640、1440×900 下欢迎页和设置对话框无重叠或文本溢出。

## 验收总进度（截至 2026-08-04）

本仓库的手动验收分散在几处，这里汇总状态，避免重复跑或漏跑。

| 清单 | 项数 | 状态 |
| --- | --- | --- |
| 本文「Windows 手动验收」8 项 + TS 迁移回归 4 项 | 12 | ✅ 2026-08-03 走过一轮，未发现问题 |
| `editor-shell-status.md` §9 手动验收清单 | 25 | ⚠️ 已完成 9 项（第 2–9、13），**余 16 项待执行** |
| `2026-08-03-game-version-reliability.md` 的追加清单 | 7 | ❌ 待执行 |
| 技术债与首次运行批次（见下） | 6 | ❌ 待执行 |
| 面板 chrome 改版（见下） | 4 | ❌ 待执行 |
| 文件树滚动（见下） | 1 | ❌ 待执行 |

`editor-shell-status.md` 那份已完成的 9 项覆盖了终端 / PTY 与 **claude code 的
MCP 通道**（含 `check_ecl`、`report_to_user` 实际调用、AI 辅助包生成、重启后
配置无 diff）。剩下 16 项集中在两处：

- **opencode / codex 的实机接入**（第 10–12 项）。此前只逐字核对过两者的配置
  格式与官方文档一致，**没有一次实机握手**。这也是三家 client 里唯一没验的部分。
- **MSG / STD / DAT 三条工作流**（第 14–25 项，共 12 项）：解包→编辑→打包的
  往返正确性，以及写错指令名时失败卡片是否显示 stderr、原文件是否不被覆盖损坏。

### 待验收：技术债与首次运行批次（2026-08-03）

1. 全新安装（未配 thtk）启动，欢迎页出现工具链提示条，点击可直接打开工具链设置。
2. 未配 thtk 时在工作台点任意工具链菜单项，报错说明去哪配、以及 thtk 需自行下载。
3. 只配了部分工具（如缺 thmsg）时，提示条点名缺哪几个。
4. 打开一个大目录项目 / 刷新工具链设置页时，界面**仍可交互**（五个重活命令已移出主线程）。
5. 项目配置写 `encoding: "utf-8"` 后编译 `.decl`，thecl **不再收到 `-j`**。
6. 故意写坏 `.thtk-project.json` 后编译 `.decl`，输出面板先出现「项目配置未生效」卡片
   并指向配置文件路径，而不是直接报一堆 unknown instruction。

### 待验收：面板 chrome 改版（2026-08-04）

1. 底部面板只剩**一条**头栏，不再有两个「隐藏」按钮。
2. 活动 tab 是近白色下划线而非蓝框；hover 只有半透明底色、无描边。
3. 键盘 Tab 走到面板按钮上能看到蓝色焦点框（`#0078d4`）。
4. 终端无会话时，空状态里的「新建终端」按钮可点。

> 这批只在 Linux 上逐条 grep 验证了产出的 CSS 规则，**观感未经实机确认**——
> 扁平风格的成败在 hover 底色浓淡与下划线位置，截图才看得准。

### 待验收：文件树滚动（2026-08-04）

侧栏文件树在项目文件多于一屏时可用滚轮滚动；文件名过长时可横向拖动查看。

---

## Windows 手动验收执行记录（2026-08-03）

上述 8 项 + 下列 4 项 TS 迁移专属回归，由作者在 Windows 上走过一轮，**未发现问题**。

安装包由 Linux 交叉编译产出（`cargo-xwin` + `--config '{"bundle":{"targets":["nsis"]}}'`，
因为 `--bundles nsis` 会被 CLI 按宿主平台拒绝）：
`thtk-studio_0.1.0_x64-setup.exe`，5.2 MB，NSIS v3.11，未签名。

迁移专属回归（原清单成文于全量 TS 迁移之前，故补充）：

- A1 文件树刷新：展开多层后外部新增文件，展开状态保持且新文件出现。
  对应迁移中修掉的 `_collectLoadedDirs` 空值判断错误——修复前任何带子目录的项目刷新都会抛异常。
- A2 ECL 内建函数：多 eclmap 项目中 `sin` / `cos` / `sqrt` 补全与高亮正常。
  对应 `0f96ec6` 修掉的 builtins 被清空问题，单 eclmap 不复现。
- A3 会话恢复：标签、激活项、未保存草稿在重启后完整回来。
- A4 面板冒烟：终端多标签、输出面板、文件树增删改 / 拖拽 / 复制粘贴。

一轮通过不等于覆盖完全——这是单人单次走查，不是回归套件。触及这些路径的改动仍应重跑本清单。

### 验收中确认的既有行为

**安装包不含 thtk 工具。** `tauri.conf.json` 没有配 `bundle.resources`，而 `thtk_dir` 默认为空串，
为空时 `resolve_tool_path` 退回裸 exe 名走 `PATH`（`common/toolchain.rs:105`）。
因此全新安装后所有工具链动作都不可用，直到用户在「工具链设置」里指定 thtk 目录。
考虑到 thtk 是第三方 GPL 工具，不随包分发是合理的，但首次运行体验值得单独处理
（候选方案：首启引导，或工具链不可用时在界面上给出明确指引而非报错）。

## 最终验证命令

```bash
npm run test
npm run typecheck
npm run build
```

```bash
P=/data/sunyunbo/miniconda3/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig
export LD_LIBRARY_PATH=$P/lib
export PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

## 完成定义

- 启动、打开、恢复、切换项目和编辑项目配置形成一条无死角的用户路径。
- 项目切换不会静默丢失脏标签，也不会在加载失败时破坏当前工作区。
- 最近项目和项目配置均由 Rust 侧提供稳定、可测试的数据契约。
- 新增前端领域边界使用 TypeScript，并有自动化测试覆盖。
- README、状态文档、实现代码与手动验收清单保持一致。
