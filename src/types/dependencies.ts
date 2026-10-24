/**
 * 依赖状态的前后端契约。
 * 对应 `src-tauri/src/common/dependencies.rs`（`rename_all = "camelCase"`），
 * 字段集合由 `src-tauri/src/wire_format_tests.rs` 的
 * `dependency_status_is_camel_case` 钉住。
 *
 * 这是**只读**状态：依赖卡片不从这些字段写回配置。托管 thtk 安装与
 * map 快照更新属于后续里程碑，届时 `DependencySource` 才会多出
 * managed / snapshot 之类的取值。
 */

/** 当前依赖从哪来。M0/M1 只会是 none / manual / bundled。 */
export type DependencySource = 'none' | 'manual' | 'bundled'

/** thtk 的手动配置状态（`available` 只表示 thecl.exe 路径存在，未探测版本） */
export interface ThtkDependencyStatus {
  source: DependencySource
  configured: boolean
  available: boolean
  /** AppConfig.thtk_dir 的当前值，可能为空 */
  thtkDir: string
  /** 实际解析到的 thecl.exe 路径，可能为空 */
  resolvedPath: string
  message: string
  /** 许可证说明，例如 "BSD-style (retain COPYING)" */
  license: string
}

/** 托管 ECL map 的覆盖情况 */
export interface MapDependencyStatus {
  source: DependencySource
  /** 释放后的 bundled 目录绝对路径 */
  bundledDir: string
  /** 已释放、可用的版本（数字形式，如 95 表示 th095） */
  coveredVersions: number[]
  /** thecl 支持但没有内置 map 的版本 */
  missingVersions: number[]
  /** 内置基线来源 commit */
  sourceCommit: string
  license: string
}

export interface DependencyStatus {
  thtk: ThtkDependencyStatus
  maps: MapDependencyStatus
  warnings: string[]
}
