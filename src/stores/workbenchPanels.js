import { defineStore } from 'pinia'

function clampNumber(value, min, max) {
  return Math.min(max, Math.max(min, Math.round(value)))
}

/** 底部面板普通模式下的最大高度:视口减去 topbar(68) + 状态栏(24) + 编辑器最小可见高度 */
function maxBottomPanelHeight() {
  return window.innerHeight - 68 - 24 - 60
}

export const useWorkbenchPanelsStore = defineStore('workbenchPanels', {
  state: () => ({
    bottomVisible: true,
    activeBottomPanel: 'terminal',
    rightVisible: true,
    activeRightPanel: 'outline',
    minimapVisible: true,
    // ---- 可拖动尺寸(持久化)与最大化(会话内) ----
    bottomPanelHeight: 240,
    leftSidebarWidth: 280,
    rightSidebarWidth: 320,
    bottomMaximized: false
  }),

  actions: {
    showBottomPanel(panel = this.activeBottomPanel) {
      this.activeBottomPanel = panel
      this.bottomVisible = true
    },

    hideBottomPanel() {
      this.bottomVisible = false
      this.bottomMaximized = false
    },

    setBottomPanelHeight(height) {
      // 拖到接近顶部(剩余 <60px)→ 自动进入最大化,对标 VS Code
      const maxHeight = maxBottomPanelHeight()
      if (height >= maxHeight) {
        this.bottomMaximized = true
        return
      }
      this.bottomMaximized = false
      this.bottomPanelHeight = clampNumber(height, 100, maxHeight)
    },

    setLeftSidebarWidth(width) {
      this.leftSidebarWidth = clampNumber(width, 160, 600)
    },

    setRightSidebarWidth(width) {
      this.rightSidebarWidth = clampNumber(width, 160, 600)
    },

    toggleBottomMaximized() {
      this.bottomMaximized = !this.bottomMaximized
      if (this.bottomMaximized) {
        this.bottomVisible = true
      }
    },

    exitBottomMaximized() {
      this.bottomMaximized = false
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
      // 尺寸恢复时直接 clamp(不走 setBottomPanelHeight:其"贴顶自动最大化"
      // 逻辑在窗口变小后恢复旧值时会导致启动即全屏)
      if (typeof snapshot.bottomPanelHeight === 'number') {
        this.bottomPanelHeight = clampNumber(
          snapshot.bottomPanelHeight,
          100,
          maxBottomPanelHeight()
        )
      }
      if (typeof snapshot.leftSidebarWidth === 'number') {
        this.setLeftSidebarWidth(snapshot.leftSidebarWidth)
      }
      if (typeof snapshot.rightSidebarWidth === 'number') {
        this.setRightSidebarWidth(snapshot.rightSidebarWidth)
      }
      // bottomMaximized 刻意不持久化:刷新后还原为普通高度
    },

    toSnapshot() {
      return {
        bottomVisible: this.bottomVisible,
        activeBottomPanel: this.activeBottomPanel,
        rightVisible: this.rightVisible,
        activeRightPanel: this.activeRightPanel,
        minimapVisible: this.minimapVisible,
        bottomPanelHeight: this.bottomPanelHeight,
        leftSidebarWidth: this.leftSidebarWidth,
        rightSidebarWidth: this.rightSidebarWidth
      }
    }
  }
})
