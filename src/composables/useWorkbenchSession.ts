import { onMounted, onBeforeUnmount, watch } from 'vue'
import {
  flushSnapshotSave,
  loadEditorSnapshot,
  loadProjectSnapshot,
  loadTerminalSnapshot,
  scheduleSnapshotSave,
  snapshotStorageKeys
} from '../utils/workbenchState'
import type { useEditorStore } from '../stores/editor'
import type { useProjectStore } from '../stores/project'
import type { useTerminalStore } from '../stores/terminal'
import type { useWorkbenchPanelsStore } from '../stores/workbenchPanels'

export interface WorkbenchSessionDeps {
  projectStore: ReturnType<typeof useProjectStore>
  editorStore: ReturnType<typeof useEditorStore>
  terminalStore: ReturnType<typeof useTerminalStore>
  workbenchPanelsStore: ReturnType<typeof useWorkbenchPanelsStore>
  showReloadNotice: (text: string) => void
  /** 由 WorkbenchRoot 注入 useProjectActions.openProjectPath，恢复走和其他入口同一条流程 */
  openProjectPath: (path: string, options?: { silent?: boolean }) => Promise<boolean>
}

export function useWorkbenchSession({
  projectStore,
  editorStore,
  terminalStore,
  workbenchPanelsStore,
  showReloadNotice,
  openProjectPath
}: WorkbenchSessionDeps) {
  type StopSubscription = () => void
  let stopProjectSubscription: StopSubscription | null = null
  let stopEditorSubscription: StopSubscription | null = null
  let stopTerminalSubscription: StopSubscription | null = null
  let stopWorkbenchPanelsSubscription: StopSubscription | null = null
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

    const notices = []

    if (projectSnapshot?.rootPath) {
      // silent：上次的目录可能已被删除、改名，或只是网络盘还没挂上，这属于正常情况。
      // 不弹错误也不重试——最近项目列表里那条记录还在，用户可以自己处理。
      const restored = await openProjectPath(projectSnapshot.rootPath, { silent: true })
      if (!restored) {
        notices.push(`上次的项目暂时打不开：${projectSnapshot.rootPath}`)
      }
    }

    // 无论项目是否恢复成功都要恢复标签页。restoreSession 的失败分支会用快照里
    // 保存的草稿内容把读不到的文件恢复成脏标签；跳过它等于把用户没保存的改动扔掉，
    // 而紧接着的 flushSnapshots 会用空快照覆盖 localStorage —— 盘回来了也找不回。
    if (editorSnapshot) {
      const restoreResult = await editorStore.restoreSession(editorSnapshot)
      if (restoreResult?.droppedDraftCount) {
        notices.push(`有 ${restoreResult.droppedDraftCount} 个草稿因磁盘已变化而未恢复。`)
      } else if (restoreResult?.missingCount) {
        notices.push(`有 ${restoreResult.missingCount} 个文件未找到，已按上次的内容保留为未保存标签。`)
      }
    }

    if (notices.length) {
      showReloadNotice(notices.join('　'))
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
