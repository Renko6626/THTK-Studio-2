<template>
  <div class="h-full flex flex-col bg-[#111111]">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="text-xs font-semibold uppercase tracking-wider text-gray-400">Output</div>
      <button
        type="button"
        class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
        @click="reportsStore.clearOutput()"
      >
        清空
      </button>
    </div>

    <div class="flex-1 overflow-auto px-3 py-2">
      <div v-for="group in reportsStore.outputGroups" :key="group.ownerKey" class="mb-3 rounded border border-white/8 bg-[#181818]">
        <div class="flex items-center justify-between gap-4 border-b border-white/6 px-3 py-2">
          <div class="min-w-0">
            <div class="flex items-center gap-2 text-[11px] uppercase tracking-[0.12em] text-gray-500">
              <span>{{ group.source }}</span>
              <span>{{ group.operation }}</span>
              <span>{{ group.scriptKind }}</span>
            </div>
            <div class="mt-1 truncate text-sm text-gray-200">
              {{ group.title || group.path || 'Untitled operation' }}
            </div>
          </div>
          <div class="shrink-0 text-[11px] text-gray-500">
            {{ formatTimestamp(group.timestamp) }}
          </div>
        </div>

        <div class="px-3 py-2 font-mono text-xs leading-5">
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

<script setup>
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'

const reportsStore = useWorkbenchReportsStore()

function lineClass(level) {
  if (level === 'error') return 'text-[#f48771]'
  if (level === 'warning') return 'text-[#dcdcaa]'
  if (level === 'success') return 'text-[#89d185]'
  return 'text-gray-200'
}

function formatTimestamp(timestamp) {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}
</script>
