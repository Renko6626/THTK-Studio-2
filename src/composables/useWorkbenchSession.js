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
  showReloadNotice,
  // 由 WorkbenchRoot 注入 useProjectActions.openProjectPath，恢复走和其他入口同一条流程
  openProjectPath
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

    terminalStore.hydrate(terminalSnapshot)
    workbenchPanelsStore.hydrate(panelSnapshot ? JSON.parse(panelSnapshot) : null)

    let projectRestored = false
    if (projectSnapshot?.rootPath) {
      // silent：上次的目录可能已被删除或改名，这属于正常情况。失败就停在欢迎页，
      // 不弹错误、也不重试——最近项目列表仍然留着那条记录供用户处理。
      projectRestored = await openProjectPath(projectSnapshot.rootPath, { silent: true })
      if (!projectRestored) {
        showReloadNotice(`上次的项目已无法打开：${projectSnapshot.rootPath}`)
      }
    }

    // 项目没恢复成功就别再恢复标签页——那些路径都在打不开的目录底下
    if (editorSnapshot && (projectRestored || !projectSnapshot?.rootPath)) {
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
