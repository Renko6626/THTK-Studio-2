// src/services/terminal/sessionRuntime.js
// 模块级终端运行时：xterm 实例与 DOM 容器脱离组件生命周期存活。
// 面板被 v-if 卸载/重建时只是重新 append 容器，PTY 会话与回显不中断。
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { listen } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'
import { ptyCreate, ptyWrite, ptyResize, ptyKill } from '../../api'

const runtimes = new Map()
// 当前面板宿主元素：面板挂载时注册；之后新建的会话容器直接 append 到这里
let currentHostEl = null

export async function openTerminalSession({ shell = null, cwd = null, onExit = null } = {}) {
  const sessionId = await ptyCreate({ shell, cwd, cols: 80, rows: 24 })

  // Track cleanup handles so the error path can roll back completely
  let unlistenOutput = null
  let unlistenExit = null
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

    term.onData((data) => {
      if (exited) return
      ptyWrite(sessionId, data).catch(() => {})
    })

    // 先挂监听，再（由 showSession）fit+resize 触发提示符重绘，规避早期输出竞态
    unlistenOutput = await listen(`pty://output/${sessionId}`, ({ payload }) => {
      term.write(payload)
    })
    unlistenExit = await listen(`pty://exit/${sessionId}`, ({ payload }) => {
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

  runtimes.set(sessionId, { term, fit, container, unlistenOutput, unlistenExit })
  return sessionId
}

export function mountAllSessions(hostEl) {
  currentHostEl = hostEl
  for (const runtime of runtimes.values()) {
    hostEl.appendChild(runtime.container)
  }
}

export function showSession(sessionId) {
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

export function fitSession(sessionId) {
  const runtime = runtimes.get(sessionId)
  if (!runtime || runtime.container.style.display === 'none') return
  if (!runtime.container.clientWidth || !runtime.container.clientHeight) return
  runtime.fit.fit()
  ptyResize(sessionId, runtime.term.cols, runtime.term.rows).catch(() => {})
}

export async function disposeTerminalSession(sessionId) {
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
