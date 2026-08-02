import type { AppConfig, FileNode, ProjectConfig } from '../../src/types'

/**
 * 测试用的数据构造器。
 *
 * 测试通常只关心一两个字段（例如 mapPaths），但类型要求完整结构。
 * 这些工厂给出合理默认值，调用方只覆盖自己在意的部分——比在每个用例里
 * 手写全部字段可读得多，也不会因为结构新增字段就要改一堆测试。
 */

export function makeProjectConfig(overrides: Partial<ProjectConfig> = {}): ProjectConfig {
  return {
    gameVersion: '',
    encoding: 'shift-jis',
    mapPaths: [],
    toolchain: { thtkDir: '' },
    ...overrides
  }
}

export function makeAppConfig(overrides: Partial<AppConfig> = {}): AppConfig {
  return {
    thtk_dir: '',
    thecl_path: '',
    eclmap_path: '',
    tool_overrides: {},
    default_game_version: '20',
    theme: 'dark',
    mcp_port: 39127,
    recent_projects: [],
    ...overrides
  }
}

export function makeFileNode(overrides: Partial<FileNode> & Pick<FileNode, 'path'>): FileNode {
  const name = overrides.path.split(/[\\/]/).pop() || overrides.path
  return {
    name,
    is_dir: false,
    size: null,
    extension: null,
    category: 'unknown',
    // 后端对未展开目录发的是 null（Rust Option<Vec> 的 None），不是 undefined
    children: null,
    isLeaf: true,
    lossy: false,
    ...overrides
  }
}
