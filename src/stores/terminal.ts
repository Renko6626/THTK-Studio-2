import { defineStore } from 'pinia'
import {
  openTerminalSession,
  showSession,
  disposeTerminalSession
} from '../services/terminal/sessionRuntime'
import { useProjectStore } from './project'
import { useWorkbenchReportsStore } from './workbenchReports'

export interface TerminalSessionInfo {
  id: number
  title: string
  exited: boolean
}

interface TerminalState {
  /** xterm 实例在 sessionRuntime 的模块级 Map 里，这里只存展示用的元信息 */
  sessions: TerminalSessionInfo[]
  activeSessionId: number | null
  /** 创建中(ptyCreate 未返回)的会话数，供自动开启去重 */
  pendingOpenCount: number
}

export interface OpenSessionOptions {
  /** 指定 shell(如 "cmd.exe")；null = 后端自动探测 */
  shell?: string | null
  /** tab 标题里展示的 shell 名(如 "cmd") */
  label?: string | null
}

let titleCounter = 0

export const useTerminalStore = defineStore('terminal', {
  state: (): TerminalState => ({
    sessions: [],
    activeSessionId: null,
    pendingOpenCount: 0
  }),

  getters: {
    sessionCount: (state): number => state.sessions.length,
    activeSession: (state): TerminalSessionInfo | null =>
      state.sessions.find((session) => session.id === state.activeSessionId) || null
  },

  actions: {
    /**
     * 打开终端会话。
     * @param shell 指定 shell(如 "cmd.exe");null = 后端自动探测
     * @param label tab 标题里展示的 shell 名(如 "cmd")
     */
    async openSession({ shell = null, label = null }: OpenSessionOptions = {}) {
      const projectStore = useProjectStore()
      const cwd = projectStore.rootPath || null
      this.pendingOpenCount += 1
      try {
        let id
        let effectiveLabel = label
        try {
          id = await openTerminalSession({
            shell,
            cwd,
            onExit: (sessionId) => this.markExited(sessionId)
          })
        } catch (error) {
          if (!shell) throw error
          // 指定 shell 启动失败 → 回退到默认探测,并告知用户
          useWorkbenchReportsStore().publishToolResult({
            ownerKey: 'terminal:shell-fallback',
            source: 'terminal',
            operation: 'open',
            scriptKind: 'shell',
            title: `"${label || shell}" 启动失败,已回退默认 shell`,
            path: null,
            success: false,
            message: String(error),
            diagnostics: []
          })
          effectiveLabel = null
          id = await openTerminalSession({
            shell: null,
            cwd,
            onExit: (sessionId) => this.markExited(sessionId)
          })
        }
        titleCounter += 1
        const title = effectiveLabel
          ? `终端 ${titleCounter} (${effectiveLabel})`
          : `终端 ${titleCounter}`
        this.sessions.push({ id, title, exited: false })
        this.setActive(id)
        return id
      } catch (error) {
        useWorkbenchReportsStore().publishToolResult({
          ownerKey: 'terminal:open-failed',
          source: 'terminal',
          operation: 'open',
          scriptKind: 'shell',
          title: '打开终端失败',
          path: null,
          success: false,
          message: String(error),
          diagnostics: []
        })
        // swallow after reporting
      } finally {
        this.pendingOpenCount -= 1
      }
    },

    setActive(id: number) {
      if (!this.sessions.some((session) => session.id === id)) return
      this.activeSessionId = id
      showSession(id)
    },

    markExited(id: number) {
      const session = this.sessions.find((s) => s.id === id)
      if (session) session.exited = true
    },

    async closeSession(id: number) {
      const index = this.sessions.findIndex((session) => session.id === id)
      if (index === -1) return
      // Determine and activate the next session BEFORE the async dispose so the
      // UI never points at a removed session during the IPC round-trip.
      if (this.activeSessionId === id) {
        this.sessions.splice(index, 1)
        const next = this.sessions[index] || this.sessions[index - 1]
        if (next) {
          this.setActive(next.id)
        } else {
          this.activeSessionId = null
        }
      } else {
        this.sessions.splice(index, 1)
      }
      await disposeTerminalSession(id)
    },

    // ---- 会话快照兼容层（useWorkbenchSession 仍会调用） ----
    // PTY 会话本质上不可跨刷新恢复，所以这三个是刻意的空实现。
    // 形参照旧声明出来：调用方确实在传值，签名写成零参会让每个调用点报错，
    // 而真正该改的是"要不要保留这层兼容"，不是调用点。
    hydrate(_snapshot?: unknown) {},
    toSnapshot(): Record<string, never> {
      return {}
    },
    setWorkingDirectory(_path?: string) {}
  }
})
