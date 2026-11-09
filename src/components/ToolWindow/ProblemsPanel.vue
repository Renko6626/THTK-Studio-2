<template>
  <div class="h-full flex flex-col bg-[#1e1e1e] text-[#cccccc]">
    <div class="h-7 pl-5 pr-2 flex items-center justify-between shrink-0 text-[11px]">
      <div class="flex items-center gap-3 text-[#9d9d9d]">
        <span class="flex items-center gap-1"><n-icon :size="14" class="text-[#f48771]"><ErrorCircle16Regular /></n-icon>{{ reportsStore.errorCount }}</span>
        <span class="flex items-center gap-1"><n-icon :size="14" class="text-[#cca700]"><Warning16Regular /></n-icon>{{ reportsStore.warningCount }}</span>
      </div>
      <button type="button" class="panel-action" title="清空问题" aria-label="清空问题" @click="reportsStore.clearProblems()"><n-icon :size="16"><Delete16Regular /></n-icon></button>
    </div>
    <div class="flex-1 overflow-auto pb-2">
      <button v-for="problem in sortedProblems" :key="problem.id" type="button" class="problem-row" :title="`${problem.message}\n${problem.path || ''}\n${sourceDetail(problem)}`" @click="openProblem(problem)">
        <n-icon :size="16" :class="severityClass(problem.severity)">
          <ErrorCircle16Regular v-if="problem.severity === 'error'" />
          <Warning16Regular v-else-if="problem.severity === 'warning'" />
          <Info16Regular v-else />
        </n-icon>
        <span class="truncate">{{ problem.message }}</span>
        <span class="text-[11px] text-[#858585] shrink-0">{{ sourceLabel(problem.source) }}</span>
        <span class="truncate text-[11px] text-[#858585] max-w-[180px]">{{ problem.path?.split(/[\\/]/).pop() || '未知文件' }} ({{ problem.line }}, {{ problem.column || 1 }})</span>
      </button>
      <div v-if="!sortedProblems.length" class="px-5 py-3 text-xs text-[#9d9d9d]">当前工作区没有问题。</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui'
import { ErrorCircle16Regular, Warning16Regular, Info16Regular, Delete16Regular } from '@vicons/fluent'
import { computed, nextTick } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import type { ProblemEntry } from '../../stores/workbenchReports'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'

const editorStore = useEditorStore()
const reportsStore = useWorkbenchReportsStore()

const sortedProblems = computed(() => {
  const severityRank: Record<string, number> = { error: 0, warning: 1, info: 2 }
  return [...reportsStore.problemEntries].sort((a, b) => {
    return (
      (severityRank[a.severity] ?? 99) - (severityRank[b.severity] ?? 99) ||
      String(a.path || '').localeCompare(String(b.path || '')) ||
      a.line - b.line ||
      a.column - b.column
    )
  })
})

function severityClass(severity: string) {
  if (severity === 'error') return 'text-[#f48771]'
  if (severity === 'warning') return 'text-[#cca700]'
  return 'text-[#75beff]'
}

function sourceLabel(source: string) {
  if (source === 'thecl') return 'THECL'
  if (source === 'ecl-static') return 'STATIC'
  return String(source || 'system').toUpperCase()
}

function sourceDetail(problem: ProblemEntry) {
  if (problem.source === 'ecl-static') {
    return '本地静态检查'
  }

  if (problem.source === 'thecl') {
    return `thecl / ${problem.operation}`
  }

  return `${problem.source}/${problem.operation}`
}

async function openProblem(problem: ProblemEntry) {
  if (!problem.path) return

  await editorStore.openFile({
    path: problem.path,
    name: problem.path.split(/[\\/]/).pop(),
    extension: problem.path.split('.').pop()?.toLowerCase()
  })

  // 等待 Vue watch 触发 Monaco model 切换完成后再跳转
  await nextTick()

  dispatchEditorRevealLocation({
    path: problem.path,
    line: problem.line,
    column: problem.column || 1
  })
}
</script>

<style scoped>
.problem-row {
  appearance: none; border: 0; background: transparent; color: inherit;
  width: 100%; display: flex; align-items: center; gap: 8px;
  min-height: 28px; padding: 3px 20px; text-align: left;
  font: inherit; font-size: 12px; cursor: pointer; box-sizing: border-box;
}
.problem-row:hover { background: #2a2d2e; }
.problem-row:focus-visible { outline: 1px solid #0078d4; outline-offset: -1px; }
.problem-row > span:first-of-type { flex: 1; min-width: 0; }
</style>
