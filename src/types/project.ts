/**
 * 前后端之间的项目相关数据契约。
 *
 * 这些结构对应 Rust 侧的 `common/project_config.rs`、`common/recent_projects.rs`
 * 和 `main.rs` 的 `ProjectOpenResult`（serde `rename_all = "camelCase"`）。
 * 改这里之前先确认 Rust 那边也改了——两边对不上时 TS 只能保证前端内部自洽。
 */

export type ProjectConfigStatus = 'absent' | 'loaded' | 'invalid'

export type ProjectEncoding = 'shift-jis' | 'utf-8'

export interface ProjectToolchainConfig {
  /** 覆盖全局 thtk_dir；空字符串表示沿用全局设置 */
  thtkDir: string
}

export interface ProjectConfig {
  gameVersion: string
  encoding: ProjectEncoding
  mapPaths: string[]
  toolchain: ProjectToolchainConfig
}

/**
 * 配置加载结果。`absent` 与 `invalid` 必须可区分：把损坏当成"还没有配置"，
 * 保存动作就会静默覆盖用户手写的内容。
 */
export interface ProjectConfigLoad {
  status: ProjectConfigStatus
  value: ProjectConfig | null
  error: string | null
  /** 配置文件的绝对路径，损坏时要展示给用户 */
  path: string
}

export interface ProjectOpenResult {
  rootPath: string
  files: unknown[]
  projectConfig: ProjectConfigLoad
}

export interface RecentProjectView {
  path: string
  name: string
  /** Unix 毫秒时间戳 */
  lastOpenedAt: number
  /** 后端每次读取时现算，不落盘 */
  available: boolean
}
