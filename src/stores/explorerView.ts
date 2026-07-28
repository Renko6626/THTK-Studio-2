import { defineStore } from 'pinia'

interface ExplorerViewState {
  selectedPaths: string[]
}

export const useExplorerViewStore = defineStore('explorerView', {
  state: (): ExplorerViewState => ({
    selectedPaths: []
  }),

  getters: {
    selectionCount: (state): number => state.selectedPaths.length,
    hasSelection: (state): boolean => state.selectedPaths.length > 0
  },

  actions: {
    setSelectedPaths(paths: string[] | null | undefined) {
      this.selectedPaths = Array.isArray(paths) ? [...paths] : []
    },

    clearSelection() {
      this.selectedPaths = []
    }
  }
})
