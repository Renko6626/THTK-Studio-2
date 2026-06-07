import { defineStore } from 'pinia'
import {
  openTerminalSession,
  showSession,
  disposeTerminalSession
} from '../services/terminal/sessionRuntime'
import { useProjectStore } from './project'

let titleCounter = 0

export const useTerminalStore = defineStore('terminal', {
  state: () => ({
    sessions: [],          // { id, title } — xterm 实例在 sessionRuntime 模块级 Map 里
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
        const id = await openTerminalSession({ cwd: projectStore.rootPath || null })
        titleCounter += 1
        this.sessions.push({ id, title: `终端 ${titleCounter}` })
        this.setActive(id)
        return id
      } finally {
        this.pendingOpenCount -= 1
      }
    },

    setActive(id) {
      if (!this.sessions.some((session) => session.id === id)) return
      this.activeSessionId = id
      showSession(id)
    },

    async closeSession(id) {
      const index = this.sessions.findIndex((session) => session.id === id)
      if (index === -1) return
      this.sessions.splice(index, 1)
      await disposeTerminalSession(id)
      if (this.activeSessionId === id) {
        const next = this.sessions[index] || this.sessions[index - 1]
        if (next) {
          this.setActive(next.id)
        } else {
          this.activeSessionId = null
        }
      }
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
