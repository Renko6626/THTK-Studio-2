<template>
  <div class="h-full flex flex-col bg-[#111111]">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="flex items-center gap-3 text-xs font-semibold uppercase tracking-wider text-gray-400">
        <span>Problems</span>
        <span class="text-[#f48771]">{{ reportsStore.errorCount }} Errors</span>
        <span class="text-[#dcdcaa]">{{ reportsStore.warningCount }} Warnings</span>
      </div>
      <button
        type="button"
        class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
        @click="reportsStore.clearProblems()"
      >
        清空
      </button>
    </div>

    <div class="flex-1 overflow-auto">
      <button
        v-for="problem in sortedProblems"
        :key="problem.id"
        type="button"
        class="w-full text-left px-3 py-2 border-b border-white/6 border-l-2 border-l-transparent bg-transparent hover:bg-[#1b1d1f] hover:border-l-[#3b82f6]"
        @click="openProblem(problem)"
      >
        <div class="flex items-center gap-2 text-sm">
          <span class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-[0.08em]" :class="severityClass(problem.severity)">
            {{ problem.severity.toUpperCase() }}
          </span>
          <span class="rounded-sm px-1.5 py-0.5 text-[10px] font-semibold tracking-[0.08em] uppercase" :class="sourceClass(problem.source)">
            {{ sourceLabel(problem.source) }}
          </span>
          <span class="text-gray-100 truncate">{{ problem.message }}</span>
        </div>
        <div class="mt-1 text-[11px] text-gray-500 flex items-center gap-3">
          <span>{{ sourceDetail(problem) }}</span>
          <span>{{ problem.scriptKind }}</span>
          <span class="truncate">{{ problem.path || 'Unknown file' }}</span>
          <span>Ln {{ problem.line }}, Col {{ problem.column || 1 }}</span>
        </div>
      </button>

      <div v-if="!sortedProblems.length" class="h-full flex items-center justify-center text-sm text-gray-500">
        暂无问题记录
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'

const editorStore = useEditorStore()
const reportsStore = useWorkbenchReportsStore()

const sortedProblems = computed(() => {
  const severityRank = { error: 0, warning: 1, info: 2 }
  return [...reportsStore.problemEntries].sort((a, b) => {
    return (
      (severityRank[a.severity] ?? 99) - (severityRank[b.severity] ?? 99) ||
      String(a.path || '').localeCompare(String(b.path || '')) ||
      a.line - b.line ||
      a.column - b.column
    )
  })
})

function severityClass(severity) {
  if (severity === 'error') return 'bg-[#5a1d1d] text-[#f48771]'
  if (severity === 'warning') return 'bg-[#4a3f14] text-[#dcdcaa]'
  return 'bg-white/10 text-gray-300'
}

function sourceLabel(source) {
  if (source === 'thecl') return 'THECL'
  if (source === 'ecl-static') return 'STATIC'
  return String(source || 'system').toUpperCase()
}

function sourceClass(source) {
  if (source === 'thecl') return 'border border-[#3b82f6]/45 text-[#9ecbff] bg-transparent'
  if (source === 'ecl-static') return 'border border-[#6a9955]/45 text-[#b5cea8] bg-transparent'
  return 'border border-white/10 text-gray-300 bg-transparent'
}

function sourceDetail(problem) {
  if (problem.source === 'ecl-static') {
    return '本地静态检查'
  }

  if (problem.source === 'thecl') {
    return `thecl / ${problem.operation}`
  }

  return `${problem.source}/${problem.operation}`
}

async function openProblem(problem) {
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
