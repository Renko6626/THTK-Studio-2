/**
 * 文件系统相关的前后端契约。
 * 对应 `src-tauri/src/common/fs_utils.rs`。
 */

/** 对应 fs_utils.rs 的 `FileCategory`（`rename_all = "camelCase"`） */
export type FileCategory =
  | 'sourceScript'
  | 'binaryScript'
  | 'archive'
  | 'image'
  | 'assetDefinition'
  | 'directory'
  | 'unknown'

/**
 * 对应 fs_utils.rs 的 `FileNode`。
 *
 * ⚠️ 该结构体**没有** `rename_all`，所以字段是 snake_case——唯独 `is_leaf` 带了
 * `#[serde(rename = "isLeaf")]`。这个不一致是历史遗留，不要在类型里"顺手统一"，
 * 否则运行时字段名对不上。
 */
export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  /** 目录为 null */
  size: number | null
  extension: string | null
  category: FileCategory
  /** 仅目录有；未加载时为 null/undefined，前端据此做懒加载 */
  children?: FileNode[] | null
  isLeaf: boolean
  /** 文件名含非 UTF-8 字节，IDE 操作可能失败 */
  lossy: boolean
}

/**
 * 对应 `common/file_watcher.rs` 的 `FileChangeEvent`（`rename_all = "camelCase"`）。
 *
 * `kind` 在 Rust 侧是裸 String，只会产出这两个值：
 * - `modify` — 处理时路径仍存在（涵盖新建与修改，文件和目录都算）
 * - `remove` — 处理时路径已不存在
 */
export interface FileChangeEvent {
  path: string
  kind: 'modify' | 'remove'
}
