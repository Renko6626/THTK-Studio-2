import { invoke } from '@tauri-apps/api/core'

export interface PtyCreateOptions {
  shell?: string | null
  cwd?: string | null
  cols: number
  rows: number
}

/** 创建一个 PTY 会话，返回 sessionId */
export function ptyCreate({
  shell = null,
  cwd = null,
  cols,
  rows
}: PtyCreateOptions): Promise<number> {
  return invoke('pty_create', { shell, cwd, cols, rows })
}

/**
 * 向 PTY 写入数据（用户输入）
 * @param {number} sessionId
 * @param {string} data
 */
export function ptyWrite(sessionId: number, data: string): Promise<void> {
  return invoke('pty_write', { sessionId, data })
}

/**
 * 调整 PTY 尺寸
 * @param {number} sessionId
 * @param {number} cols
 * @param {number} rows
 */
export function ptyResize(sessionId: number, cols: number, rows: number): Promise<void> {
  return invoke('pty_resize', { sessionId, cols, rows })
}

/**
 * 终止 PTY 会话
 * @param {number} sessionId
 */
export function ptyKill(sessionId: number): Promise<void> {
  return invoke('pty_kill', { sessionId })
}
