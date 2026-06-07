import { invoke } from '@tauri-apps/api/core'

/**
 * 获取文件树（浅层，只读一层目录）
 * @param {string} path - 根目录路径
 */
export function getFileTree(path) {
  return invoke('get_file_tree', { path })
}

/**
 * 按需加载目录的子节点
 * @param {string} path - 目录路径
 */
export function getDirChildren(path) {
  return invoke('get_dir_children', { path })
}

/**
 * 设置当前项目根目录
 * @param {string} path
 */
export function setProjectRoot(path) {
  return invoke('set_project_root', { path })
}

/**
 * 读取文件内容
 * @param {string} path - 文件路径
 */
export function readFile(path) {
  return invoke('read_file', { path })
}
