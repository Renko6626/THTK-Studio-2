import { defineStore } from 'pinia'

interface ToolchainSettingsState {
  visible: boolean
}

export const useToolchainSettingsStore = defineStore('toolchainSettings', {
  state: (): ToolchainSettingsState => ({
    visible: false
  }),

  actions: {
    open() {
      this.visible = true
    },

    close() {
      this.visible = false
    }
  }
})
