import { defineStore } from 'pinia'

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

      this.outputEntries = [
        ...this.outputEntries.filter(item => item.ownerKey !== ownerKey),
        ...nextEntries
      ]
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

      this.problemEntries = [
        ...this.problemEntries.filter(item => item.ownerKey !== ownerKey),
        ...normalized
      ]
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
