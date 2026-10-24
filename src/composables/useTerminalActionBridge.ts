/** 终端动作桥：只传目标会话和动作，不持有 xterm 或 PTY。 */
export type TerminalRuntimeAction = 'copy' | 'paste' | 'selectAll' | 'clear' | 'scrollBottom'
export type TerminalAction = TerminalRuntimeAction | 'find' | 'togglePanel' | 'rename' | 'close'
export interface TerminalActionRequest {
  sessionId: number
  action: TerminalAction
}
export const terminalActionEvent = 'thtk:terminal-action'

export function dispatchTerminalAction(sessionId: number, action: TerminalAction): void {
  window.dispatchEvent(new CustomEvent<TerminalActionRequest>(terminalActionEvent, {
    detail: { sessionId, action }
  }))
}

export function resolveTerminalShortcut(
  event: KeyboardEvent,
  platform: 'windows' | 'linux' | 'mac',
  hasSelection: boolean
): TerminalAction | null {
  if (event.type !== 'keydown' || event.altKey || event.isComposing) return null
  const primary = platform === 'mac'
    ? event.metaKey && !event.ctrlKey
    : event.ctrlKey && !event.metaKey
  if (!primary) return null
  const key = event.key.toLowerCase()
  if (key === 'c' && (platform === 'mac' || event.shiftKey || hasSelection)) return 'copy'
  if (key === 'v' && event.shiftKey && platform !== 'mac') return 'paste'
  if (key === 'f' && !event.shiftKey) return 'find'
  if (key === '`' && !event.shiftKey) return 'togglePanel'
  return null
}

export const terminalErrorEvent = 'thtk:terminal-error'
export function reportTerminalError(message: string): void {
  window.dispatchEvent(new CustomEvent<string>(terminalErrorEvent, { detail: message }))
}
