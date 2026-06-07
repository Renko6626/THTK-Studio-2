import { defineStore } from 'pinia'

export const useToolchainSettingsStore = defineStore('toolchainSettings', {
  state: () => ({
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
