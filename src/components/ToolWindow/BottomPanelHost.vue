<template>
  <div class="workbench-panel h-full flex flex-col bg-[#1e1e1e]">
    <div class="h-[35px] pl-5 pr-2 flex items-center justify-between border-b border-[#2b2b2b] shrink-0">
      <div class="flex items-center gap-5 h-full min-w-0" role="tablist">
        <button
          v-for="panel in panels"
          :key="panel.key"
          type="button"
          class="panel-tab !px-0 gap-1.5"
          :class="panel.key === workbenchPanelsStore.activeBottomPanel ? 'panel-tab-active' : ''"
          role="tab"
          :aria-selected="panel.key === workbenchPanelsStore.activeBottomPanel"
          @click="workbenchPanelsStore.showBottomPanel(panel.key)"
        >
          {{ panel.label }}
          <span v-if="panel.key === 'problems' && reports.problemEntries.length" class="rounded-full bg-[#3c3c3c] px-1.5 text-[10px] leading-4 text-[#cccccc]">{{ reports.problemEntries.length }}</span>
        </button>
      </div>
      <div class="flex items-center">
        <button
          type="button"
          class="panel-action"
          :title="workbenchPanelsStore.bottomMaximized ? '还原面板高度' : '最大化面板'"
          @click="workbenchPanelsStore.toggleBottomMaximized()"
        >
          <n-icon :size="16">
            <ChevronDown16Regular v-if="workbenchPanelsStore.bottomMaximized" />
            <ChevronUp16Regular v-else />
          </n-icon>
        </button>
        <button
          type="button"
          class="panel-action"
          title="隐藏面板"
          @click="workbenchPanelsStore.hideBottomPanel()"
        >
          <n-icon :size="16"><Dismiss16Regular /></n-icon>
        </button>
      </div>
    </div>

    <div class="flex-1 min-h-0">
      <div v-show="workbenchPanelsStore.activeBottomPanel === 'terminal'" class="h-full">
        <TerminalPanel />
      </div>
      <OutputPanel v-if="workbenchPanelsStore.activeBottomPanel === 'output'" />
      <ProblemsPanel v-else-if="workbenchPanelsStore.activeBottomPanel === 'problems'" />

      <div
        v-else-if="workbenchPanelsStore.activeBottomPanel !== 'terminal'"
        class="h-full flex items-center justify-center text-sm text-[#9d9d9d] bg-[#1e1e1e]"
      >
        {{ currentPanelLabel }} 面板尚未实现
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NIcon } from 'naive-ui'
import { ChevronDown16Regular, ChevronUp16Regular, Dismiss16Regular } from '@vicons/fluent'
import TerminalPanel from './TerminalPanel.vue'
import OutputPanel from './OutputPanel.vue'
import ProblemsPanel from './ProblemsPanel.vue'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import type { BottomPanelKey } from '../../stores/workbenchPanels'

const reports = useWorkbenchReportsStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()

const panels: { key: BottomPanelKey; label: string }[] = [
  { key: 'terminal', label: '终端' },
  { key: 'output', label: '输出' },
  { key: 'problems', label: '问题' }
]

const currentPanelLabel = computed(() => {
  return panels.find(panel => panel.key === workbenchPanelsStore.activeBottomPanel)?.label || '未知'
})

</script>

<style scoped>
.workbench-panel { font-family: "Segoe UI", "Microsoft YaHei", system-ui, sans-serif; }
.workbench-panel :deep(button) { font-family: inherit; }
</style>
