import { defineStore } from 'pinia'
import {
  openTerminalSession,
  showSession,
  disposeTerminalSession
} from '../services/terminal/sessionRuntime'
import { useProjectStore } from './project'
import { useWorkbenchReportsStore } from './workbenchReports'

let titleCounter = 0

export const useTerminalStore = defineStore('terminal', {
  state: () => ({
    sessions: [],          // { id, title, exited } — xterm 实例在 sessionRuntime 模块级 Map 里
    activeSessionId: null,
    pendingOpenCount: 0    // 创建中(ptyCreate 未返回)的会话数，供自动开启去重
  }),

  getters: {
    sessionCount: (state) => state.sessions.length,
    activeSession: (state) =>
      state.sessions.find((session) => session.id === state.activeSessionId) || null
  },

  actions: {
    async openSession() {
      const projectStore = useProjectStore()
      this.pendingOpenCount += 1
      try {
        const id = await openTerminalSession({
          cwd: projectStore.rootPath || null,
          onExit: (sessionId) => this.markExited(sessionId)
        })
        titleCounter += 1
        this.sessions.push({ id, title: `终端 ${titleCounter}`, exited: false })
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

    setActive(id) {
      if (!this.sessions.some((session) => session.id === id)) return
      this.activeSessionId = id
      showSession(id)
    },

    markExited(id) {
      const session = this.sessions.find((s) => s.id === id)
      if (session) session.exited = true
    },

    async closeSession(id) {
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
    // PTY 会话本质上不可跨刷新恢复，保留空实现。
    hydrate() {},
    toSnapshot() {
      return {}
    },
    setWorkingDirectory() {}
  }
})
