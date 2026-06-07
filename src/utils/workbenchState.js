const STORAGE_KEYS = {
  project: 'thtk-studio:project',
  editor: 'thtk-studio:editor',
  terminal: 'thtk-studio:terminal'
}

const pendingTimers = new Map()

function loadJson(key, fallback) {
  try {
    const raw = window.localStorage.getItem(key)
    return raw ? JSON.parse(raw) : fallback
  } catch {
    return fallback
  }
}

function saveJson(key, value) {
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Ignore storage failures for now.
  }
}

export function loadProjectSnapshot() {
  return loadJson(STORAGE_KEYS.project, null)
}

export function saveProjectSnapshot(snapshot) {
  saveJson(STORAGE_KEYS.project, snapshot)
}

export function loadEditorSnapshot() {
  return loadJson(STORAGE_KEYS.editor, null)
}

export function saveEditorSnapshot(snapshot) {
  saveJson(STORAGE_KEYS.editor, snapshot)
}

export function loadTerminalSnapshot() {
  return loadJson(STORAGE_KEYS.terminal, null)
}

export function saveTerminalSnapshot(snapshot) {
  saveJson(STORAGE_KEYS.terminal, snapshot)
}

export function scheduleSnapshotSave(key, snapshot, delay = 300) {
  if (pendingTimers.has(key)) {
    window.clearTimeout(pendingTimers.get(key))
  }

  const timer = window.setTimeout(() => {
    saveJson(key, snapshot)
    pendingTimers.delete(key)
  }, delay)

  pendingTimers.set(key, timer)
}

export function flushSnapshotSave(key, snapshot) {
  if (pendingTimers.has(key)) {
    window.clearTimeout(pendingTimers.get(key))
    pendingTimers.delete(key)
  }
  saveJson(key, snapshot)
}

export function snapshotStorageKeys() {
  return { ...STORAGE_KEYS }
}
