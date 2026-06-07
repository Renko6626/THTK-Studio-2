import { defineStore } from 'pinia'
import { readFile, saveFile } from '../api'

const MAX_PERSISTED_DRAFT_CHARS = 200_000
const MAX_PERSISTED_TOTAL_DRAFT_CHARS = 1_000_000

function getPathSeparator(path) {
  return path.includes('\\') ? '\\' : '/'
}

function getBaseName(path) {
  return path.split(/[\\/]/).pop()
}

function isBinaryEclFile(fileNode) {
  const category = String(fileNode?.category || '').toLowerCase()
  const extension = String(fileNode?.extension || '').toLowerCase()
  return category === 'binaryscript' && extension === 'ecl'
}

function createTextTab(path, content, language, overrides = {}) {
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

function createBinaryScriptTab(fileNode) {
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
  state: () => ({
    tabs: [],          // [{ path, name, content, isDirty, language, viewType }]
    activePath: null,  // 当前激活的 tab path
    compiling: false   // 是否正在编译
  }),

  getters: {
    activeTab: (state) => state.tabs.find(t => t.path === state.activePath),
    hasDirtyTabs: (state) => state.tabs.some(t => t.isDirty)
  },

  actions: {
    inferLanguage(path) {
      const ext = path.split('.').pop()?.toLowerCase()
      if (['decl', 'ecl', 'c', 'cpp', 'h'].includes(ext)) return 'cpp'
      if (['js', 'json', 'ts'].includes(ext)) return ext
      if (ext === 'vue') return 'html'
      return 'plaintext'
    },

    // 核心：打开文件
    async openFile(fileNode) {
      // 1. 如果 Tab 已存在，直接切换过去
      const existingTab = this.tabs.find(t => t.path === fileNode.path)
      if (existingTab) {
        this.activePath = fileNode.path
        return
      }

      // 2. 二进制脚本不进入文本编辑器，改为专用工作区视图
      if (isBinaryEclFile(fileNode)) {
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
    closeTab(path) {
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
    updateContent(path, newContent) {
      const tab = this.tabs.find(t => t.path === path)
      if (tab) {
        tab.content = newContent
        tab.isDirty = newContent !== tab.originalContent
      }
    },

    // 保存文件
    async saveActiveFile() {
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

    async saveAllFiles() {
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

    closeTabsUnderPath(path) {
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

    handlePathRename(oldPath, newPath) {
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

    async restoreSession(snapshot) {
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
          const content = shouldRestoreDraft ? savedTab.content : diskContent

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

    toSnapshot() {
      let persistedDraftChars = 0

      return {
        activePath: this.activePath,
        tabs: this.tabs.map((tab) => {
          const snapshot = {
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
