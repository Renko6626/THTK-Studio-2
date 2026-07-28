import { invoke } from '@tauri-apps/api/core'

/**
 * 获取用户设置
 */
export function getSettings() {
  return invoke('get_settings')
}

/**
 * 保存用户设置
 * @param {Object} config 
 */
export function saveSettings(config) {
  return invoke('save_settings', { config })
}

/**
 * 获取当前 thecl 可用性与版本信息
 */
export function getTheclStatus() {
  return invoke('get_thecl_status')
}

export function getToolchainStatus(tool) {
  return invoke('get_toolchain_status', { tool })
}

export function getToolchainStatuses() {
  return invoke('get_toolchain_statuses')
}

/**
 * 加载项目配置 (.thtk-project.json)
 * @returns {Promise<{status: 'absent'|'loaded'|'invalid', value: Object|null, error: string|null, path: string}>}
 *   status 区分"没有配置文件"与"文件损坏"；后者不经确认不得覆盖。
 */
export function loadProjectConfig() {
  return invoke('load_project_config')
}

/**
 * 保存项目配置
 * @param {Object} config
 */
export function saveProjectConfig(config) {
  return invoke('save_project_config_cmd', { config })
}

/**
 * 最近打开的项目，按最近打开时间降序，最多 10 条。
 * 不会自动剔除失效路径——磁盘临时离线不该让记录消失，由用户决定删不删。
 * @returns {Promise<Array<{path: string, name: string, lastOpenedAt: number}>>}
 */
export function listRecentProjects() {
  return invoke('list_recent_projects')
}

/** 移除一条最近项目，返回移除后的列表 */
export function removeRecentProject(path) {
  return invoke('remove_recent_project', { path })
}

export function clearRecentProjects() {
  return invoke('clear_recent_projects')
}
