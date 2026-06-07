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
 * @returns {Promise<Object|null>} 项目配置或 null（不存在时）
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
