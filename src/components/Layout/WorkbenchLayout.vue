<template>
  <div class="h-screen w-screen bg-[#1e1e1e] text-white flex overflow-hidden">
    <aside
      v-if="showLeftSidebar"
      class="shrink-0 bg-[#252526] border-r border-black overflow-hidden"
      :style="{ width: `${leftSidebarWidth}px` }"
    >
      <slot name="left-sidebar" />
    </aside>

    <div class="flex-1 min-w-0 flex flex-col overflow-hidden">
      <header v-if="$slots.topbar" class="shrink-0 bg-[#1e1e1e] border-b border-white/6 relative z-10 overflow-visible">
        <slot name="topbar" />
      </header>

      <div class="flex-1 min-h-0 flex overflow-hidden">
        <main class="flex-1 min-w-0 min-h-0 flex flex-col bg-[#1e1e1e] relative z-20 overflow-visible">
          <div class="flex-1 min-h-0 overflow-hidden">
            <slot name="main" />
          </div>

          <section
            v-if="showBottomPanel"
            class="shrink-0 border-t border-black bg-[#181818]"
            :style="{ height: `${bottomPanelHeight}px` }"
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

        <aside
          v-if="showRightSidebar"
          class="shrink-0 bg-[#252526] border-l border-black overflow-hidden"
          :style="{ width: `${rightSidebarWidth}px` }"
        >
          <slot name="right-sidebar" />
        </aside>
      </div>
    </div>
  </div>
</template>

<script setup>
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
  },
  leftSidebarWidth: {
    type: Number,
    default: 280
  },
  rightSidebarWidth: {
    type: Number,
    default: 320
  },
  bottomPanelHeight: {
    type: Number,
    default: 240
  }
})
</script>
