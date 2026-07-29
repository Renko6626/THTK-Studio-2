import { beforeEach, describe, expect, it, vi } from 'vitest'
import { withSetup, createFakeStore } from '../helpers/withSetup.js'
import { useWorkbenchSession } from '../../src/composables/useWorkbenchSession'
import {
  loadEditorSnapshot,
  loadProjectSnapshot,
  loadTerminalSnapshot
} from '../../src/utils/workbenchState'

vi.mock('../../src/utils/workbenchState', () => ({
  loadProjectSnapshot: vi.fn(),
  loadEditorSnapshot: vi.fn(),
  loadTerminalSnapshot: vi.fn(),
  flushSnapshotSave: vi.fn(),
  scheduleSnapshotSave: vi.fn(),
  snapshotStorageKeys: () => ({
    project: 'p',
    editor: 'e',
    terminal: 't'
  })
}))

const DIRTY_SNAPSHOT = {
  activePath: '/share/proj/a.decl',
  tabs: [
    {
      path: '/share/proj/a.decl',
      isDirty: true,
      content: '改了一半没保存',
      originalContent: '原始内容'
    }
  ]
}

function setup({ openResult = true } = {}) {
  const editorStore = createFakeStore()
  editorStore.restoreSession = vi.fn().mockResolvedValue({ missingCount: 0, droppedDraftCount: 0 })

  const terminalStore = createFakeStore()
  terminalStore.hydrate = vi.fn()
  terminalStore.setWorkingDirectory = vi.fn()

  const workbenchPanelsStore = createFakeStore()
  workbenchPanelsStore.hydrate = vi.fn()

  const openProjectPath = vi.fn().mockResolvedValue(openResult)
  const showReloadNotice = vi.fn()

  withSetup(() =>
    useWorkbenchSession({
      projectStore: createFakeStore({ rootPath: null }),
      editorStore,
      terminalStore,
      workbenchPanelsStore,
      showReloadNotice,
      openProjectPath
    })
  )

  return { editorStore, openProjectPath, showReloadNotice }
}

/** onMounted 里的 restoreWorkbench 是 void 调用，等微任务队列排空 */
const flush = () => new Promise(resolve => setTimeout(resolve, 0))

describe('useWorkbenchSession 恢复流程', () => {
  beforeEach(() => {
    loadTerminalSnapshot.mockReturnValue(null)
    loadProjectSnapshot.mockReturnValue({ rootPath: '/share/proj' })
    loadEditorSnapshot.mockReturnValue(DIRTY_SNAPSHOT)
    window.localStorage.clear()
  })

  it('项目打不开时仍然恢复标签页', async () => {
    // 回归防线：曾经这里会跳过 restoreSession，而 flushSnapshots 随后把空快照
    // 写回 localStorage —— 网络盘晚挂载一次，未保存的草稿就永久没了。
    const { editorStore, openProjectPath } = setup({ openResult: false })
    await flush()

    expect(openProjectPath).toHaveBeenCalledWith('/share/proj', { silent: true })
    expect(editorStore.restoreSession).toHaveBeenCalledWith(DIRTY_SNAPSHOT)
  })

  it('项目打不开时给出提示但不重试', async () => {
    const { openProjectPath, showReloadNotice } = setup({ openResult: false })
    await flush()

    expect(openProjectPath).toHaveBeenCalledTimes(1)
    expect(showReloadNotice).toHaveBeenCalledWith(expect.stringContaining('/share/proj'))
  })

  it('项目正常打开时照常恢复标签页', async () => {
    const { editorStore, showReloadNotice } = setup({ openResult: true })
    await flush()

    expect(editorStore.restoreSession).toHaveBeenCalledWith(DIRTY_SNAPSHOT)
    expect(showReloadNotice).not.toHaveBeenCalled()
  })

  it('没有项目快照时不调用打开动作，但仍恢复标签页', async () => {
    loadProjectSnapshot.mockReturnValue(null)
    const { editorStore, openProjectPath } = setup()
    await flush()

    expect(openProjectPath).not.toHaveBeenCalled()
    expect(editorStore.restoreSession).toHaveBeenCalledWith(DIRTY_SNAPSHOT)
  })

  it('未找到的文件按草稿保留时，提示里说明内容没丢', async () => {
    const { editorStore, showReloadNotice } = setup({ openResult: false })
    editorStore.restoreSession.mockResolvedValue({ missingCount: 2, droppedDraftCount: 0 })
    await flush()

    const notice = showReloadNotice.mock.calls.at(-1)?.[0] || ''
    expect(notice).toContain('2')
  })
})
