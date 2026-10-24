import { afterEach, describe, expect, it, vi } from 'vitest'
import { editor } from 'monaco-editor/esm/vs/editor/editor.api'
import { EditorModelSessions } from '../../src/services/workbench/editorModels'
import type { EditorTab } from '../../src/stores/editor'

vi.mock('monaco-editor', async () => import('monaco-editor/esm/vs/editor/editor.api'))

function tab(path: string, content: string): EditorTab {
  return { path, content, name: path.split('/').pop()!, language: 'txt', viewType: 'text', size: null, extension: 'txt', category: null, isDirty: false, originalContent: content }
}

afterEach(() => {
  editor.getModels().forEach(model => model.dispose())
})

describe('编辑器文件会话', () => {
  it('切换文件后复用真实文本模型及撤销历史', async () => {
    const sessions = new EditorModelSessions()
    const first = sessions.open(tab('/a.txt', 'original'))
    first.model.pushEditOperations([], [{ range: first.model.getFullModelRange(), text: 'edited' }], () => null)
    first.model.pushStackElement()
    sessions.open(tab('/b.txt', 'other'))
    const restored = sessions.open(tab('/a.txt', 'edited'))
    expect(restored.model).toBe(first.model)
    await restored.model.undo()
    expect(restored.model.getValue()).toBe('original')
    sessions.dispose()
  })

  it('关闭后清理旧模型和视图状态，重开使用新的磁盘内容', () => {
    const sessions = new EditorModelSessions()
    const first = sessions.open(tab('/a.txt', 'old'))
    first.viewState = { cursorState: [], viewState: { scrollTop: 200, scrollTopWithoutViewZones: 200, scrollLeft: 0, firstPosition: { lineNumber: 1, column: 1 }, firstPositionDeltaTop: 0 }, contributionsState: {} }
    sessions.retain([])
    expect(first.model.isDisposed()).toBe(true)
    const reopened = sessions.open(tab('/a.txt', 'new on disk'))
    expect(reopened.model.getValue()).toBe('new on disk')
    expect(reopened.viewState).toBeNull()
    sessions.dispose()
    expect(reopened.model.isDisposed()).toBe(true)
  })

  it('保留打开的非活动文件，并同步外部更新', () => {
    const sessions = new EditorModelSessions()
    const first = sessions.open(tab('/a.txt', 'before'))
    const second = sessions.open(tab('/b.txt', 'other'))
    sessions.retain(['/a.txt'])
    expect(first.model.isDisposed()).toBe(false)
    expect(second.model.isDisposed()).toBe(true)
    expect(sessions.open(tab('/a.txt', 'external change')).model.getValue()).toBe('external change')
    sessions.dispose()
  })
})
