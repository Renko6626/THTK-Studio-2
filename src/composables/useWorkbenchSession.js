import { onMounted, onBeforeUnmount, watch } from 'vue'
import {
  flushSnapshotSave,
  loadEditorSnapshot,
  loadProjectSnapshot,
  loadTerminalSnapshot,
  scheduleSnapshotSave,
  snapshotStorageKeys
} from '../utils/workbenchState'

export function useWorkbenchSession({
  projectStore,
  editorStore,
  terminalStore,
  workbenchPanelsStore,
  showReloadNotice
}) {
  let stopProjectSubscription = null
  let stopEditorSubscription = null
  let stopTerminalSubscription = null
  let stopWorkbenchPanelsSubscription = null
  const storageKeys = snapshotStorageKeys()
  const panelStorageKey = 'thtk-studio:workbench-panels'

  function flushSnapshots() {
    flushSnapshotSave(storageKeys.project, projectStore.toSnapshot())
    flushSnapshotSave(storageKeys.editor, editorStore.toSnapshot())
    flushSnapshotSave(storageKeys.terminal, terminalStore.toSnapshot())
    flushSnapshotSave(panelStorageKey, workbenchPanelsStore.toSnapshot())
  }

  async function restoreWorkbench() {
    const projectSnapshot = loadProjectSnapshot()
    const editorSnapshot = loadEditorSnapshot()
    const terminalSnapshot = loadTerminalSnapshot()
    const panelSnapshot = window.localStorage.getItem(panelStorageKey)

    projectStore.hydrate(projectSnapshot)
    terminalStore.hydrate(terminalSnapshot)
    workbenchPanelsStore.hydrate(panelSnapshot ? JSON.parse(panelSnapshot) : null)

    if (projectSnapshot?.rootPath) {
      await projectStore.loadProject(projectSnapshot.rootPath)
    }

    if (editorSnapshot) {
      const restoreResult = await editorStore.restoreSession(editorSnapshot)
      if (restoreResult?.droppedDraftCount) {
        showReloadNotice(`有 ${restoreResult.droppedDraftCount} 个草稿因磁盘已变化而未恢复。`)
      } else if (restoreResult?.missingCount) {
        showReloadNotice(`有 ${restoreResult.missingCount} 个文件在恢复时未找到。`)
      }
    }
  }

  function startSnapshotSubscriptions() {
    stopProjectSubscription = projectStore.$subscribe(() => {
      scheduleSnapshotSave(storageKeys.project, projectStore.toSnapshot(), 200)
    })

    stopEditorSubscription = editorStore.$subscribe(() => {
      scheduleSnapshotSave(storageKeys.editor, editorStore.toSnapshot(), 500)
    })

    stopTerminalSubscription = terminalStore.$subscribe(() => {
      scheduleSnapshotSave(storageKeys.terminal, terminalStore.toSnapshot(), 250)
    })

    stopWorkbenchPanelsSubscription = workbenchPanelsStore.$subscribe(() => {
      scheduleSnapshotSave(panelStorageKey, workbenchPanelsStore.toSnapshot(), 250)
    })
  }

  watch(
    () => projectStore.rootPath,
    (path) => {
      if (path) {
        terminalStore.setWorkingDirectory(path)
      }
    },
    { immediate: true }
  )

  onMounted(() => {
    startSnapshotSubscriptions()
    void restoreWorkbench()
  })

  onBeforeUnmount(() => {
    stopProjectSubscription?.()
    stopEditorSubscription?.()
    stopTerminalSubscription?.()
    stopWorkbenchPanelsSubscription?.()
    flushSnapshots()
  })

  return {
    flushSnapshots
  }
}
