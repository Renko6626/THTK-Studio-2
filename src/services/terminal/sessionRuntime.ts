// src/services/terminal/sessionRuntime.ts
// 模块级终端运行时：xterm 实例与 DOM 容器脱离组件生命周期存活。
// 面板被 v-if 卸载/重建时只是重新 append 容器，PTY 会话与回显不中断。
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { listen } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { ptyCreate, ptyWrite, ptyResize, ptyKill } from '../../api'

interface SessionRuntime {
  term: Terminal
  fit: FitAddon
  container: HTMLDivElement
  unlistenOutput: UnlistenFn
  unlistenExit: UnlistenFn
}

/** PTY 退出回调；code 为 null 表示后端没拿到退出码 */
export type SessionExitHandler = (sessionId: number, code: number | null) => void

export interface OpenTerminalSessionOptions {
  shell?: string | null
  cwd?: string | null
  onExit?: SessionExitHandler | null
}

/** 后端 pty://exit 事件的负载 */
interface PtyExitPayload {
  code?: number | null
}

const runtimes = new Map<number, SessionRuntime>()
// 当前面板宿主元素：面板挂载时注册；之后新建的会话容器直接 append 到这里
let currentHostEl: HTMLElement | null = null

export async function openTerminalSession({
  shell = null,
  cwd = null,
  onExit = null
}: OpenTerminalSessionOptions = {}): Promise<number> {
  const sessionId = await ptyCreate({ shell, cwd, cols: 80, rows: 24 })

  // Track cleanup handles so the error path can roll back completely
  let unlistenOutput: UnlistenFn | null = null
  let unlistenExit: UnlistenFn | null = null
  let exited = false

  const term = new Terminal({
    scrollback: 5000,
    fontSize: 13,
    fontFamily: 'Consolas, "Courier New", monospace',
    cursorBlink: true,
    theme: { background: '#111111' }
  })
  const fit = new FitAddon()
  term.loadAddon(fit)

  const container = document.createElement('div')
  container.className = 'absolute inset-0'
  container.style.display = 'none'
  if (currentHostEl?.isConnected) {
    currentHostEl.appendChild(container)
  }

  try {
    term.open(container)

    term.onData((data: string) => {
      if (exited) return
      ptyWrite(sessionId, data).catch(() => {})
    })

    // 先挂监听，再（由 showSession）fit+resize 触发提示符重绘，规避早期输出竞态
    unlistenOutput = await listen<string>(`pty://output/${sessionId}`, ({ payload }) => {
      term.write(payload)
    })
    unlistenExit = await listen<PtyExitPayload>(`pty://exit/${sessionId}`, ({ payload }) => {
      exited = true
      const code = payload?.code
      term.write(`\r\n\x1b[90m[进程已退出${code != null ? `，code ${code}` : ''}]\x1b[0m\r\n`)
      onExit?.(sessionId, payload?.code ?? null)
    })
  } catch (err) {
    // Roll back everything so we don't leak a live PTY or orphaned Terminal
    if (unlistenOutput) unlistenOutput()
    if (unlistenExit) unlistenExit()
    term.dispose()
    container.remove()
    ptyKill(sessionId).catch(() => {})
    throw err
  }

  // 到这里两个 unlisten 一定已经赋值：上面的 try 要么全部成功，要么抛出
  runtimes.set(sessionId, {
    term,
    fit,
    container,
    unlistenOutput: unlistenOutput!,
    unlistenExit: unlistenExit!
  })
  return sessionId
}

export function mountAllSessions(hostEl: HTMLElement): void {
  currentHostEl = hostEl
  for (const runtime of runtimes.values()) {
    hostEl.appendChild(runtime.container)
  }
}

export function showSession(sessionId: number): void {
  for (const [id, runtime] of runtimes) {
    runtime.container.style.display = id === sessionId ? 'block' : 'none'
  }
  const runtime = runtimes.get(sessionId)
  if (!runtime) return
  // display 切换后等一帧再 fit，否则容器尺寸还是 0
  requestAnimationFrame(() => {
    if (!runtimes.has(sessionId)) return
    if (!runtime.container.clientWidth || !runtime.container.clientHeight) return
    runtime.fit.fit()
    ptyResize(sessionId, runtime.term.cols, runtime.term.rows).catch(() => {})
    runtime.term.focus()
  })
}

export function fitSession(sessionId: number): void {
  const runtime = runtimes.get(sessionId)
  if (!runtime || runtime.container.style.display === 'none') return
  if (!runtime.container.clientWidth || !runtime.container.clientHeight) return
  runtime.fit.fit()
  ptyResize(sessionId, runtime.term.cols, runtime.term.rows).catch(() => {})
}

export async function disposeTerminalSession(sessionId: number): Promise<void> {
  const runtime = runtimes.get(sessionId)
  if (!runtime) return
  runtimes.delete(sessionId)
  runtime.unlistenOutput()
  runtime.unlistenExit()
  runtime.term.dispose()
  runtime.container.remove()
  try {
    await ptyKill(sessionId)
  } catch {
    // 进程可能已自然退出且后端已自清理，忽略
  }
}
