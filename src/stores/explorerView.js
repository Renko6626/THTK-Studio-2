import { defineStore } from 'pinia'

export const useExplorerViewStore = defineStore('explorerView', {
  state: () => ({
    selectedPaths: []
  }),

  getters: {
    selectionCount: (state) => state.selectedPaths.length,
    hasSelection: (state) => state.selectedPaths.length > 0
  },

  actions: {
    setSelectedPaths(paths) {
      this.selectedPaths = Array.isArray(paths) ? [...paths] : []
    },

    clearSelection() {
      this.selectedPaths = []
    }
  }
})
