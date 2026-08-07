<template>
  <div class="h-full flex flex-col bg-[#252526]">
    <div class="h-8 px-3 flex items-center justify-between border-b border-white/6 text-[11px] font-semibold uppercase tracking-[0.08em] text-gray-400">
      <span>辅助侧栏</span>
      <div class="flex items-center gap-1">
        <button
          v-for="panel in panels"
          :key="panel.key"
          type="button"
          class="h-6 px-2 rounded-sm border border-transparent bg-transparent text-[10px] transition-colors"
          :class="panelClasses(panel.key)"
          @click="workbenchPanelsStore.showRightPanel(panel.key)"
        >
          {{ panel.label }}
        </button>
      </div>
    </div>

    <div class="flex-1 min-h-0 min-w-0 overflow-hidden p-3">
      <EclOutlinePanel v-if="workbenchPanelsStore.activeRightPanel === 'outline'" />
      <TimelinePanel v-else-if="workbenchPanelsStore.activeRightPanel === 'timeline'" />
      <EclReferencesPanel v-else />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import type { RightPanelKey } from '../../stores/workbenchPanels'
import EclOutlinePanel from './EclOutlinePanel.vue'
import EclReferencesPanel from './EclReferencesPanel.vue'
import TimelinePanel from './TimelinePanel.vue'

const workbenchPanelsStore = useWorkbenchPanelsStore()

const panels: { key: RightPanelKey; label: string }[] = [
  { key: 'references', label: '引用' },
  { key: 'outline', label: '大纲' },
  // 不替换 outline，并列即可：各面板自己判断适用性（outline 只认 .decl、
  // timeline 只认 .dstd/.dmsg）。按文件类型换面板的分支逻辑会越堆越乱，
  // 而 ANM 进来又是另一种结构。
  { key: 'timeline', label: '时间线' }
]

function panelClasses(key: RightPanelKey) {
  return key === workbenchPanelsStore.activeRightPanel
    ? 'text-gray-100 border-[#3b82f6]'
    : 'text-gray-500 hover:text-gray-200 hover:border-[#3b82f6]/55'
}
</script>
