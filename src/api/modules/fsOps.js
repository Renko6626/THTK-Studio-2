import { invoke } from '@tauri-apps/api/core'

/**
 * 保存文件
 * @param {string} path - 路径
 * @param {string} content - 内容
 * @param {boolean} isSource - true=保存为UTF8, false=保存为ShiftJIS
 */
export function saveFile(path, content, isSource = true) {
  return invoke('save_file', { path, content, isSource })
}

/**
 * 创建文件夹
 * @param {string} path 
 */
export function createDir(path) {
  return invoke('create_directory', { path })
}

/**
 * 创建空文件
 * @param {string} path 
 */
export function createFile(path) {
  return invoke('create_file', { path })
}

/**
 * 重命名文件/文件夹
 * @param {string} oldPath 
 * @param {string} newPath 
 */
export function renameEntry(oldPath, newPath) {
  return invoke('rename_entry', { oldPath, newPath })
}

/**
 * 复制文件/文件夹
 * @param {string} sourcePath
 * @param {string} destinationPath
 */
export function copyEntry(sourcePath, destinationPath) {
  return invoke('copy_entry', { sourcePath, destinationPath })
}

/**
 * 写入系统文件剪贴板
 * @param {string[]} paths
 */
export function setFileClipboard(paths) {
  return invoke('set_file_clipboard', { paths })
}

/**
 * 读取系统文件剪贴板
 */
export function getFileClipboard() {
  return invoke('get_file_clipboard')
}

/**
 * 删除文件/文件夹
 * @param {string} path 
 */
export function deleteEntry(path) {
  return invoke('delete_entry', { path })
}
