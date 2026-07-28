import type { FileCategory } from '../types'

export interface SnapshotStorageKeys {
  project: string
  editor: string
  terminal: string
}

const STORAGE_KEYS: SnapshotStorageKeys = {
  project: 'thtk-studio:project',
  editor: 'thtk-studio:editor',
  terminal: 'thtk-studio:terminal'
}

export interface ProjectSnapshot {
  rootPath: string | null
}

/** 单个标签的持久化形态，由 editor store 的 toSnapshot() 产出 */
export interface EditorTabSnapshot {
  path: string
  name: string
  isDirty: boolean
  language: string
  viewType: 'text' | 'binary-script'
  size: number | null
  extension: string | null
  category: FileCategory | null
  /** 仅脏的文本标签、且未超出草稿预算时才落盘 */
  content?: string
  originalContent?: string
}

export interface EditorSnapshot {
  activePath: string | null
  tabs: EditorTabSnapshot[]
}

export interface TerminalSnapshot {
  sessions?: unknown[]
  activeId?: string | null
}

const pendingTimers = new Map<string, number>()

/**
 * localStorage 里的内容是**不可信**的：可能是上一个版本写的旧结构，也可能被用户
 * 手改过。这里的类型断言只表达"我们期望它是什么"，消费方仍然要逐字段容错——
 * 例如 editorStore.restoreSession 对每个标签都做了自己的校验。
 */
function loadJson<T>(key: string, fallback: T): T {
  try {
    const raw = window.localStorage.getItem(key)
    return raw ? (JSON.parse(raw) as T) : fallback
  } catch {
    return fallback
  }
}

function saveJson(key: string, value: unknown): void {
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Ignore storage failures for now.
  }
}

export function loadProjectSnapshot(): ProjectSnapshot | null {
  return loadJson<ProjectSnapshot | null>(STORAGE_KEYS.project, null)
}

export function saveProjectSnapshot(snapshot: ProjectSnapshot): void {
  saveJson(STORAGE_KEYS.project, snapshot)
}

export function loadEditorSnapshot(): EditorSnapshot | null {
  return loadJson<EditorSnapshot | null>(STORAGE_KEYS.editor, null)
}

export function saveEditorSnapshot(snapshot: EditorSnapshot): void {
  saveJson(STORAGE_KEYS.editor, snapshot)
}

export function loadTerminalSnapshot(): TerminalSnapshot | null {
  return loadJson<TerminalSnapshot | null>(STORAGE_KEYS.terminal, null)
}

export function saveTerminalSnapshot(snapshot: TerminalSnapshot): void {
  saveJson(STORAGE_KEYS.terminal, snapshot)
}

export function scheduleSnapshotSave(key: string, snapshot: unknown, delay = 300): void {
  if (pendingTimers.has(key)) {
    window.clearTimeout(pendingTimers.get(key))
  }

  const timer = window.setTimeout(() => {
    saveJson(key, snapshot)
    pendingTimers.delete(key)
  }, delay)

  pendingTimers.set(key, timer)
}

export function flushSnapshotSave(key: string, snapshot: unknown): void {
  if (pendingTimers.has(key)) {
    window.clearTimeout(pendingTimers.get(key))
    pendingTimers.delete(key)
  }
  saveJson(key, snapshot)
}

export function snapshotStorageKeys(): SnapshotStorageKeys {
  return { ...STORAGE_KEYS }
}
