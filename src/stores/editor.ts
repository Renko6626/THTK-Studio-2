import { defineStore } from 'pinia'
import { readFile, saveFile } from '../api'
import type { FileCategory, FileNode } from '../types'
import type { EditorSnapshot, EditorTabSnapshot } from '../utils/workbenchState'

export type EditorViewType = 'text' | 'binary-script'

export interface EditorTab {
  path: string
  name: string
  content: string
  originalContent: string
  isDirty: boolean
  language: string
  viewType: EditorViewType
  size: number | null
  extension: string | null
  category: FileCategory | null
}

interface EditorState {
  tabs: EditorTab[]
  /** 当前激活的 tab path */
  activePath: string | null
  compiling: boolean
}

/** createTextTab 的可选覆盖项 */
type TextTabOverrides = Partial<Omit<EditorTab, 'path' | 'content' | 'language'>>

interface RestoreSessionResult {
  restoredCount: number
  /** 磁盘已变化、草稿无法安全恢复而被丢弃的数量 */
  droppedDraftCount: number
  /** 文件读不到、按快照内容恢复成脏标签的数量 */
  missingCount: number
}

const MAX_PERSISTED_DRAFT_CHARS = 200_000
const MAX_PERSISTED_TOTAL_DRAFT_CHARS = 1_000_000

function getPathSeparator(path: string): string {
  return path.includes('\\') ? '\\' : '/'
}

function getBaseName(path: string): string {
  return path.split(/[\\/]/).pop() || path
}

const BINARY_SCRIPT_EXTENSIONS = new Set(['ecl', 'msg', 'std', 'dat', 'anm'])

function isBinaryScript(fileNode: FileNode | null | undefined): boolean {
  const extension = String(fileNode?.extension || '').toLowerCase()
  return BINARY_SCRIPT_EXTENSIONS.has(extension)
}

function createTextTab(
  path: string,
  content: string,
  language: string,
  overrides: TextTabOverrides = {}
): EditorTab {
  return {
    path,
    name: getBaseName(path),
    content,
    originalContent: content,
    isDirty: false,
    language,
    viewType: 'text',
    size: overrides.size ?? null,
    extension: overrides.extension ?? path.split('.').pop()?.toLowerCase() ?? null,
    category: overrides.category ?? null,
    ...overrides
  }
}

function createBinaryScriptTab(fileNode: EditorTabSnapshot | FileNode): EditorTab {
  return {
    path: fileNode.path,
    name: fileNode.name || getBaseName(fileNode.path),
    content: '',
    originalContent: '',
    isDirty: false,
    language: 'plaintext',
    viewType: 'binary-script',
    size: fileNode.size ?? null,
    extension: fileNode.extension ?? fileNode.path.split('.').pop()?.toLowerCase() ?? null,
    category: fileNode.category ?? 'binaryScript'
  }
}

export const useEditorStore = defineStore('editor', {
  state: (): EditorState => ({
    tabs: [],
    activePath: null,
    compiling: false
  }),

  getters: {
    activeTab: (state): EditorTab | undefined => state.tabs.find(t => t.path === state.activePath),
    hasDirtyTabs: (state): boolean => state.tabs.some(t => t.isDirty)
  },

  actions: {
    inferLanguage(path: string): string {
      const ext = path.split('.').pop()?.toLowerCase() ?? ''
      if (['decl', 'ecl', 'c', 'cpp', 'h'].includes(ext)) return 'cpp'
      if (['js', 'json', 'ts'].includes(ext)) return ext
      if (ext === 'vue') return 'html'
      return 'plaintext'
    },

    // 核心：打开文件
    async openFile(fileNode: FileNode) {
      // 1. 如果 Tab 已存在，直接切换过去
      const existingTab = this.tabs.find(t => t.path === fileNode.path)
      if (existingTab) {
        this.activePath = fileNode.path
        return
      }

      // 2. 二进制脚本不进入文本编辑器，改为专用工作区视图
      if (isBinaryScript(fileNode)) {
        this.tabs.push(createBinaryScriptTab(fileNode))
        this.activePath = fileNode.path
        return
      }

      // 3. 读取文件内容
      try {
        const loadPath = fileNode.path
        const content = await readFile(loadPath)
        const newTab = createTextTab(loadPath, content, this.inferLanguage(loadPath), {
          size: fileNode.size ?? null,
          extension: fileNode.extension ?? loadPath.split('.').pop()?.toLowerCase() ?? null,
          category: fileNode.category ?? null
        })
        this.tabs.push(newTab)
        this.activePath = loadPath

      } catch (err) {
        console.error('无法读取文件', err)
      }
    },

    // 关闭文件
    closeTab(path: string) {
      const index = this.tabs.findIndex(t => t.path === path)
      if (index === -1) return

      // 如果关闭的是当前激活的 tab，需要切换到隔壁那个
      if (path === this.activePath) {
        const nextTab = this.tabs[index + 1] || this.tabs[index - 1]
        this.activePath = nextTab ? nextTab.path : null
      }

      this.tabs.splice(index, 1)
    },

    // 更新内容 (打字时触发)
    updateContent(path: string, newContent: string) {
      const tab = this.tabs.find(t => t.path === path)
      if (tab) {
        tab.content = newContent
        tab.isDirty = newContent !== tab.originalContent
      }
    },

    /**
     * 保存当前文件。
     *
     * ⚠️ 返回类型是 `boolean | undefined` 而不是 `boolean`：非文本标签会走裸 return，
     * 产出 undefined。而 saveAllFiles 把 falsy 当成失败，那个结果又被
     * useProjectActions 当作"是否中止项目切换"的依据。这是既有 bug，
     * 按迁移纪律不在本次提交里改行为——下一个提交单独修并配测试。
     */
    async saveActiveFile(): Promise<boolean | undefined> {
      const tab = this.activeTab
      if (!tab || tab.viewType !== 'text') return

      try {
        await saveFile(tab.path, tab.content, true)
        tab.originalContent = tab.content
        tab.isDirty = false
        return true
      } catch (e) {
        console.error(e)
        return false
      }
    },

    async saveAllFiles(): Promise<boolean> {
      const dirtyTabs = this.tabs.filter(tab => tab.isDirty)
      if (!dirtyTabs.length) return true

      const originalActivePath = this.activePath
      let allSucceeded = true

      for (const tab of dirtyTabs) {
        this.activePath = tab.path
        const succeeded = await this.saveActiveFile()
        if (!succeeded) {
          allSucceeded = false
        }
      }

      this.activePath = originalActivePath
      return allSucceeded
    },

    closeActiveTab() {
      if (this.activePath) {
        this.closeTab(this.activePath)
      }
    },

    closeTabsUnderPath(path: string | null | undefined) {
      if (!path) return
      const prefixWin = `${path}\\`
      const prefixUnix = `${path}/`
      const remainingTabs = this.tabs.filter(
        tab => tab.path !== path && !tab.path.startsWith(prefixWin) && !tab.path.startsWith(prefixUnix)
      )

      this.tabs = remainingTabs
      if (!remainingTabs.some(tab => tab.path === this.activePath)) {
        this.activePath = remainingTabs[remainingTabs.length - 1]?.path || null
      }
    },

    handlePathRename(oldPath: string, newPath: string) {
      if (!oldPath || !newPath || oldPath === newPath) return

      const separator = getPathSeparator(oldPath)
      const prefix = `${oldPath}${separator}`

      this.tabs = this.tabs.map((tab) => {
        if (tab.path === oldPath) {
          return {
            ...tab,
            path: newPath,
            name: getBaseName(newPath)
          }
        }

        if (tab.path.startsWith(prefix)) {
          const nextPath = `${newPath}${tab.path.slice(oldPath.length)}`
          return {
            ...tab,
            path: nextPath,
            name: getBaseName(nextPath)
          }
        }

        return tab
      })

      if (this.activePath === oldPath) {
        this.activePath = newPath
      } else if (this.activePath?.startsWith(prefix)) {
        this.activePath = `${newPath}${this.activePath.slice(oldPath.length)}`
      }
    },

    async restoreSession(snapshot: EditorSnapshot | null): Promise<RestoreSessionResult | undefined> {
      if (!snapshot?.tabs?.length) return

      const restoredTabs = []
      let droppedDraftCount = 0
      let missingCount = 0

      for (const savedTab of snapshot.tabs) {
        if (!savedTab?.path) continue

        if (savedTab.viewType === 'binary-script') {
          restoredTabs.push(createBinaryScriptTab(savedTab))
          continue
        }

        try {
          const diskContent = await readFile(savedTab.path)
          const canRestoreDraft =
            savedTab.isDirty &&
            typeof savedTab.content === 'string' &&
            typeof savedTab.originalContent === 'string' &&
            savedTab.originalContent === diskContent

          if (savedTab.isDirty && !canRestoreDraft) {
            droppedDraftCount += 1
          }

          const shouldRestoreDraft = canRestoreDraft
          // canRestoreDraft 里已经断言过 content 是 string，这里再判一次只为让 TS 收窄
          const content =
            shouldRestoreDraft && typeof savedTab.content === 'string'
              ? savedTab.content
              : diskContent

          restoredTabs.push(createTextTab(
            savedTab.path,
            content,
            savedTab.language || this.inferLanguage(savedTab.path),
            {
              name: savedTab.name || getBaseName(savedTab.path),
              originalContent: savedTab.originalContent ?? diskContent,
              isDirty: shouldRestoreDraft,
              size: savedTab.size ?? null,
              extension: savedTab.extension ?? null,
              category: savedTab.category ?? null
            }
          ))
        } catch {
          missingCount += 1
          if (typeof savedTab.content !== 'string') continue
          restoredTabs.push(createTextTab(
            savedTab.path,
            savedTab.content,
            savedTab.language || this.inferLanguage(savedTab.path),
            {
              name: savedTab.name || getBaseName(savedTab.path),
              originalContent: savedTab.originalContent ?? savedTab.content,
              isDirty: Boolean(savedTab.isDirty),
              size: savedTab.size ?? null,
              extension: savedTab.extension ?? null,
              category: savedTab.category ?? null
            }
          ))
        }
      }

      this.tabs = restoredTabs
      this.activePath =
        restoredTabs.find(tab => tab.path === snapshot.activePath)?.path ||
        restoredTabs[restoredTabs.length - 1]?.path ||
        null

      return {
        restoredCount: restoredTabs.length,
        droppedDraftCount,
        missingCount
      }
    },

    toSnapshot(): EditorSnapshot {
      let persistedDraftChars = 0

      return {
        activePath: this.activePath,
        tabs: this.tabs.map((tab) => {
          const snapshot: EditorTabSnapshot = {
            path: tab.path,
            name: tab.name,
            isDirty: tab.isDirty,
            language: tab.language,
            viewType: tab.viewType || 'text',
            size: tab.size ?? null,
            extension: tab.extension ?? null,
            category: tab.category ?? null
          }

          if (tab.viewType !== 'text' || !tab.isDirty) {
            return snapshot
          }

          const draftLength = tab.content?.length ?? 0
          const fitsBudget =
            draftLength <= MAX_PERSISTED_DRAFT_CHARS &&
            persistedDraftChars + draftLength <= MAX_PERSISTED_TOTAL_DRAFT_CHARS

          if (fitsBudget) {
            persistedDraftChars += draftLength
            snapshot.content = tab.content
            snapshot.originalContent = tab.originalContent
          }

          return snapshot
        })
      }
    }
  }
})
