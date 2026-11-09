<template>
  <div class="h-full flex flex-col bg-[#1e1e1e]">
    <!-- 不重复画标题：正上方 BottomPanelHost 的 tab 已经写着「输出」 -->
    <div class="h-7 pl-4 pr-2 flex items-center justify-end shrink-0">
      <button type="button" class="panel-action" title="清空输出" aria-label="清空输出" @click="reportsStore.clearOutput()"><n-icon :size="16"><Delete16Regular /></n-icon></button>
    </div>

    <div class="flex-1 overflow-auto px-5 pb-3">
      <div v-for="group in reportsStore.outputGroups" :key="group.ownerKey" class="mb-3 border-b border-white/8 pb-2">
        <div class="flex items-center justify-between gap-4 py-1.5">
          <div class="min-w-0">
            <div class="flex items-center gap-2 text-[10px] text-[#858585]">
              <span>{{ group.source }}</span>
              <span>{{ group.operation }}</span>
              <span>{{ group.scriptKind }}</span>
            </div>
            <div class="mt-0.5 truncate text-xs text-[#cccccc]">
              {{ group.title || group.path || 'Untitled operation' }}
            </div>
          </div>
          <div class="shrink-0 text-[11px] text-gray-500">
            {{ formatTimestamp(group.timestamp) }}
          </div>
        </div>

        <div class="py-1 font-mono text-xs leading-5">
          <div
            v-for="entry in group.lines"
            :key="entry.id"
            class="whitespace-pre-wrap break-words"
            :class="lineClass(entry.level)"
          >
            {{ entry.text || ' ' }}
          </div>
        </div>
      </div>

      <div v-if="!reportsStore.outputGroups.length" class="h-full flex items-center justify-center text-sm text-gray-500">
        暂无输出记录
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui'
import { Delete16Regular } from '@vicons/fluent'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'

const reportsStore = useWorkbenchReportsStore()

function lineClass(level: string) {
  if (level === 'error') return 'text-[#f48771]'
  if (level === 'warning') return 'text-[#dcdcaa]'
  if (level === 'success') return 'text-[#89d185]'
  return 'text-gray-200'
}

function formatTimestamp(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}
</script>
