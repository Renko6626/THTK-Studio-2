# 前端 TypeScript 全量迁移计划

> **给执行者：** 本计划用 `superpowers:subagent-driven-development` 或 `superpowers:executing-plans` 逐任务推进。步骤用 `- [ ]` 复选框跟踪。

**日期：** 2026-07-28
**目标：** 把 `src/` 下 57 个 `.js`（5400 行）与 23 个 `.vue`（3895 行）全部迁到 TypeScript，最终关闭 `allowJs`，使前端任何新增代码都必须过类型检查。

**架构：** 自底向上分档推进。每一档只在其依赖已经 typed 之后才动，否则 `.ts` 文件 import 未迁移的 `.js` 只能拿到推断类型甚至 `any`，类型工作等于空转。跨 Rust 边界的数据结构先在 `src/types/` 集中定义，作为所有上层的共同基准。

**技术栈：** TypeScript 5.9（**必须钉 5.x**）、vue-tsc 3.x、Vitest 4、Vue 3.5、Pinia 3、naive-ui 2.43、monaco-editor 0.53。

---

## 实施状态（2026-07-28 完成）

13 个任务全部完成。`src/` 与 `tests/` 下 `.js` 清零，`allowJs: false`，
23 个 `.vue` 中 22 个标了 `lang="ts"`（`EmptyEditorState.vue` 是纯模板、无 script）。

最终门禁：`npm run typecheck` / `npm test`（45）/ `npm run build` / `cargo test`（132）全绿，
并用 Vite dev server 实测确认 `index.html` → `/src/main.ts` 能正常解析。

### 与计划的偏差

- **`sessionRuntime` 从 Task 5 提前到 Task 4。** terminal store 依赖它，而计划把
  services 排在 stores 之后——这是原计划的排序错误。
- **测试文件也一并迁移并纳入门禁**（计划只提到 Task 13 顺带做）。收益不小：
  `vi.mock` 的替身此前完全不受检查，迁移后暴露出多处 fixture 缺字段。为此新增
  `tests/helpers/naive.ts`（naive-ui API 替身）与 `tests/helpers/fixtures.ts`
  （数据构造器）——测试通常只关心一两个字段，但类型要求完整结构。
- **新增 `src-tauri/src/wire_format_tests.rs`**（计划中没有）。前端类型与 Rust
  结构体之间没有自动同步机制，而本项目的 serde 命名风格并不统一。这 9 个测试
  钉住序列化后的字段名，改结构体时会失败并提醒同步更新 `src/types/`。

### 迁移期间发现并修复的真实 bug

两个都由类型检查暴露、经测试实证后单独提交（迁移提交只做类型）：

1. **文件树刷新在有未展开目录时必崩**（`4073c49`）。`_collectLoadedDirs` 用
   `children !== undefined` 判断"已加载"，但后端对未展开目录发的是 `null`
   （Rust `Option<Vec>`），于是 `null.length` 抛 TypeError。任何带子目录的项目
   刷新都是坏的，而异常是未捕获的 Promise rejection，表现为"刷新没反应"。
2. **多份 eclmap 合并时 builtins 被清空**（`0f96ec6`）。`builtins` 是 `string[]`，
   却被喂给按 `name` 去重的 `dedupeByName`，结果全部被跳过——内置函数
   （sin / cos 等）在补全与高亮里整体消失。

另外清掉若干死代码：`runThtkRaw`（后端根本没有 `run_thtk_raw` 命令）、
`theclMetadata` 里重复的 `createDefaultBuildPayload`、`useWorkbenchShortcuts`
从未使用的 `projectStore` 参数、`tokenizer` 里算完不用的 `normalizeEclSemanticData`、
`renderFileIcon` 的死导入、`FileTree` 解构却不用的 `draggingNode`。

### 仍未解决

- **`.vue` 的模板表达式检查有限**。SFC 已进门禁，但模板里的类型推断不如
  `<script>` 严格；组件层依然零测试覆盖。Task 10–12 的安全网只有 typecheck +
  build + 人工审阅（三档均已逐文件核对模板区零改动）。
- **`uno.config..js` 文件名笔误**未处理。它是构建配置而非 `src/` 代码，改名会让
  UnoCSS preflight 突然生效并改变全局样式，需单独一批并实机比对。

## Global Constraints

- `strict: true` 全程保持开启。不得为了让某一档过关而临时关掉。
- `typescript` 钉在 `~5.9`。7.x 移除了 `./lib/tsc` 导出，`vue-tsc` 直接起不来。
- 每一档结束时三个门禁全绿：`npm run typecheck && npm test && npm run build`。Rust 侧不受影响，但合并前需再跑一次 `cargo test --manifest-path src-tauri/Cargo.toml`。
- **类型迁移与行为修复分开提交。** 迁移 commit 只做类型；`strict` 暴露出的真实空值/逻辑 bug 单独一个 commit，并配一个会失败的测试。审阅时必须能一眼分清哪些是机械转换、哪些是真改动。
- 不得用 `any` 蒙混。确实无法表达时用 `unknown` 加显式收窄，并写注释说明为什么。临时用 `as` 断言必须带注释指明它依赖哪个未迁移的模块，等那个模块迁完后删掉。
- 不新增运行时依赖。类型包（`@types/*`）可以加。
- 文件重命名一律用 `git mv`，保留历史。
- `src/` 内的相对 import **不带扩展名**（现状如此，勘察确认过），改名不会断。但 `tests/` 里有 11 处带 `.js` 后缀的 import 与 `vi.mock` 路径，对应文件改名时必须同步改，否则测试直接报模块找不到。

### 序列化命名不统一（最容易踩的坑）

Rust 侧三种命名风格并存，类型定义必须逐个核对 `src-tauri/src/` 的结构体，不能想当然：

| 结构 | 位置 | 风格 |
| --- | --- | --- |
| `ProjectConfig` / `RecentProject` / `ProjectConfigLoad` | `common/project_config.rs`、`common/recent_projects.rs` | `rename_all = "camelCase"` |
| `AppConfig` | `config.rs` | **无 `rename_all` → snake_case**（`thtk_dir`、`default_game_version`） |
| `FileNode` | `common/fs_utils.rs` | **混合**：`is_dir`、`size` 是 snake_case，但 `is_leaf` 带 `#[serde(rename = "isLeaf")]` |
| `FileCategory` | `common/fs_utils.rs` | `rename_all = "camelCase"`（`sourceScript`、`binaryScript`…） |
| `ToolchainStatus` | `common/toolchain.rs` | `rename_all = "camelCase"` |

写每个类型时打开对应的 `.rs` 文件逐字段对，写完在类型定义上方注释标明来源文件。

---

## 迁移配方

每个文件都走同一套流程，后续任务不再重复：

1. `git mv path/to/file.js path/to/file.ts`
2. 跑 `npm run typecheck`，把报错从上到下修掉
3. 该文件如果被 `tests/` 引用，同步改测试里的 import 路径与 `vi.mock` 路径
4. 一档做完跑三门禁，然后提交

**常见报错与对策：**

- `Property 'x' does not exist on type 'never'` — Pinia store 的 `state: () => ({ items: [] })` 会推断成 `never[]`。给 state 写显式接口：`state: (): XxxState => ({ ... })`。
- `Object is possibly 'null' / 'undefined'` — 这类**大概率是真 bug**。按 Global Constraints 记下来，迁移 commit 里先用最小改动让它编过（可选链或早返回），行为修复单独提交。
- 隐式 `any` 的函数参数 — 补形参类型。若类型来自尚未迁移的模块，写局部 `interface XxxLike` 只声明真正用到的字段，并注释「等 X 迁完删掉」。
- 第三方类型 — naive-ui、monaco-editor、`@tauri-apps/api` 都自带类型，直接 `import type` 即可，不要另装 `@types/*`。

### `.vue` 在 Task 10 之前解析为 `any`（实测确认）

`tsconfig.json` 的 `include` 目前只有 `src/**/*.ts`，所以 `.ts` 文件 import 的
`.vue` 组件**完全不做检查**——用探针验证过：访问一个根本不存在的属性
（`MonacoEditor.__definitelyNotAProp`）typecheck 照样通过。

影响：`services/workbench/editorViews.ts`、`services/toolchains/registry.ts` 里
对组件的引用现在是零保护，组件的 props 契约也无从校验。这不是可以绕过的问题，
而是 Task 13 把 `src/**/*.vue` 加进 `include` 之后才会真正生效。在那之前不要
以为「typecheck 通过」意味着组件用法正确。

---

## Task 1：共享类型基座

**Files:**
- Modify: `src/types/project.ts`
- Create: `src/types/fs.ts`、`src/types/toolchain.ts`、`src/types/index.ts`

**Interfaces:**
- Produces：`FileNode`、`FileCategory`、`AppConfig`、`ToolchainStatus`、`EclMapSemanticData`、`TheclRequest`、`TheclResult`、`Diagnostic`。后续所有档次从 `src/types` 导入这些。
- Consumes：无。

- [ ] **Step 1: 对着 Rust 结构体写 `src/types/fs.ts`**

打开 `src-tauri/src/common/fs_utils.rs` 逐字段核对。注意 `isLeaf` 是唯一被 rename 的字段：

```ts
/** 对应 src-tauri/src/common/fs_utils.rs 的 FileCategory（rename_all = "camelCase"） */
export type FileCategory =
  | 'sourceScript'
  | 'binaryScript'
  | 'archive'
  | 'image'
  | 'assetDefinition'
  | 'directory'
  | 'unknown'

/**
 * 对应 src-tauri/src/common/fs_utils.rs 的 FileNode。
 * 该结构体**没有** rename_all，所以字段是 snake_case——只有 is_leaf 带了
 * #[serde(rename = "isLeaf")]。不要统一成 camelCase。
 */
export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  size: number | null
  extension: string | null
  category: FileCategory
  children?: FileNode[] | null
  isLeaf: boolean
  lossy: boolean
}
```

- [ ] **Step 2: 写 `src/types/toolchain.ts`**

打开 `src-tauri/src/config.rs` 与 `src-tauri/src/common/toolchain.rs` 核对：

```ts
/**
 * 对应 src-tauri/src/config.rs 的 AppConfig。
 * 该结构体没有 rename_all，字段保持 snake_case。
 */
export interface AppConfig {
  thtk_dir: string
  thecl_path: string
  eclmap_path: string
  tool_overrides: Record<string, string>
  default_game_version: string
  theme: string
  mcp_port: number
  recent_projects: RecentProjectStored[]
}

/** AppConfig 里落盘的形态，没有 available 字段 */
export interface RecentProjectStored {
  path: string
  name: string
  lastOpenedAt: number
}

/** 对应 common/toolchain.rs 的 ToolchainStatus（rename_all = "camelCase"） */
export interface ToolchainStatus {
  tool: string
  label: string
  exeName: string
  configuredPath: string
  resolvedPath: string
  available: boolean
  version: string
  message: string
}

export type ToolchainId = 'thecl' | 'thmsg' | 'thanm' | 'thstd' | 'thdat'
```

- [ ] **Step 3: 在 `src/types/project.ts` 补 ECL 与诊断相关类型**

`ProjectOpenResult.files` 当前是 `unknown[]`，换成真实类型：

```ts
import type { FileNode } from './fs'

export interface ProjectOpenResult {
  rootPath: string
  files: FileNode[]
  projectConfig: ProjectConfigLoad
}

/** 对应 modules/ecl/error_parser.rs 产出的诊断 */
export interface Diagnostic {
  path: string
  line: number
  column: number
  severity: 'error' | 'warning' | 'info'
  message: string
}

export interface EclInstruction {
  opcode: number
  name: string
  signature?: string
  description?: string
}

export interface EclMapSemanticData {
  version: string
  sourcePath: string
  resolvedPath?: string
  instructions: EclInstruction[]
  builtins: EclInstruction[]
  error?: string
}
```

- [ ] **Step 4: 写 `src/types/index.ts` 作为统一出口**

```ts
export * from './fs'
export * from './project'
export * from './toolchain'
```

- [ ] **Step 5: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add src/types && git commit -m "feat(types): 建立跨 Rust 边界的共享类型基座"
```

**验收：** `npm run typecheck` 通过；每个类型定义上方都注明了对应的 `.rs` 文件；`ProjectOpenResult.files` 不再是 `unknown[]`。

---

## Task 2：utils（3 个文件，203 行）

**Files:**
- Modify: `src/utils/pathNormalize.js` → `.ts`、`src/utils/workbenchState.js` → `.ts`、`src/utils/renderFileIcon.js` → `.ts`
- Modify: `tests/unit/workbenchSession.spec.js`（`vi.mock` 路径去掉 `.js`）

**Interfaces:**
- Produces：`normalizePath(path: string): string`、`pathsEqual(a: string, b: string): boolean`、`snapshotStorageKeys()`、`loadProjectSnapshot()` 等快照读写函数、`renderFileIcon(node: FileNode)`。
- Consumes：Task 1 的 `FileNode`。

- [ ] **Step 1: 迁 `pathNormalize`（最简单，先热身）**

```bash
git mv src/utils/pathNormalize.js src/utils/pathNormalize.ts
```

签名改成：

```ts
export function normalizePath(path: string): string
export function pathsEqual(a: string | null | undefined, b: string | null | undefined): boolean
```

注意现有实现里 `normalizePath` 对非字符串会原样返回，迁移后签名收紧为 `string`，调用方传 `null` 的地方会在 typecheck 里暴露——那些就是 Global Constraints 说的「真 bug」，记下来单独修。

- [ ] **Step 2: 迁 `workbenchState`，给快照结构写类型**

```ts
export interface ProjectSnapshot {
  rootPath: string | null
}

export interface EditorTabSnapshot {
  path: string
  name: string
  isDirty: boolean
  language: string
  viewType: 'text' | 'binary-script'
  size: number | null
  extension: string | null
  category: FileCategory | null
  /** 仅脏的文本标签且未超预算时才落盘 */
  content?: string
  originalContent?: string
}

export interface EditorSnapshot {
  activePath: string | null
  tabs: EditorTabSnapshot[]
}
```

- [ ] **Step 3: 迁 `renderFileIcon`**

它返回 Vue VNode，用 `import type { VNode } from 'vue'` 标注返回值。

- [ ] **Step 4: 修测试里的 import 路径**

`tests/unit/workbenchSession.spec.js` 里 `vi.mock('../../src/utils/workbenchState.js', ...)` 与顶部 import 的 `.js` 后缀全部去掉。

- [ ] **Step 5: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(utils): 迁移到 TypeScript"
```

**验收：** 三门禁通过；`EditorTabSnapshot` 与 `src/stores/editor.js` 的 `toSnapshot()` 实际产出一致。

---

## Task 3：api 层（7 个文件，333 行）

这是性价比最高的一档：它是前后端契约的唯一入口，typed 之后所有 store 和 composable 白拿类型。

**Files:**
- Modify: `src/api/index.js` → `.ts`
- Modify: `src/api/modules/{compiler,config,eclMap,fs,fsOps,pty}.js` → `.ts`

**Interfaces:**
- Produces：全部 `invoke` 包装函数的 typed 签名。后续所有档次依赖这些返回类型。
- Consumes：Task 1 的全部类型。

- [ ] **Step 1: 迁 `src/api/modules/fs.ts`**

```ts
import { invoke } from '@tauri-apps/api/core'
import type { FileNode, ProjectOpenResult } from '../../types'

export function getFileTree(path: string): Promise<FileNode[]> {
  return invoke('get_file_tree', { path })
}

export function getDirChildren(path: string): Promise<FileNode[]> {
  return invoke('get_dir_children', { path })
}

export function openProject(path: string): Promise<ProjectOpenResult> {
  return invoke('open_project', { path })
}

export function readFile(path: string): Promise<string> {
  return invoke('read_file', { path })
}
```

- [ ] **Step 2: 迁 `config.ts`**

保留现有 JSDoc 里的语义说明（三态、available 不落盘等），把 `@returns` 换成真实类型：

```ts
import type { AppConfig, ProjectConfig, ProjectConfigLoad, RecentProjectView, ToolchainStatus } from '../../types'

export function getSettings(): Promise<AppConfig> {
  return invoke('get_settings')
}

export function saveSettings(config: AppConfig): Promise<void> {
  return invoke('save_settings', { config })
}

export function loadProjectConfig(): Promise<ProjectConfigLoad> {
  return invoke('load_project_config')
}

export function saveProjectConfig(config: ProjectConfig, expectedRoot: string): Promise<void> {
  return invoke('save_project_config_cmd', { config, expectedRoot })
}

export function listRecentProjects(): Promise<RecentProjectView[]> {
  return invoke('list_recent_projects')
}
```

- [ ] **Step 3: 迁 `compiler.ts`、`fsOps.ts`、`pty.ts`、`eclMap.ts`**

`compiler.ts` 的请求/结果结构对照 `src-tauri/src/modules/ecl/compiler.rs` 的 `TheclRequest` / `EclResult`；这两个结构体带 `rename_all = "camelCase"`，逐字段核对后把类型加进 `src/types/toolchain.ts`。

- [ ] **Step 4: 迁 `src/api/index.ts`**

只是 `export *`，无需改动内容，`git mv` 即可。

- [ ] **Step 5: 修测试里的 mock**

`tests/unit/{projectActions,stores,semanticLoader}.spec.js` 里的 `vi.mock('../../src/api', ...)` 路径不带扩展名，无需改；但 mock 工厂返回的对象现在要与真实签名兼容，跑测试确认没有因类型收紧而失败。

- [ ] **Step 6: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(api): 迁移到 TypeScript，前后端契约进入类型门禁"
```

**验收：** 所有 `invoke` 包装都有确切返回类型，没有一个是 `Promise<any>`；`AppConfig` 用 snake_case、`ProjectConfig` 用 camelCase，与 Rust 侧一致。

---

## Task 4：stores（9 个文件，1090 行）

**Files:**
- Modify: `src/stores/{buildDialog,editor,explorerClipboard,explorerView,project,terminal,toolchainSettings,workbenchPanels,workbenchReports}.js` → `.ts`
- Modify: `tests/unit/stores.spec.js`、`tests/unit/projectActions.spec.js`（import 去后缀）
- Modify: `src/composables/useProjectActions.ts`（删除 `EditorTabLike` 临时接口）

**Interfaces:**
- Produces：全部 store 的 typed state / getters / actions。特别是 `EditorTab`，供 composable 和组件使用。
- Consumes：Task 1 类型、Task 3 的 api 签名。

- [ ] **Step 1: 先迁小的（`toolchainSettings`、`explorerView`、`buildDialog`、`explorerClipboard`）**

这四个都是十几到三十行的 UI 状态 store，模式一致：

```ts
interface ToolchainSettingsState {
  visible: boolean
}

export const useToolchainSettingsStore = defineStore('toolchainSettings', {
  state: (): ToolchainSettingsState => ({ visible: false }),
  actions: {
    open() { this.visible = true },
    close() { this.visible = false }
  }
})
```

- [ ] **Step 2: 迁 `editor.ts`（331 行，本档核心）**

先定义标签类型，它会被 composable 和组件反复使用：

```ts
import type { FileCategory } from '../types'

export type EditorViewType = 'text' | 'binary-script'

export interface EditorTab {
  path: string
  name: string
  content: string
  originalContent: string
  language: string
  viewType: EditorViewType
  isDirty: boolean
  size: number | null
  extension: string | null
  category: FileCategory | null
}

interface EditorState {
  tabs: EditorTab[]
  activePath: string | null
}
```

`saveActiveFile()` 目前在非 text 标签上会返回 `undefined`，而 `saveAllFiles()` 把它当失败处理——迁移时把返回类型标成 `Promise<boolean>`，编译器会逼出这个分支。这是真 bug，按 Global Constraints 单独提交修复（早返回改成 `return true`），并在 `tests/unit/stores.spec.js` 补一条断言。

- [ ] **Step 3: 迁 `project.ts`**

```ts
import type { FileNode, ProjectConfig, ProjectConfigLoad, ProjectConfigStatus } from '../types'

interface ProjectState {
  rootPath: string | null
  files: FileNode[]
  isLoading: boolean
  projectConfig: ProjectConfig | null
  projectConfigStatus: ProjectConfigStatus
  projectConfigError: string | null
  projectConfigPath: string
  _refreshPromise: Promise<void> | null
  _refreshPending: boolean
}
```

- [ ] **Step 4: 迁 `workbenchReports.ts`、`workbenchPanels.ts`、`terminal.ts`**

`workbenchReports` 的 `outputEntries` / `problemEntries` 需要 `OutputEntry` / `ProblemEntry` 接口；`publishToolResult` 的参数对象写成具名 interface。

- [ ] **Step 5: 删掉 `useProjectActions.ts` 里的临时接口**

`EditorTabLike` 是当初为绕开 `never[]` 加的，注释里写明了「editor store 迁到 TS 之后应该删掉」。现在删，改成 `import type { EditorTab } from '../stores/editor'`，并去掉那处 `as` 断言。

- [ ] **Step 6: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(stores): 迁移到 TypeScript"
```

**验收：** 三门禁通过；`useProjectActions.ts` 里不再有 `EditorTabLike` 与对应的 `as` 断言；行为修复是独立 commit 且带测试。

---

## Task 5：services（非 ECL，5 个文件，333 行）

**Files:**
- Modify: `src/services/toolchains/{registry,thecl,theclMetadata}.js` → `.ts`
- Modify: `src/services/workbench/editorViews.js` → `.ts`
- Modify: `src/services/terminal/sessionRuntime.js` → `.ts`

**Interfaces:**
- Produces：`ToolchainDescriptor`（注册表条目的形状，新增工具链时的契约）、`WorkbenchEditorView`。
- Consumes：Task 1 类型、Task 3 api、Task 4 stores。

- [ ] **Step 1: 给 `registry.ts` 的描述符写接口**

这是 CLAUDE.md 里点名的「registry-driven extensibility point」，类型化之后新增工具链会有编译期检查：

```ts
import type { ToolchainId } from '../../types'

export interface ToolchainDescriptor {
  id: ToolchainId
  label: string
  exeName: string
  supportsBuildDialog: boolean
  buildDialogSubtitle?: string
  createDefaultPayload?: () => Record<string, unknown>
  buildRequest?: (payload: Record<string, unknown>) => unknown
  execute?: (request: unknown) => Promise<unknown>
}

export const TOOLCHAIN_REGISTRY: Record<ToolchainId, ToolchainDescriptor> = { /* ... */ }
```

`createDefaultPayload` / `buildRequest` / `execute` 目前只有 `thecl` 实现，其余四个是 stub。用可选属性表达这一点，不要为了让 stub 编过而放宽成 `any`。

- [ ] **Step 2: 迁 `editorViews.ts`**

```ts
import type { Component } from 'vue'
import type { EditorTab, EditorViewType } from '../../stores/editor'

export interface WorkbenchEditorView {
  id: EditorViewType
  component: Component
  statusLabel?: (tab: EditorTab) => string
}
```

- [ ] **Step 3: 迁 `sessionRuntime.ts`**

它持有模块级的 xterm.js `Terminal` 实例。包名是 `@xterm/xterm`（不是旧的 `xterm`），自带类型：

```ts
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
```

现有实现已经是这两个 import，迁移时只需给模块级的会话表标注类型，例如
`const sessions = new Map<string, { terminal: Terminal; fitAddon: FitAddon }>()`。

- [ ] **Step 4: 迁 `thecl.ts`、`theclMetadata.ts`**

`theclMetadata.ts` 里的 `createDefaultTheclPayload()` 定义了构建对话框的表单形状，
Task 10 与 Task 12 的组件都依赖它，在这里定型：

```ts
export interface TheclBuildPayload {
  tool: 'thecl'
  mode: 'compile' | 'decompile' | 'header'
  inputPath: string
  version: string
  outputPath: string
  mapPaths: string[]
  useShiftJis: boolean
  rawDump: boolean
  simpleCreation: boolean
  showOffsets: boolean
}

export function createDefaultTheclPayload(): TheclBuildPayload
```

字段以 `src/services/toolchains/theclMetadata.js` 现有实现为准，迁移时逐个核对不要凭记忆。

- [ ] **Step 5: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(services): 工具链注册表与工作区视图迁移到 TypeScript"
```

**验收：** `TOOLCHAIN_REGISTRY` 的五个条目都满足 `ToolchainDescriptor`；新增一个字段拼错时 typecheck 会报错。

---

## Task 6：composables — 工作台组（6 个文件，312 行）

**Files:**
- Modify: `src/composables/{useBeforeUnloadGuard,useEditorActionBridge,useMcpBridge,useResizable,useWorkbenchSession,useWorkbenchShortcuts}.js` → `.ts`
- Modify: `tests/unit/workbenchSession.spec.js`（import 去后缀）

**Interfaces:**
- Produces：`useWorkbenchSession({...}): { flushSnapshots: () => void }`、`useWorkbenchShortcuts` 的注入参数类型。
- Consumes：Task 2 utils、Task 4 stores。

- [ ] **Step 1: 给 `useWorkbenchSession.ts` 的依赖注入写接口**

它接收四个 store 加两个回调，目前全靠解构没有任何约束：

```ts
import type { useEditorStore } from '../stores/editor'
import type { useProjectStore } from '../stores/project'

interface WorkbenchSessionDeps {
  projectStore: ReturnType<typeof useProjectStore>
  editorStore: ReturnType<typeof useEditorStore>
  terminalStore: ReturnType<typeof useTerminalStore>
  workbenchPanelsStore: ReturnType<typeof useWorkbenchPanelsStore>
  showReloadNotice: (text: string) => void
  openProjectPath: (path: string, options?: { silent?: boolean }) => Promise<boolean>
}
```

`ReturnType<typeof useXxxStore>` 是 Pinia 的标准写法，能拿到完整的 store 类型而不必手写。

- [ ] **Step 2: 迁 `useWorkbenchShortcuts.ts`**

同样用 `WorkbenchShortcutsDeps` 接口，`openFolder: () => Promise<boolean>`。

- [ ] **Step 3: 迁其余四个小文件**

`useResizable`、`useBeforeUnloadGuard`、`useEditorActionBridge`、`useMcpBridge` 都在 40 行以内。

- [ ] **Step 4: 修测试 import 并验证提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(composables): 工作台层迁移到 TypeScript"
```

**验收：** `tests/unit/workbenchSession.spec.js` 里的假 store 若缺字段，typecheck 会报错（说明注入契约真的被约束住了）。

---

## Task 7：composables — 文件树组（4 个文件，853 行）

**Files:**
- Modify: `src/composables/{useContextMenu,useFileOperations,useFileTreeActions,useFileTreeDnD}.js` → `.ts`

**Interfaces:**
- Produces：文件树右键菜单项、拖拽负载与批量操作的类型。
- Consumes：Task 1 `FileNode`、Task 3 api（`fsOps`）、Task 4 stores。

- [ ] **Step 1: 迁 `useContextMenu.ts`**

菜单项对接 naive-ui 的 `DropdownOption`，直接 `import type { DropdownOption } from 'naive-ui'`。

- [ ] **Step 2: 迁 `useFileOperations.ts`**

它的 `handleCreate(parentPath, type)` 的 `type` 是 `'file' | 'dir'` 字面量联合，写出来能防止拼错。

- [ ] **Step 3: 迁 `useFileTreeActions.ts`（336 行）**

这个文件处理多选与批量操作，`getActionEntries()` 返回 `FileNode[]`。注意它内部会 `findNodeByPath` 过滤掉失效路径，返回值可能为空数组。

- [ ] **Step 4: 迁 `useFileTreeDnD.ts`**

拖拽负载写成具名类型，并显式拒绝 OS 外部文件拖入（现有逻辑已有，类型化时保留）。

- [ ] **Step 5: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(composables): 文件树交互层迁移到 TypeScript"
```

**验收：** 三门禁通过。本档无测试覆盖，提交前人工过一遍 diff 确认没有行为改动混入。

---

## Task 8：composables — 工具链组（5 个文件，786 行）

**Files:**
- Modify: `src/composables/{useEclSemanticVocabulary,useFileWatcher,useTheclActions,useToolchainActions,useToolchainResult}.js` → `.ts`

**Interfaces:**
- Produces：`useToolchainActions` 的全部 `run*` 方法签名、`useToolchainResult` 的发布参数类型。
- Consumes：Task 3 api、Task 4 stores、Task 5 registry。

- [ ] **Step 1: 迁 `useToolchainResult.ts`**

先迁它，因为 `useToolchainActions` 依赖它。发布参数写成具名 interface（字段见现有实现的解构默认值）。

- [ ] **Step 2: 迁 `useToolchainActions.ts`（347 行，本档最大）**

它现在用 JSDoc 写了 `@param {{ message?: import('naive-ui').MessageApiInjection }}`，换成 `import type { MessageApi } from 'naive-ui'`。

两个名字都有效：naive-ui 的 `es/message/index.d.ts` 里是
`export type { MessageApiInjection as MessageApi, ... }`。统一用 `MessageApi`，
与已迁移的 `useProjectActions.ts` 保持一致。

- [ ] **Step 3: 迁 `useTheclActions.ts`**

`applyProjectDefaults(request, projectStore)` 的 request 类型来自 Task 3 的 `TheclRequest`。

- [ ] **Step 4: 迁 `useEclSemanticVocabulary.ts`、`useFileWatcher.ts`**

- [ ] **Step 5: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(composables): 工具链动作层迁移到 TypeScript"
```

**验收：** 三门禁通过；`useToolchainActions` 里五个工具链的 `run*` 方法参数类型互不混用。

---

## Task 9：ECL 语言服务（17 个文件，1477 行）

**Files:**
- Modify: `src/services/languages/ecl/*.js` → `.ts`（全部 17 个）
- Create: `tests/unit/eclSemanticState.spec.ts`

**Interfaces:**
- Produces：Monaco provider 的注册函数、`updateEclSemanticVocabulary` / `clearEclSemanticVocabulary` 的 typed 签名。
- Consumes：Task 1 的 `EclMapSemanticData` / `EclInstruction`、Task 3 的 `eclMap` api。

本档是最大的一块，但 `monaco-editor` 自带完整类型，provider 的签名会被 Monaco 的接口直接约束住，实际比行数看起来顺。

- [ ] **Step 1: 先补一层特征测试再动手**

本档目前零测试覆盖，而它是纯逻辑（无 DOM、无 Tauri），适合先钉住行为。给 `semantic-state.js` 的作用域管理写测试：

`semantic-state.js` 的实际导出是这五个（已核对）：`setActiveEclSemanticScope`、
`updateScopedEclSemanticData`、`clearScopedEclSemanticData`、`getEclSemanticDataForModel`、
`getActiveEclSemanticData`。**没有**按作用域直接取数据的 getter，读取只能经「活动作用域」
或「Monaco model」两条路径。测试要照这个 API 写：

```ts
import { describe, expect, it } from 'vitest'
import {
  clearScopedEclSemanticData,
  getActiveEclSemanticData,
  setActiveEclSemanticScope,
  updateScopedEclSemanticData
} from '../../src/services/languages/ecl/semantic-state'

const dataFor = (version: string) => ({
  version,
  sourcePath: '',
  instructions: [{ opcode: 1, name: `ins_${version}` }],
  builtins: []
})

describe('ECL 词表作用域', () => {
  it('按作用域隔离，切换活动作用域拿到各自的词表', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    updateScopedEclSemanticData('/proj/b', dataFor('17'))

    setActiveEclSemanticScope('/proj/a')
    expect(getActiveEclSemanticData().version).toBe('18')

    setActiveEclSemanticScope('/proj/b')
    expect(getActiveEclSemanticData().version).toBe('17')
  })

  it('清掉一个作用域后回落到空词表，不影响另一个', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    updateScopedEclSemanticData('/proj/b', dataFor('17'))

    clearScopedEclSemanticData('/proj/a')

    setActiveEclSemanticScope('/proj/a')
    // 取不到时返回的是空词表而不是 null，迁移后要保持这个契约
    expect(getActiveEclSemanticData().instructions).toEqual([])

    setActiveEclSemanticScope('/proj/b')
    expect(getActiveEclSemanticData().version).toBe('17')
  })
})
```

先跑一遍确认通过（这是特征测试，不是 TDD 的红灯），再开始迁移。

- [ ] **Step 2: 迁数据层（`semantic-state`、`vocabulary`、`dynamic-vocabulary`、`semantic-loader`、`instruction-display`）**

`semantic-loader` 已经有 7 个测试，迁完跑一遍确认全绿。

- [ ] **Step 3: 迁语法层（`tokenizer`、`theme`、`language-config`）**

`tokenizer` 返回 Monaco 的 `languages.IMonarchLanguage`，`theme` 返回 `editor.IStandaloneThemeData`，直接用 Monaco 的类型标注。

- [ ] **Step 4: 迁六个 Monaco provider**

确切文件：`completion-provider.js`、`hover-provider.js`、`definition-provider.js`、
`references-provider.js`、`signature-help-provider.js`、`document-symbols.js`
（均在 `src/services/languages/ecl/` 下）。

每个 provider 实现 Monaco 对应的接口，例如：

```ts
import * as monaco from 'monaco-editor'

export const eclCompletionProvider: monaco.languages.CompletionItemProvider = {
  provideCompletionItems(model, position) { /* ... */ }
}
```

用接口标注后，Monaco 会检查返回值形状——现有代码里若有字段名写错或缺 `range`，这一步会暴露。

- [ ] **Step 5: 迁诊断层（`static-diagnostics`、`toolchain-diagnostics`）与 `register`**

- [ ] **Step 6: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(ecl): 语言服务迁移到 TypeScript"
```

**验收：** 三门禁通过；provider 全部用 Monaco 官方接口标注而非自定义结构；`semantic-loader` 的 7 个既有测试仍绿。

---

## Task 10：组件 — 布局与编辑器（8 个文件，945 行）

从这里开始动 `.vue`。**本档起没有自动化安全网**：组件层零测试覆盖，开发机也跑不起 Tauri 窗口。安全网只有 typecheck + build + 人工审阅，因此每一档都必须严格「只改类型不改行为」。

**Files:**
- Modify: `src/App.vue`、`src/components/Layout/{WorkbenchRoot,WorkbenchLayout}.vue`、`src/components/Editor/{MonacoEditor,BinaryScriptView,TabGroup,WorkbenchEditorHost,EmptyEditorState}.vue`

- [ ] **Step 1: 给每个 SFC 的 `<script setup>` 加 `lang="ts"`**

```vue
<script setup lang="ts">
```

- [ ] **Step 2: 把 `defineProps` / `defineEmits` 改成类型形式**

```vue
<script setup lang="ts">
const props = defineProps<{ model: TheclBuildPayload }>()
const emit = defineEmits<{ 'update:model': [value: TheclBuildPayload] }>()
</script>
```

不要保留运行时的 `{ type: Object, required: true }` 写法——类型形式才进 typecheck。

- [ ] **Step 3: 处理 `ref` 的显式类型**

模板引用与可空状态要标注，例如 `const editorRef = ref<HTMLElement | null>(null)`。

- [ ] **Step 4: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(components): 布局与编辑器组件迁移到 TypeScript"
```

**验收：** 三门禁通过；diff 里没有任何模板结构变化，只有 `<script>` 块的类型标注。

---

## Task 11：组件 — 侧边栏与工具窗（8 个文件，1432 行）

**Files:**
- Modify: `src/components/Sidebar/{FileTree,RightSidebar,EclOutlinePanel,EclReferencesPanel}.vue`
- Modify: `src/components/ToolWindow/{BottomPanelHost,TerminalPanel,OutputPanel,ProblemsPanel}.vue`

- [ ] **Step 1: 先迁四个小的（`RightSidebar`、`BottomPanelHost`、`OutputPanel`、`ProblemsPanel`）**

- [ ] **Step 2: 迁 `EclOutlinePanel`、`EclReferencesPanel`、`TerminalPanel`**

- [ ] **Step 3: 迁 `FileTree.vue`（549 行，全项目最大的组件）**

它同时用了 naive-ui 的 `NTree`（`TreeOption` 类型）、右键菜单、内联重命名和拖拽。`TreeOption` 与项目的 `FileNode` 结构不同，转换处要显式写类型，不要用 `as any` 绕过。

- [ ] **Step 4: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(components): 侧边栏与工具窗迁移到 TypeScript"
```

**验收：** 三门禁通过；`FileTree.vue` 里没有新增 `any`。

---

## Task 12：组件 — 对话框与欢迎页（7 个文件，1518 行）

**Files:**
- Modify: `src/components/Dialogs/{ProjectSettingsDialog,ToolchainSettingsDialog,BuildConfigDialog,BuildDialogShell}.vue`
- Modify: `src/components/Dialogs/forms/TheclBuildForm.vue`
- Modify: `src/components/Welcome/WelcomeView.vue`
- Modify: `src/components/Common/MenuBar.vue`

- [ ] **Step 1: 迁 `BuildDialogShell`、`BuildConfigDialog`、`TheclBuildForm`**

三者通过 `model` 对象串联，用 Task 5 定义的类型统一。

- [ ] **Step 2: 迁 `ProjectSettingsDialog`、`ToolchainSettingsDialog`**

`ProjectSettingsDialog` 的 `form.gameVersion` 是 `string | null`（`clearable` 的 `n-select` 会给 `null`），类型要如实反映，不要收紧成 `string` 后再用 `!` 绕过。

- [ ] **Step 3: 迁 `WelcomeView`、`MenuBar`**

`MenuBar` 的菜单项数组对接 naive-ui `DropdownOption`，含 `divider` 类型的项。

- [ ] **Step 4: 验证并提交**

```bash
npm run typecheck && npm test && npm run build
git add -A && git commit -m "refactor(components): 对话框与欢迎页迁移到 TypeScript"
```

**验收：** 三门禁通过。

---

## Task 13：收口

**Files:**
- Modify: `src/main.js` → `.ts`、`index.html`
- Modify: `tsconfig.json`、`vitest.config.js` → `.ts`
- Modify: `tests/**/*.spec.js` → `.ts`、`tests/helpers/withSetup.js` → `.ts`
- Modify: `CLAUDE.md`、`AGENTS.md`、`README.md`、本计划

- [ ] **Step 1: 迁入口并改 `index.html`**

```bash
git mv src/main.js src/main.ts
```

`index.html` 第 12 行改成 `<script type="module" src="/src/main.ts"></script>`。**漏改这一步应用会白屏**，而 `npm run build` 未必报错——改完必须跑一次 `npm run dev` 确认页面能起。

- [ ] **Step 2: 迁测试文件**

四个 `.spec.js` 与 `tests/helpers/withSetup.js` 全部改 `.ts`，同时把剩余的 `.js` 后缀 import 去掉。`tsconfig.json` 的 `include` 已经含 `tests/**/*.ts`，迁完测试代码本身也进类型门禁。

- [ ] **Step 3: 关闭 `allowJs`**

```json
{
  "compilerOptions": {
    "allowJs": false,
    "checkJs": false
  },
  "include": ["src/**/*.ts", "src/**/*.vue", "tests/**/*.ts"]
}
```

`include` 加上 `src/**/*.vue`，让 vue-tsc 检查 SFC。跑 `npm run typecheck`，此时任何残留的 `.js` 都会以「找不到模块」的形式暴露。

- [ ] **Step 4: 确认没有残留**

```bash
find src -name '*.js' | grep -v node_modules
```

预期输出为空。若有残留，回到对应档次补迁。

- [ ] **Step 5: 更新文档**

- `CLAUDE.md`：把「currently mostly `.js`, despite the docs' TS goal」改成实际状态；把「新增领域边界写 `.ts`」改成「全部代码都是 TS，`allowJs` 已关闭」。
- `AGENTS.md`：无需改（本来就写着用 TypeScript），但可加一句说明门禁位置。
- `README.md`：从「尚未完成」里移除 TypeScript 迁移条目。
- 本计划：在顶部加实施状态表，记录每档的实际提交与遇到的偏差。

- [ ] **Step 6: 最终验证并提交**

```bash
npm run typecheck && npm test && npm run build
cargo test --manifest-path src-tauri/Cargo.toml
git add -A && git commit -m "chore(ts): 关闭 allowJs，前端全量 TypeScript 化收口"
```

**验收：** `find src -name '*.js'` 无输出；`allowJs: false` 下三门禁全绿；`npm run dev` 页面正常。

---

## 完成定义

- `src/` 下没有 `.js` 文件，23 个 `.vue` 全部使用 `lang="ts"`。
- `tsconfig.json` 中 `allowJs: false`、`strict: true`，`include` 覆盖 `src/**/*.ts`、`src/**/*.vue`、`tests/**/*.ts`。
- `npm run typecheck`、`npm test`、`npm run build`、`cargo test` 四项全绿。
- 跨 Rust 边界的每个类型都注明了对应的 `.rs` 文件，且序列化命名风格与后端一致（camelCase / snake_case / 混合三种情况都已如实反映）。
- 迁移过程中发现的行为 bug 均已单独提交并配有会失败的测试；没有任何一个被 `any` 或 `!` 掩盖。
- 文档与实际状态一致。

## 已知风险

- **Task 10–12 没有自动化安全网。** 组件层零测试、本机跑不起 Tauri，只能靠 typecheck + build + 人工审阅。这三档合并后应当在 Windows 上跑一遍 `2026-07-13-mvp-project-workflow-closure.md` 的手动验收清单，再考虑发版。
- **`strict` 会翻出既有 bug。** 这是收益不是成本，但会让某些档次的实际工作量超出行数预估。行为修复必须单独提交，不要为了赶进度混进迁移 commit。
- **`uno.config..js` 文件名笔误不在本计划范围内。** 它是 `.js` 但属于构建配置而非 `src/`，改名会让 UnoCSS preflight 突然生效并改变全局样式，需单独一批并实机比对。
