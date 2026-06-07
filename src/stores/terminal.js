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
    /**
     * 打开终端会话。
     * @param shell 指定 shell(如 "cmd.exe");null = 后端自动探测
     * @param label tab 标题里展示的 shell 名(如 "cmd")
     */
    async openSession({ shell = null, label = null } = {}) {
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
