import { defineStore } from 'pinia'

// 输出与问题条目原先无上限，长时间使用会单调增长（终端侧已用 scrollback 上限解决）。
// 超出时丢弃最旧的：输出面板里越新的结果越可能是用户正在看的那一次。
// 代价是单次超长的工具输出会丢掉开头几行——这比整个会话吃光内存要好。
const MAX_OUTPUT_ENTRIES = 5000
const MAX_PROBLEM_ENTRIES = 2000

/** 保留最近的 limit 条，返回新数组 */
function capEntries(entries, limit) {
  return entries.length > limit ? entries.slice(entries.length - limit) : entries
}

function createId(prefix) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function normalizeLevel(level) {
  return ['error', 'warning', 'info', 'success'].includes(level) ? level : 'info'
}

function normalizeOwnerKey(entry) {
  return entry.ownerKey || `${entry.source || 'system'}:${entry.operation || 'general'}:${entry.path || 'workspace'}`
}

function toOutputLines(text) {
  return String(text ?? '')
    .replace(/\r\n/g, '\n')
    .split('\n')
}

export const useWorkbenchReportsStore = defineStore('workbenchReports', {
  state: () => ({
    outputEntries: [],
    problemEntries: []
  }),

  getters: {
    errorCount: (state) => state.problemEntries.filter(item => item.severity === 'error').length,
    warningCount: (state) => state.problemEntries.filter(item => item.severity === 'warning').length,
    outputGroups: (state) => {
      const groups = new Map()

      state.outputEntries.forEach((entry) => {
        const key = entry.ownerKey
        if (!groups.has(key)) {
          groups.set(key, {
            ownerKey: key,
            source: entry.source,
            operation: entry.operation,
            scriptKind: entry.scriptKind,
            title: entry.title,
            path: entry.path,
            timestamp: entry.timestamp,
            level: entry.level,
            lines: []
          })
        }

        const group = groups.get(key)
        group.timestamp = Math.max(group.timestamp, entry.timestamp)
        group.level = entry.level === 'error' ? 'error' : group.level
        group.lines.push(entry)
      })

      return [...groups.values()].sort((a, b) => b.timestamp - a.timestamp)
    }
  },

  actions: {
    clearOutput() {
      this.outputEntries = []
    },

    clearProblems() {
      this.problemEntries = []
    },

    clearOwner(ownerKey) {
      this.outputEntries = this.outputEntries.filter(item => item.ownerKey !== ownerKey)
      this.problemEntries = this.problemEntries.filter(item => item.ownerKey !== ownerKey)
    },

    pushOutputEntry(entry) {
      this.outputEntries.push({
        id: createId('output'),
        ownerKey: normalizeOwnerKey(entry),
        timestamp: Date.now(),
        source: entry.source || 'system',
        operation: entry.operation || 'general',
        scriptKind: entry.scriptKind || 'text',
        level: normalizeLevel(entry.level),
        title: entry.title || '',
        path: entry.path || null,
        text: entry.text || ''
      })

      // 只在越界时裁掉溢出的部分，通常一次一条
      if (this.outputEntries.length > MAX_OUTPUT_ENTRIES) {
        this.outputEntries.splice(0, this.outputEntries.length - MAX_OUTPUT_ENTRIES)
      }
    },

    replaceOutput(ownerKey, entries) {
      const nextEntries = (entries || []).map((entry) => ({
        id: createId('output'),
        ownerKey,
        timestamp: Date.now(),
        source: entry.source || 'system',
        operation: entry.operation || 'general',
        scriptKind: entry.scriptKind || 'text',
        level: normalizeLevel(entry.level),
        title: entry.title || '',
        path: entry.path || null,
        text: entry.text || ''
      }))

      this.outputEntries = capEntries(
        [...this.outputEntries.filter(item => item.ownerKey !== ownerKey), ...nextEntries],
        MAX_OUTPUT_ENTRIES
      )
    },

    pushOutputText(entry) {
      toOutputLines(entry.text).forEach((line) => {
        this.pushOutputEntry({
          ...entry,
          text: line
        })
      })
    },

    replaceProblems(ownerKey, problems) {
      const normalized = (problems || []).map((problem) => ({
        id: createId('problem'),
        ownerKey,
        source: problem.source || 'system',
        operation: problem.operation || 'general',
        scriptKind: problem.scriptKind || 'text',
        path: problem.path || null,
        line: Number(problem.line || 1),
        column: Number(problem.column || 1),
        severity: normalizeLevel(problem.severity || 'error'),
        message: problem.message || 'Unknown issue'
      }))

      this.problemEntries = capEntries(
        [...this.problemEntries.filter(item => item.ownerKey !== ownerKey), ...normalized],
        MAX_PROBLEM_ENTRIES
      )
    },

    publishToolResult({
      ownerKey,
      source = 'toolchain',
      operation = 'general',
      scriptKind = 'text',
      title = '',
      path = null,
      success = true,
      message = '',
      diagnostics = []
    }) {
      const normalizedOwnerKey = ownerKey || `${source}:${operation}:${path || 'workspace'}`

      if (message) {
        const outputLines = toOutputLines(message).map((line) => ({
          ownerKey: normalizedOwnerKey,
          source,
          operation,
          scriptKind,
          title,
          path,
          level: success ? 'info' : 'error',
          text: line
        }))

        this.replaceOutput(normalizedOwnerKey, outputLines)
      } else {
        this.replaceOutput(normalizedOwnerKey, [])
      }

      this.replaceProblems(
        normalizedOwnerKey,
        diagnostics.map((diagnostic) => ({
          source,
          operation,
          scriptKind,
          path: diagnostic.path || path,
          line: diagnostic.line,
          column: diagnostic.column,
          severity: diagnostic.severity,
          message: diagnostic.message
        }))
      )
    }
  }
})
