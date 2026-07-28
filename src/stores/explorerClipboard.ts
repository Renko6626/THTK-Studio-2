import { defineStore } from 'pinia'
import type { FileNode } from '../types'

export type ClipboardMode = 'copy' | 'cut' | null

interface ExplorerClipboardState {
  mode: ClipboardMode
  /** 条目来自文件树，即 projectStore.files 里的节点 */
  entries: FileNode[]
}

/** 单个或多个条目都接受，内部一律归一成数组并浅拷贝 */
function toEntryList(entries: FileNode | FileNode[] | null | undefined): FileNode[] {
  const list = Array.isArray(entries) ? entries : [entries]
  return list.filter((entry): entry is FileNode => Boolean(entry)).map(entry => ({ ...entry }))
}

export const useExplorerClipboardStore = defineStore('explorerClipboard', {
  state: (): ExplorerClipboardState => ({
    mode: null,
    entries: []
  }),

  getters: {
    entry: (state): FileNode | null => state.entries[0] || null,
    hasEntry: (state): boolean => state.entries.length > 0,
    hasEntries: (state): boolean => state.entries.length > 0,
    count: (state): number => state.entries.length,
    isCut: (state): boolean => state.mode === 'cut',
    isCopy: (state): boolean => state.mode === 'copy'
  },

  actions: {
    setCopy(entries: FileNode | FileNode[] | null | undefined) {
      this.mode = 'copy'
      this.entries = toEntryList(entries)
    },

    setCut(entries: FileNode | FileNode[] | null | undefined) {
      this.mode = 'cut'
      this.entries = toEntryList(entries)
    },

    clear() {
      this.mode = null
      this.entries = []
    }
  }
})
