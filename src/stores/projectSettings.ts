import { defineStore } from 'pinia'

/** 项目设置对话框的可见性。表单状态留在组件里，这里只管 UI 开关。 */
export const useProjectSettingsStore = defineStore('projectSettings', {
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
