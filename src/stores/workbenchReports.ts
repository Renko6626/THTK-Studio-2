import { defineStore } from 'pinia'
import type { Diagnostic } from '../types'

export type ReportLevel = 'error' | 'warning' | 'info' | 'success'

export interface OutputEntry {
  id: string
  ownerKey: string
  timestamp: number
  source: string
  operation: string
  scriptKind: string
  level: ReportLevel
  title: string
  path: string | null
  text: string
}

export interface ProblemEntry {
  id: string
  ownerKey: string
  source: string
  operation: string
  scriptKind: string
  path: string | null
  line: number
  column: number
  severity: ReportLevel
  message: string
}

/** outputGroups getter 的产物：同一 ownerKey 的输出行聚成一组 */
export interface OutputGroup {
  ownerKey: string
  source: string
  operation: string
  scriptKind: string
  title: string
  path: string | null
  timestamp: number
  level: ReportLevel
  lines: OutputEntry[]
}

/** 写入侧的入参一律是部分字段，缺的由 store 补默认值 */
export interface OutputEntryInput {
  ownerKey?: string
  source?: string
  operation?: string
  scriptKind?: string
  level?: string
  title?: string
  path?: string | null
  text?: string
}

export interface ProblemInput {
  source?: string
  operation?: string
  scriptKind?: string
  path?: string | null
  line?: number | null
  column?: number | null
  severity?: string
  message?: string
}

export interface PublishToolResultInput {
  ownerKey?: string
  source?: string
  operation?: string
  scriptKind?: string
  title?: string
  path?: string | null
  success?: boolean
  message?: string
  diagnostics?: Diagnostic[]
}

interface WorkbenchReportsState {
  outputEntries: OutputEntry[]
  problemEntries: ProblemEntry[]
}

// 输出与问题条目原先无上限，长时间使用会单调增长（终端侧已用 scrollback 上限解决）。
// 超出时丢弃最旧的：输出面板里越新的结果越可能是用户正在看的那一次。
// 代价是单次超长的工具输出会丢掉开头几行——这比整个会话吃光内存要好。
const MAX_OUTPUT_ENTRIES = 5000
const MAX_PROBLEM_ENTRIES = 2000

/** 保留最近的 limit 条，返回新数组 */
function capEntries<T>(entries: T[], limit: number): T[] {
  return entries.length > limit ? entries.slice(entries.length - limit) : entries
}

function createId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function normalizeLevel(level: string | undefined): ReportLevel {
  return (['error', 'warning', 'info', 'success'] as const).includes(level as ReportLevel)
    ? (level as ReportLevel)
    : 'info'
}

function normalizeOwnerKey(entry: OutputEntryInput): string {
  return entry.ownerKey || `${entry.source || 'system'}:${entry.operation || 'general'}:${entry.path || 'workspace'}`
}

function toOutputLines(text: unknown): string[] {
  return String(text ?? '')
    .replace(/\r\n/g, '\n')
    .split('\n')
}

export const useWorkbenchReportsStore = defineStore('workbenchReports', {
  state: (): WorkbenchReportsState => ({
    outputEntries: [],
    problemEntries: []
  }),

  getters: {
    errorCount: (state): number => state.problemEntries.filter(item => item.severity === 'error').length,
    warningCount: (state): number => state.problemEntries.filter(item => item.severity === 'warning').length,
    outputGroups: (state): OutputGroup[] => {
      const groups = new Map<string, OutputGroup>()

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

        const group = groups.get(key)!
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

    clearOwner(ownerKey: string) {
      this.outputEntries = this.outputEntries.filter(item => item.ownerKey !== ownerKey)
      this.problemEntries = this.problemEntries.filter(item => item.ownerKey !== ownerKey)
    },

    pushOutputEntry(entry: OutputEntryInput) {
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

    replaceOutput(ownerKey: string, entries: OutputEntryInput[] | null | undefined) {
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

    pushOutputText(entry: OutputEntryInput) {
      toOutputLines(entry.text).forEach((line) => {
        this.pushOutputEntry({
          ...entry,
          text: line
        })
      })
    },

    replaceProblems(ownerKey: string, problems: ProblemInput[] | null | undefined) {
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
    }: PublishToolResultInput) {
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
