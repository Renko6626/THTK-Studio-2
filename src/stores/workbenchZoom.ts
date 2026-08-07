import { defineStore } from 'pinia'
import {
  applyZoom,
  clampLevel,
  formatScale,
  loadZoomLevel,
  MAX_LEVEL,
  MIN_LEVEL,
  saveZoomLevel
} from '../services/workbench/zoom'

/**
 * 工作台缩放等级。0 = 100%，每级 1.2 倍（同 VS Code）。
 *
 * 走 Tauri 的 webview zoom，一次覆盖 Monaco / xterm / UI 三处——理由见
 * `services/workbench/zoom.ts` 的模块注释。
 */
export const useWorkbenchZoomStore = defineStore('workbenchZoom', {
  state: () => ({
    level: 0
  }),
  getters: {
    /** 给菜单显示的百分比 */
    label: (state) => formatScale(state.level),
    canZoomIn: (state) => state.level < MAX_LEVEL,
    canZoomOut: (state) => state.level > MIN_LEVEL
  },
  actions: {
    /** 启动时恢复上次的缩放。放在 store 而不是组件里，避免多个组件各恢复一次 */
    async restore() {
      this.level = loadZoomLevel()
      await applyZoom(this.level)
    },
    async setLevel(level: number) {
      const next = clampLevel(level)
      if (next === this.level) return
      this.level = next
      saveZoomLevel(next)
      await applyZoom(next)
    },
    zoomIn() {
      return this.setLevel(this.level + 1)
    },
    zoomOut() {
      return this.setLevel(this.level - 1)
    },
    reset() {
      return this.setLevel(0)
    }
  }
})
