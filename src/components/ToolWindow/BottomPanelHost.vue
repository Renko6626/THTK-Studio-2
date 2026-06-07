<template>
  <div class="h-full flex flex-col bg-[#181818]">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="flex items-center gap-1">
        <button
          v-for="panel in panels"
          :key="panel.key"
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent uppercase tracking-[0.08em] transition-colors"
          :class="panelClasses(panel.key)"
          @click="workbenchPanelsStore.showBottomPanel(panel.key)"
        >
          {{ panel.label }}
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
          @click="workbenchPanelsStore.hideBottomPanel()"
        >
          隐藏
        </button>
      </div>
    </div>

    <div class="flex-1 min-h-0">
      <TerminalPanel v-if="workbenchPanelsStore.activeBottomPanel === 'terminal'" />
      <OutputPanel v-else-if="workbenchPanelsStore.activeBottomPanel === 'output'" />
      <ProblemsPanel v-else-if="workbenchPanelsStore.activeBottomPanel === 'problems'" />

      <div
        v-else
        class="h-full flex items-center justify-center text-sm text-gray-500 bg-[#111111]"
      >
        {{ currentPanelLabel }} 面板尚未实现
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import TerminalPanel from './TerminalPanel.vue'
import OutputPanel from './OutputPanel.vue'
import ProblemsPanel from './ProblemsPanel.vue'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'

const workbenchPanelsStore = useWorkbenchPanelsStore()

const panels = [
  { key: 'terminal', label: '终端' },
  { key: 'output', label: '输出' },
  { key: 'problems', label: '问题' }
]

const currentPanelLabel = computed(() => {
  return panels.find(panel => panel.key === workbenchPanelsStore.activeBottomPanel)?.label || '未知'
})

function panelClasses(key) {
  return key === workbenchPanelsStore.activeBottomPanel
    ? 'text-gray-100 border-[#3b82f6] bg-transparent'
    : 'text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/55'
}
</script>
