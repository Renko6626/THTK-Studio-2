import { invoke } from '@tauri-apps/api/core'

/**
 * 创建一个 PTY 会话
 * @param {{shell?: string|null, cwd?: string|null, cols: number, rows: number}} options
 * @returns {Promise<number>} sessionId
 */
export function ptyCreate({ shell = null, cwd = null, cols, rows }) {
  return invoke('pty_create', { shell, cwd, cols, rows })
}

/**
 * 向 PTY 写入数据（用户输入）
 * @param {number} sessionId
 * @param {string} data
 */
export function ptyWrite(sessionId, data) {
  return invoke('pty_write', { sessionId, data })
}

/**
 * 调整 PTY 尺寸
 * @param {number} sessionId
 * @param {number} cols
 * @param {number} rows
 */
export function ptyResize(sessionId, cols, rows) {
  return invoke('pty_resize', { sessionId, cols, rows })
}

/**
 * 终止 PTY 会话
 * @param {number} sessionId
 */
export function ptyKill(sessionId) {
  return invoke('pty_kill', { sessionId })
}
