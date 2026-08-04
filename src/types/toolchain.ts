/**
 * 应用配置与工具链调用的前后端契约。
 * 对应 `src-tauri/src/config.rs`、`src-tauri/src/common/toolchain.rs`
 * 与 `src-tauri/src/modules/ecl/{compiler,error_parser}.rs`。
 */

/** thtk 的五个工具 */
export type ToolchainId = 'thecl' | 'thmsg' | 'thanm' | 'thstd' | 'thdat'

/**
 * `AppConfig` 里落盘的最近项目形态（`common/recent_projects.rs` 的 `RecentProject`，
 * `rename_all = "camelCase"`）。没有 `available`——那是读取时现算的，见
 * `types/project.ts` 的 `RecentProjectView`。
 */
export interface RecentProjectStored {
  path: string
  name: string
  /** Unix 毫秒时间戳 */
  lastOpenedAt: number
}

/**
 * 对应 `config.rs` 的 `AppConfig`。
 *
 * ⚠️ 该结构体**没有** `rename_all`，字段全是 snake_case。不要写成 camelCase。
 */
export interface AppConfig {
  thtk_dir: string
  thecl_path: string
  eclmap_path: string
  /** key 为 ToolchainId，值为该工具 exe 的覆盖路径 */
  tool_overrides: Record<string, string>
  default_game_version: string
  theme: string
  mcp_port: number
  recent_projects: RecentProjectStored[]
}

/**
 * 工具链设置表单实际提交的字段。
 *
 * ⚠️ 刻意**不是**完整的 `AppConfig`：`mcp_port` 与 `recent_projects` 由后端维护，
 * 没有表单入口。后端的 `merge_user_settings` 会从当前配置继承这两项——早先
 * `save_settings` 直接整体替换，保存一次工具链设置就会清空最近项目。
 * 这里用独立类型把该契约写死，避免有人"顺手"补齐成 AppConfig。
 */
export type UserSettingsPayload = Omit<AppConfig, 'mcp_port' | 'recent_projects'>

/** 对应 `common/toolchain.rs` 的 `ToolchainStatus`（`rename_all = "camelCase"`） */
export interface ToolchainStatus {
  tool: string
  label: string
  exeName: string
  configuredPath: string
  resolvedPath: string
  available: boolean
  version: string
  message: string
  /** 该工具实际可用的版本（运行时探测 ∩ 静态表）；不可用时为空数组 */
  supportedVersions: number[]
}

/**
 * 对应 `modules/ecl/error_parser.rs` 的 `Diagnostic`。
 *
 * ⚠️ 该结构体没有 `rename_all`，但字段都是单词所以看不出差别。
 * `column` 是 `Option<u32>`——thecl 不总是给列号。
 * `severity` 在 Rust 侧是裸 `String`，解析器只会产出这两个值。
 */
export interface Diagnostic {
  path: string
  line: number
  column: number | null
  severity: 'error' | 'warning'
  message: string
}

/** 对应 `modules/ecl/compiler.rs` 的 `TheclMode`（`rename_all = "camelCase"`） */
export type TheclMode = 'compile' | 'decompile' | 'header'

/** 对应 `modules/ecl/compiler.rs` 的 `TheclRequest`（`rename_all = "camelCase"`） */
export interface TheclRequest {
  mode: TheclMode
  version: string
  inputPath: string
  /** 留空时后端按模式推导默认产物路径 */
  outputPath: string | null
  mapPaths: string[]
  /** 对应 thecl 的 -j */
  useShiftJis: boolean
  /** 对应 -r，仅反编译有效 */
  rawDump: boolean
  /** 对应 -s，仅编译有效 */
  simpleCreation: boolean
  /** 对应 -x，仅反编译有效 */
  showOffsets: boolean
}

/** 对应 `modules/ecl/compiler.rs` 的 `EclResult`（`rename_all = "camelCase"`） */
export interface EclResult {
  success: boolean
  tool: string
  mode: string
  scriptKind: string
  inputPath: string
  message: string
  diagnostics: Diagnostic[]
  outputPath: string | null
}

/**
 * MSG 与 STD 的结果结构与 `EclResult` 完全一致（都 `rename_all = "camelCase"`）。
 * 两者的 `diagnostics` 在 Rust 侧注明「始终为空」——复用 ECL 的 Diagnostic 只是
 * 为了让前端的问题面板能同一套代码处理，别指望能从里面拿到 thmsg/thstd 的报错。
 */
export type MsgResult = EclResult
export type StdResult = EclResult

/** 对应 `modules/thdat/compiler.rs` 的 `ThdatResult`（`rename_all = "camelCase"`） */
export interface ThdatResult {
  success: boolean
  tool: string
  /** "extract" | "pack" */
  mode: string
  archivePath: string
  targetDir: string
  message: string
  /** 同样始终为空，仅为复用问题面板 */
  diagnostics: Diagnostic[]
  fileCount: number | null
}

/** 对应 `common/system_clipboard.rs` 的 `FileClipboardPayload` */
export interface FileClipboardPayload {
  paths: string[]
}

/** 对应 `common/fs_ops.rs` 的 `EntryStat`（`rename_all = "camelCase"`） */
export interface EntryStat {
  exists: boolean
  isDir: boolean
  size: number
}

/**
 * 构建对话框的表单载荷。注意它与 `TheclRequest` **不是**同一个东西：
 * 这里的 `outputPath` 用空字符串表示"自动推导"，请求构建器再把它转成 null。
 * 字段以 `src/services/toolchains/theclMetadata.ts` 的 createDefaultTheclPayload 为准。
 */
export interface TheclBuildPayload {
  tool: 'thecl'
  mode: TheclMode
  inputPath: string
  version: string
  outputPath: string
  mapPaths: string[]
  useShiftJis: boolean
  rawDump: boolean
  simpleCreation: boolean
  showOffsets: boolean
}

/**
 * 其余四个工具在注册表里 `supportsBuildDialog: false`，还没有表单，
 * 只有最小载荷。等它们真正接上构建对话框时，各自补自己的 payload 类型。
 */
export interface GenericBuildPayload {
  tool: Exclude<ToolchainId, 'thecl'>
  inputPath: string
}

export type BuildDialogPayload = TheclBuildPayload | GenericBuildPayload
