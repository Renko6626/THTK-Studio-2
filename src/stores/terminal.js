import { defineStore } from 'pinia'
import { resolveDirectory, runShellCommand } from '../api'
import { useWorkbenchReportsStore } from './workbenchReports'

function createLine(type, text) {
  return {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    type,
    text
  }
}

function normalizeCdTarget(rawTarget) {
  const trimmed = rawTarget.trim()
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1)
  }
  return trimmed
}

export const useTerminalStore = defineStore('terminal', {
  state: () => ({
    shell: 'pwsh',
    cwd: '',
    isRunning: false,
    history: [],
    historyIndex: -1,
    lines: [
      createLine('system', 'Integrated shell ready.'),
      createLine('system', 'Built-ins: cd, pwd, cls, clear.')
    ]
  }),

  getters: {
    promptLabel: (state) => `${state.shell} ${state.cwd || '~'}>`
  },

  actions: {
    hydrate(snapshot) {
      if (!snapshot) return
      if (snapshot.shell) this.shell = snapshot.shell
      if (snapshot.cwd) this.cwd = snapshot.cwd
    },

    setWorkingDirectory(path) {
      if (path) {
        this.cwd = path
      }
    },

    clearOutput() {
      this.lines = []
    },

    pushLine(type, text) {
      if (text === undefined || text === null) return
      const normalized = String(text)
      const segments = normalized.replace(/\r\n/g, '\n').split('\n')
      segments.forEach((segment) => {
        if (segment.length === 0) {
          this.lines.push(createLine(type, ''))
        } else {
          this.lines.push(createLine(type, segment))
        }
      })
    },

    navigateHistory(direction, currentValue = '') {
      if (!this.history.length) return currentValue

      if (direction === 'up') {
        if (this.historyIndex === -1) {
          this.historyIndex = this.history.length - 1
        } else {
          this.historyIndex = Math.max(0, this.historyIndex - 1)
        }
        return this.history[this.historyIndex]
      }

      if (direction === 'down') {
        if (this.historyIndex === -1) return currentValue
        if (this.historyIndex >= this.history.length - 1) {
          this.historyIndex = -1
          return ''
        }
        this.historyIndex += 1
        return this.history[this.historyIndex]
      }

      return currentValue
    },

    async execute(command) {
      const reportsStore = useWorkbenchReportsStore()
      const raw = command.trim()
      if (!raw || this.isRunning) return

      this.history.push(raw)
      this.historyIndex = -1
      this.pushLine('prompt', `${this.promptLabel} ${raw}`)

      if (raw === 'cls' || raw === 'clear') {
        this.clearOutput()
        return
      }

      if (raw === 'pwd') {
        this.pushLine('stdout', this.cwd || '.')
        return
      }

      if (raw === 'cd' || raw.startsWith('cd ')) {
        const target = raw === 'cd' ? '.' : normalizeCdTarget(raw.slice(3))
        try {
          const nextDir = await resolveDirectory(this.cwd || null, target)
          this.cwd = nextDir
          this.pushLine('system', this.cwd)
        } catch (error) {
          this.pushLine('stderr', String(error))
        }
        return
      }

      this.isRunning = true
      try {
        const result = await runShellCommand(this.shell, raw, this.cwd || null)
        reportsStore.publishToolResult({
          ownerKey: `terminal:${Date.now()}`,
          source: 'terminal',
          operation: 'run',
          scriptKind: 'shell',
          title: raw,
          path: this.cwd || null,
          success: Boolean(result.success),
          message: [result.stdout, result.stderr].filter(Boolean).join('\n'),
          diagnostics: []
        })
        if (result.cwd) {
          this.cwd = result.cwd
        }
        if (result.stdout) this.pushLine('stdout', result.stdout)
        if (result.stderr) this.pushLine('stderr', result.stderr)
        if (!result.stdout && !result.stderr) {
          this.pushLine('system', `Process exited with code ${result.exit_code ?? 0}`)
        }
      } catch (error) {
        this.pushLine('stderr', String(error))
      } finally {
        this.isRunning = false
      }
    },

    toSnapshot() {
      return {
        shell: this.shell,
        cwd: this.cwd
      }
    }
  }
})
