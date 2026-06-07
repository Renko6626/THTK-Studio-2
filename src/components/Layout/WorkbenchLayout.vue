<template>
  <div class="h-screen w-screen bg-[#1e1e1e] text-white flex overflow-hidden">
    <aside
      v-if="showLeftSidebar"
      class="shrink-0 bg-[#252526] overflow-hidden"
      :style="{ width: `${panels.leftSidebarWidth}px` }"
    >
      <slot name="left-sidebar" />
    </aside>
    <!-- 左侧栏宽度分隔条 -->
    <div
      v-if="showLeftSidebar"
      class="shrink-0 w-1 cursor-col-resize bg-black hover:bg-[#3b82f6]/70 active:bg-[#3b82f6] transition-colors touch-none"
      @pointerdown="leftResize.onPointerdown"
    ></div>

    <div class="flex-1 min-w-0 flex flex-col overflow-hidden">
      <header v-if="$slots.topbar" class="shrink-0 bg-[#1e1e1e] border-b border-white/6 relative z-10 overflow-visible">
        <slot name="topbar" />
      </header>

      <div class="flex-1 min-h-0 flex overflow-hidden">
        <main class="flex-1 min-w-0 min-h-0 flex flex-col bg-[#1e1e1e] relative z-20 overflow-visible">
          <!-- 最大化时编辑器区隐藏(v-show 保留状态,Monaco automaticLayout 自动恢复) -->
          <div v-show="!bottomFullscreen" class="flex-1 min-h-0 overflow-hidden">
            <slot name="main" />
          </div>

          <!-- 底部面板高度分隔条(最大化时无意义,隐藏) -->
          <div
            v-if="showBottomPanel && !bottomFullscreen"
            class="shrink-0 h-1 cursor-row-resize bg-black hover:bg-[#3b82f6]/70 active:bg-[#3b82f6] transition-colors touch-none"
            @pointerdown="bottomResize.onPointerdown"
          ></div>

          <section
            v-if="showBottomPanel"
            class="bg-[#181818]"
            :class="bottomFullscreen ? 'flex-1 min-h-0' : 'shrink-0 border-t border-black'"
            :style="bottomFullscreen ? null : { height: `${panels.bottomPanelHeight}px` }"
          >
            <slot name="bottom-panel" />
          </section>

          <footer
            v-if="$slots.statusbar"
            class="shrink-0 h-6 bg-[#007acc] text-white border-t border-black/20"
          >
            <slot name="statusbar" />
          </footer>
        </main>

        <!-- 右侧栏宽度分隔条 -->
        <div
          v-if="showRightSidebar"
          class="shrink-0 w-1 cursor-col-resize bg-black hover:bg-[#3b82f6]/70 active:bg-[#3b82f6] transition-colors touch-none"
          @pointerdown="rightResize.onPointerdown"
        ></div>
        <aside
          v-if="showRightSidebar"
          class="shrink-0 bg-[#252526] overflow-hidden"
          :style="{ width: `${panels.rightSidebarWidth}px` }"
        >
          <slot name="right-sidebar" />
        </aside>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useResizable } from '../../composables/useResizable'

// 尺寸由 panels store 直接驱动(持久化在 store 的 snapshot 机制里);
// 可见性仍走 props,由 App.vue 决定。
defineProps({
  showLeftSidebar: {
    type: Boolean,
    default: true
  },
  showRightSidebar: {
    type: Boolean,
    default: false
  },
  showBottomPanel: {
    type: Boolean,
    default: false
  }
})

const panels = useWorkbenchPanelsStore()
const bottomFullscreen = computed(() => panels.bottomMaximized)

const leftResize = useResizable({
  axis: 'x',
  getValue: () => panels.leftSidebarWidth,
  setValue: (value) => panels.setLeftSidebarWidth(value)
})

const rightResize = useResizable({
  axis: 'x',
  invert: true, // 向左拖增大右侧栏
  getValue: () => panels.rightSidebarWidth,
  setValue: (value) => panels.setRightSidebarWidth(value)
})

const bottomResize = useResizable({
  axis: 'y',
  invert: true, // 向上拖增大底部面板
  getValue: () => panels.bottomPanelHeight,
  setValue: (value) => panels.setBottomPanelHeight(value),
  onDragStart: () => panels.exitBottomMaximized()
})
</script>
