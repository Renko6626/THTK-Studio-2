import { invoke } from '@tauri-apps/api/core'
import type {
  AppConfig,
  ProjectConfig,
  ProjectConfigLoad,
  RecentProjectView,
  ToolchainStatus
} from '../../types'

/**
 * 获取用户设置
 */
export function getSettings(): Promise<AppConfig> {
  return invoke('get_settings')
}

/**
 * 保存用户设置
 * @param {Object} config 
 */
export function saveSettings(config: AppConfig): Promise<void> {
  return invoke('save_settings', { config })
}

export function getToolchainStatus(tool: string): Promise<ToolchainStatus> {
  return invoke('get_toolchain_status', { tool })
}

export function getToolchainStatuses(): Promise<ToolchainStatus[]> {
  return invoke('get_toolchain_statuses')
}

/**
 * 加载项目配置 (.thtk-project.json)
 * @returns {Promise<{status: 'absent'|'loaded'|'invalid', value: Object|null, error: string|null, path: string}>}
 *   status 区分"没有配置文件"与"文件损坏"；后者不经确认不得覆盖。
 */
export function loadProjectConfig(): Promise<ProjectConfigLoad> {
  return invoke('load_project_config')
}

/**
 * 保存项目配置。
 * @param {Object} config
 * @param {string} expectedRoot 打开表单时所编辑的项目根。后端会与当前项目根比对，
 *   不一致直接拒绝——防止对话框开着时项目被切换，把 A 的配置写进 B。
 */
export function saveProjectConfig(config: ProjectConfig, expectedRoot: string): Promise<void> {
  return invoke('save_project_config_cmd', { config, expectedRoot })
}

/**
 * 最近打开的项目，按最近打开时间降序，最多 10 条。
 * 不会自动剔除失效路径——磁盘临时离线不该让记录消失，由用户决定删不删；
 * `available` 是后端每次读取时现算的，不落盘。
 * @returns {Promise<Array<{path: string, name: string, lastOpenedAt: number, available: boolean}>>}
 */
export function listRecentProjects(): Promise<RecentProjectView[]> {
  return invoke('list_recent_projects')
}

/** 移除一条最近项目，返回移除后的列表 */
export function removeRecentProject(path: string): Promise<RecentProjectView[]> {
  return invoke('remove_recent_project', { path })
}

export function clearRecentProjects(): Promise<void> {
  return invoke('clear_recent_projects')
}
