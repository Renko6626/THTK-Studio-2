import { invoke } from '@tauri-apps/api/core'
import type { FileNode, ProjectOpenResult } from '../../types'

/**
 * 获取文件树（浅层，只读一层目录）
 * @param {string} path - 根目录路径
 */
export function getFileTree(path: string): Promise<FileNode[]> {
  return invoke('get_file_tree', { path })
}

/**
 * 按需加载目录的子节点
 * @param {string} path - 目录路径
 */
export function getDirChildren(path: string): Promise<FileNode[]> {
  return invoke('get_dir_children', { path })
}

/**
 * 事务式打开项目：目录验证与首层扫描都成功后，后端才提交项目根、切换文件监听、
 * 注册 MCP 客户端并记录最近项目。任一步失败都会 reject 且不改动当前工作区。
 * @param {string} path
 * @returns {Promise<{rootPath: string, files: Object[], projectConfig: {status: 'absent'|'loaded'|'invalid', value: Object|null, error: string|null, path: string}}>}
 */
export function openProject(path: string): Promise<ProjectOpenResult> {
  return invoke('open_project', { path })
}

/**
 * 读取文件内容
 * @param {string} path - 文件路径
 */
export function readFile(path: string): Promise<string> {
  return invoke('read_file', { path })
}
