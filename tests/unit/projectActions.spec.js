import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProjectActions } from '../../src/composables/useProjectActions'
import { useEditorStore } from '../../src/stores/editor'
import { useExplorerClipboardStore } from '../../src/stores/explorerClipboard'
import { useProjectStore } from '../../src/stores/project'
import { openProject } from '../../src/api'

vi.mock('../../src/api', () => ({
  openProject: vi.fn(),
  getFileTree: vi.fn().mockResolvedValue([]),
  getDirChildren: vi.fn().mockResolvedValue([]),
  loadProjectConfig: vi.fn().mockResolvedValue({ status: 'absent', value: null, error: null, path: '' }),
  saveProjectConfig: vi.fn().mockResolvedValue(undefined),
  listRecentProjects: vi.fn().mockResolvedValue([]),
  removeRecentProject: vi.fn().mockResolvedValue([]),
  clearRecentProjects: vi.fn().mockResolvedValue(undefined),
  readFile: vi.fn().mockResolvedValue(''),
  saveFile: vi.fn().mockResolvedValue(undefined)
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn()
}))

/** 从对话框的 action 渲染函数里按按钮文字取出点击回调 */
function clickDialogButton(options, label) {
  const node = options.action()
  const button = node.children.find(child => child.children?.default?.() === label)
  if (!button) throw new Error(`对话框里没有「${label}」按钮`)
  button.props.onClick()
}

function makeActions() {
  const dialogCalls = []
  const dialog = {
    warning: vi.fn((options) => {
      dialogCalls.push(options)
      return { destroy: vi.fn() }
    })
  }
  const message = { success: vi.fn(), error: vi.fn(), warning: vi.fn() }
  return { actions: useProjectActions({ message, dialog }), dialog, dialogCalls, message }
}

function openResultFor(rootPath) {
  return {
    rootPath,
    files: [],
    projectConfig: { status: 'absent', value: null, error: null, path: '' }
  }
}

/** 给 editor store 塞一个脏标签 */
function addDirtyTab(editorStore, path) {
  editorStore.tabs.push({
    path,
    name: path.split('/').pop(),
    isDirty: true,
    viewType: 'text',
    content: 'x',
    originalContent: 'y'
  })
}

describe('useProjectActions', () => {
  let editorStore
  let projectStore
  let clipboardStore

  beforeEach(() => {
    setActivePinia(createPinia())
    editorStore = useEditorStore()
    projectStore = useProjectStore()
    clipboardStore = useExplorerClipboardStore()
    openProject.mockReset()
  })

  it('打开失败时保留当前项目并返回 false', async () => {
    openProject.mockResolvedValueOnce(openResultFor('/proj/a'))
    const { actions, message } = makeActions()
    await actions.openProjectPath('/proj/a')

    openProject.mockRejectedValueOnce('目录不存在: /gone')
    const opened = await actions.openProjectPath('/gone')

    expect(opened).toBe(false)
    expect(projectStore.rootPath).toBe('/proj/a')
    expect(message.error).toHaveBeenCalledWith(expect.stringContaining('/gone'))
  })

  it('silent 模式下失败不弹错误提示', async () => {
    openProject.mockRejectedValueOnce('打不开')
    const { actions, message } = makeActions()

    const opened = await actions.openProjectPath('/gone', { silent: true })

    expect(opened).toBe(false)
    expect(message.error).not.toHaveBeenCalled()
  })

  it('没有脏标签时切换项目不打扰用户', async () => {
    openProject.mockResolvedValueOnce(openResultFor('/proj/a'))
    const { actions, dialog } = makeActions()
    await actions.openProjectPath('/proj/a')

    openProject.mockResolvedValueOnce(openResultFor('/proj/b'))
    await actions.openProjectPath('/proj/b')

    expect(dialog.warning).not.toHaveBeenCalled()
    expect(projectStore.rootPath).toBe('/proj/b')
  })

  it('只有项目外的脏标签时不触发确认框', async () => {
    openProject.mockResolvedValueOnce(openResultFor('/proj/a'))
    const { actions, dialog } = makeActions()
    await actions.openProjectPath('/proj/a')

    // 这个文件不在 /proj/a 底下，切换时不会被关掉，拿它凑数会让用户白看一次确认框
    addDirtyTab(editorStore, '/somewhere/else/x.decl')

    openProject.mockResolvedValueOnce(openResultFor('/proj/b'))
    await actions.openProjectPath('/proj/b')

    expect(dialog.warning).not.toHaveBeenCalled()
    expect(projectStore.rootPath).toBe('/proj/b')
  })

  describe('有脏标签时的三向选择', () => {
    let actions
    let dialogCalls
    let message

    beforeEach(async () => {
      openProject.mockResolvedValueOnce(openResultFor('/proj/a'))
      const made = makeActions()
      actions = made.actions
      dialogCalls = made.dialogCalls
      message = made.message
      await actions.openProjectPath('/proj/a')
      addDirtyTab(editorStore, '/proj/a/st01.decl')
      openProject.mockResolvedValue(openResultFor('/proj/b'))
    })

    it('取消：不切换项目，标签原样保留', async () => {
      const pending = actions.openProjectPath('/proj/b')
      await vi.waitFor(() => expect(dialogCalls).toHaveLength(1))
      clickDialogButton(dialogCalls[0], '取消')

      expect(await pending).toBe(false)
      expect(projectStore.rootPath).toBe('/proj/a')
      expect(editorStore.tabs).toHaveLength(1)
    })

    it('保存失败时停止切换，避免把没存住的改动一起关掉', async () => {
      vi.spyOn(editorStore, 'saveAllFiles').mockResolvedValue(false)

      const pending = actions.openProjectPath('/proj/b')
      await vi.waitFor(() => expect(dialogCalls).toHaveLength(1))
      clickDialogButton(dialogCalls[0], '保存并切换')

      expect(await pending).toBe(false)
      expect(projectStore.rootPath).toBe('/proj/a')
      expect(message.error).toHaveBeenCalledWith(expect.stringContaining('保存失败'))
    })

    it('保存成功后完成切换并关掉旧项目的标签', async () => {
      vi.spyOn(editorStore, 'saveAllFiles').mockResolvedValue(true)

      const pending = actions.openProjectPath('/proj/b')
      await vi.waitFor(() => expect(dialogCalls).toHaveLength(1))
      clickDialogButton(dialogCalls[0], '保存并切换')

      expect(await pending).toBe(true)
      expect(projectStore.rootPath).toBe('/proj/b')
      expect(editorStore.tabs).toHaveLength(0)
    })

    it('放弃并切换：不保存，直接切', async () => {
      const saveSpy = vi.spyOn(editorStore, 'saveAllFiles')

      const pending = actions.openProjectPath('/proj/b')
      await vi.waitFor(() => expect(dialogCalls).toHaveLength(1))
      clickDialogButton(dialogCalls[0], '放弃并切换')

      expect(await pending).toBe(true)
      expect(saveSpy).not.toHaveBeenCalled()
      expect(projectStore.rootPath).toBe('/proj/b')
    })

    it('三个选项都是可见按钮，销毁性操作不藏在默认位置', async () => {
      const pending = actions.openProjectPath('/proj/b')
      await vi.waitFor(() => expect(dialogCalls).toHaveLength(1))

      const labels = dialogCalls[0].action().children.map(child => child.children.default())
      expect(labels).toEqual(['取消', '放弃并切换', '保存并切换'])

      clickDialogButton(dialogCalls[0], '取消')
      await pending
    })
  })

  it('切换项目时清空资源管理器剪贴板，避免跨项目粘贴', async () => {
    openProject.mockResolvedValueOnce(openResultFor('/proj/a'))
    const { actions } = makeActions()
    await actions.openProjectPath('/proj/a')

    clipboardStore.entries = [{ path: '/proj/a/x.decl' }]
    clipboardStore.mode = 'cut'

    openProject.mockResolvedValueOnce(openResultFor('/proj/b'))
    await actions.openProjectPath('/proj/b')

    expect(clipboardStore.entries).toHaveLength(0)
  })

  it('重入保护：前一次打开还没结束时，第二次直接返回 false', async () => {
    let releaseFirst
    openProject.mockImplementationOnce(
      () => new Promise((resolve) => { releaseFirst = () => resolve(openResultFor('/proj/a')) })
    )
    const { actions } = makeActions()

    const first = actions.openProjectPath('/proj/a')
    const second = await actions.openProjectPath('/proj/b')

    expect(second).toBe(false)
    expect(openProject).toHaveBeenCalledTimes(1)

    releaseFirst()
    expect(await first).toBe(true)
  })
})
