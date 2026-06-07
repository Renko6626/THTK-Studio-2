import { defineStore } from 'pinia'

export const useExplorerClipboardStore = defineStore('explorerClipboard', {
  state: () => ({
    mode: null,
    entries: []
  }),

  getters: {
    entry: (state) => state.entries[0] || null,
    hasEntry: (state) => state.entries.length > 0,
    hasEntries: (state) => state.entries.length > 0,
    count: (state) => state.entries.length,
    isCut: (state) => state.mode === 'cut',
    isCopy: (state) => state.mode === 'copy'
  },

  actions: {
    setCopy(entries) {
      this.mode = 'copy'
      this.entries = (Array.isArray(entries) ? entries : [entries]).filter(Boolean).map(entry => ({ ...entry }))
    },

    setCut(entries) {
      this.mode = 'cut'
      this.entries = (Array.isArray(entries) ? entries : [entries]).filter(Boolean).map(entry => ({ ...entry }))
    },

    clear() {
      this.mode = null
      this.entries = []
    }
  }
})
