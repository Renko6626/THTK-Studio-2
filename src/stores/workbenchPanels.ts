import { defineStore } from 'pinia'

function clampNumber(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(value)))
}

/** 底部面板普通模式下的最大高度:视口减去 topbar(68) + 状态栏(24) + 编辑器最小可见高度 */
function maxBottomPanelHeight(): number {
  return window.innerHeight - 68 - 24 - 60
}

export type BottomPanelKey = 'terminal' | 'output' | 'problems'
export type RightPanelKey = 'outline' | 'references'

/**
 * 持久化快照的入参。字段类型比 state 宽松，因为它来自 localStorage：
 * 可能是旧版本写的（例如 activeRightPanel 曾经有过 'inspector' 这个值，
 * 下面的 hydrate 会把它映射到 outline）。
 *
 * 注意现有实现对面板键**不做**有效性校验，手改过的快照能写进任意字符串。
 * 这是既有行为，迁移不改；真要收紧应该单独提交并配测试。
 */
export interface WorkbenchPanelsSnapshot {
  bottomVisible?: boolean
  activeBottomPanel?: string
  rightVisible?: boolean
  activeRightPanel?: string
  minimapVisible?: boolean
  bottomPanelHeight?: number
  leftSidebarWidth?: number
  rightSidebarWidth?: number
  bottomMaximized?: boolean
}

interface WorkbenchPanelsState {
  bottomVisible: boolean
  activeBottomPanel: BottomPanelKey
  rightVisible: boolean
  activeRightPanel: RightPanelKey
  minimapVisible: boolean
  bottomPanelHeight: number
  leftSidebarWidth: number
  rightSidebarWidth: number
  bottomMaximized: boolean
}

export const useWorkbenchPanelsStore = defineStore('workbenchPanels', {
  state: (): WorkbenchPanelsState => ({
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
    showBottomPanel(panel?: BottomPanelKey) {
      const target = panel ?? this.activeBottomPanel
      this.activeBottomPanel = target
      this.bottomVisible = true
    },

    hideBottomPanel() {
      this.bottomVisible = false
      this.bottomMaximized = false
    },

    setBottomPanelHeight(height: number) {
      // 拖到接近顶部(剩余 <60px)→ 自动进入最大化,对标 VS Code
      const maxHeight = maxBottomPanelHeight()
      if (height >= maxHeight) {
        this.bottomMaximized = true
        return
      }
      this.bottomMaximized = false
      this.bottomPanelHeight = clampNumber(height, 100, maxHeight)
    },

    setLeftSidebarWidth(width: number) {
      this.leftSidebarWidth = clampNumber(width, 160, 600)
    },

    setRightSidebarWidth(width: number) {
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

    toggleBottomPanel(panel?: BottomPanelKey) {
      const target = panel ?? this.activeBottomPanel
      if (this.bottomVisible && this.activeBottomPanel === target) {
        this.bottomVisible = false
        return
      }
      this.activeBottomPanel = target
      this.bottomVisible = true
    },

    showRightPanel(panel?: RightPanelKey) {
      const target = panel ?? this.activeRightPanel
      this.activeRightPanel = target
      this.rightVisible = true
    },

    toggleRightPanel(panel?: RightPanelKey) {
      const target = panel ?? this.activeRightPanel
      if (this.rightVisible && this.activeRightPanel === target) {
        this.rightVisible = false
        return
      }
      this.activeRightPanel = target
      this.rightVisible = true
    },

    toggleMinimap() {
      this.minimapVisible = !this.minimapVisible
    },

    hydrate(snapshot: WorkbenchPanelsSnapshot | null | undefined) {
      if (!snapshot) return
      if (typeof snapshot.bottomVisible === 'boolean') {
        this.bottomVisible = snapshot.bottomVisible
      }
      if (snapshot.activeBottomPanel) {
        this.activeBottomPanel = snapshot.activeBottomPanel as BottomPanelKey
      }
      if (typeof snapshot.rightVisible === 'boolean') {
        this.rightVisible = snapshot.rightVisible
      }
      if (snapshot.activeRightPanel) {
        // 'inspector' 是更早版本用过的键，升级时映射到 outline
        this.activeRightPanel =
          snapshot.activeRightPanel === 'inspector'
            ? 'outline'
            : (snapshot.activeRightPanel as RightPanelKey)
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
