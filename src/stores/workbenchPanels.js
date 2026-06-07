import { defineStore } from 'pinia'

export const useWorkbenchPanelsStore = defineStore('workbenchPanels', {
  state: () => ({
    bottomVisible: true,
    activeBottomPanel: 'terminal',
    rightVisible: true,
    activeRightPanel: 'outline',
    minimapVisible: true
  }),

  actions: {
    showBottomPanel(panel = this.activeBottomPanel) {
      this.activeBottomPanel = panel
      this.bottomVisible = true
    },

    hideBottomPanel() {
      this.bottomVisible = false
    },

    toggleBottomPanel(panel = this.activeBottomPanel) {
      if (this.bottomVisible && this.activeBottomPanel === panel) {
        this.bottomVisible = false
        return
      }
      this.activeBottomPanel = panel
      this.bottomVisible = true
    },

    showRightPanel(panel = this.activeRightPanel) {
      this.activeRightPanel = panel
      this.rightVisible = true
    },

    toggleRightPanel(panel = this.activeRightPanel) {
      if (this.rightVisible && this.activeRightPanel === panel) {
        this.rightVisible = false
        return
      }
      this.activeRightPanel = panel
      this.rightVisible = true
    },

    toggleMinimap() {
      this.minimapVisible = !this.minimapVisible
    },

    hydrate(snapshot) {
      if (!snapshot) return
      if (typeof snapshot.bottomVisible === 'boolean') {
        this.bottomVisible = snapshot.bottomVisible
      }
      if (snapshot.activeBottomPanel) {
        this.activeBottomPanel = snapshot.activeBottomPanel
      }
      if (typeof snapshot.rightVisible === 'boolean') {
        this.rightVisible = snapshot.rightVisible
      }
      if (snapshot.activeRightPanel) {
        this.activeRightPanel =
          snapshot.activeRightPanel === 'inspector'
            ? 'outline'
            : snapshot.activeRightPanel
      }
      if (typeof snapshot.minimapVisible === 'boolean') {
        this.minimapVisible = snapshot.minimapVisible
      }
    },

    toSnapshot() {
      return {
        bottomVisible: this.bottomVisible,
        activeBottomPanel: this.activeBottomPanel,
        rightVisible: this.rightVisible,
        activeRightPanel: this.activeRightPanel,
        minimapVisible: this.minimapVisible
      }
    }
  }
})
