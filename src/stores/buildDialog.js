import { defineStore } from 'pinia'
import { createDefaultBuildPayload } from '../services/toolchains/registry'

export const useBuildDialogStore = defineStore('buildDialog', {
  state: () => ({
    visible: false,
    payload: createDefaultBuildPayload()
  }),

  actions: {
    openDialog(payload = {}) {
      const tool = payload.tool || 'thecl'
      this.payload = {
        ...createDefaultBuildPayload(tool),
        ...payload,
        mapPaths: [...(payload.mapPaths || [])]
      }
      this.visible = true
    },

    openTheclDialog(payload = {}) {
      this.openDialog(payload)
    },

    close() {
      this.visible = false
    }
  }
})
