<template>
  <div class="h-full min-w-0 flex flex-col">
    <div class="h-8 shrink-0 flex items-center justify-between gap-3 mb-3">
      <div class="text-xs uppercase tracking-[0.08em] text-gray-500 leading-none">
        ECL 引用
      </div>
      <div class="shrink-0 text-[11px] text-gray-500 leading-none">
        {{ symbolSummary }}
      </div>
    </div>

    <div v-if="!isEclTab" class="text-sm text-gray-500 leading-6">
      当前活动标签不是 ECL 源文件，无法显示引用结果。
    </div>

    <div v-else-if="!currentWord" class="text-sm text-gray-500 leading-6">
      将光标移动到子程序、标签或全局定义名称上，即可查看当前文件中的引用。
    </div>

    <div v-else-if="!definition" class="text-sm text-gray-500 leading-6">
      当前光标所在的标识符不是可追踪的 ECL 定义。
    </div>

    <div v-else class="flex-1 min-h-0 min-w-0 overflow-auto space-y-3 pr-1">
      <section class="rounded border border-white/8 bg-[#1c1c1c] overflow-hidden">
        <div class="px-3 py-2 border-b border-white/8 bg-[#1c1c1c]">
          <div class="flex items-center gap-2">
            <span class="rounded-sm px-1.5 py-0.5 text-[10px] font-semibold tracking-[0.08em] uppercase border" :class="kindClass(definition.kind)">
              {{ kindLabel(definition.kind) }}
            </span>
            <span class="text-sm text-gray-100 truncate">{{ definition.name }}</span>
          </div>
          <div class="mt-1 text-[11px] text-gray-500">
            定义位于第 {{ definition.line }} 行，共 {{ references.length }} 处引用/定义
          </div>
        </div>

        <div class="px-2 py-2 bg-[#181818] space-y-1">
          <button
            v-for="reference in references"
            :key="`${reference.kind}:${reference.name}:${reference.line}:${reference.column}`"
            type="button"
            class="w-full text-left px-2 py-1.5 rounded-sm border border-transparent flex items-center justify-between gap-3 bg-transparent hover:border-[#3b82f6]/45 hover:bg-[#202224]"
            :class="referenceClasses(reference)"
            @click="revealReference(reference)"
          >
            <div class="min-w-0 flex flex-col">
              <span class="truncate text-sm text-gray-100">{{ buildReferencePreview(reference) }}</span>
              <span class="truncate text-[11px] text-gray-500">
                {{ reference.line === definition.line && reference.column === definition.column ? '定义位置' : '引用位置' }}
              </span>
            </div>
            <span class="text-[11px] text-gray-500 shrink-0">Ln {{ reference.line }}</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, reactive } from 'vue'
import { useEditorStore } from '../../stores/editor'
import {
  findEclDocumentDefinitionFromText,
  findEclDocumentReferencesFromText
} from '../../services/languages/ecl/document-symbols'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'

const editorStore = useEditorStore()
const cursorPosition = reactive({
  path: '',
  line: 1,
  column: 1
})

const activeTab = computed(() => editorStore.activeTab)
const isEclTab = computed(() => {
  const path = String(activeTab.value?.path || '').toLowerCase()
  return activeTab.value?.viewType === 'text' && (path.endsWith('.decl') || path.endsWith('.tecl'))
})

const currentLineText = computed(() => {
  if (!isEclTab.value || cursorPosition.path !== activeTab.value?.path) return ''
  const lines = String(activeTab.value?.content || '').split(/\r?\n/)
  return lines[cursorPosition.line - 1] || ''
})

const currentWord = computed(() => {
  const line = currentLineText.value
  const columnIndex = Math.max(0, cursorPosition.column - 1)
  const regex = /[A-Za-z_]\w*/g
  let match = regex.exec(line)

  while (match) {
    const start = match.index
    const end = start + match[0].length
    if (columnIndex >= start && columnIndex <= end) {
      return match[0]
    }
    match = regex.exec(line)
  }

  return ''
})

const definition = computed(() => {
  if (!isEclTab.value || !currentWord.value) return null
  return findEclDocumentDefinitionFromText(activeTab.value?.content || '', currentWord.value)
})

const references = computed(() => {
  if (!isEclTab.value || !currentWord.value) return []
  return findEclDocumentReferencesFromText(activeTab.value?.content || '', currentWord.value)
})

const symbolSummary = computed(() => {
  if (!definition.value) return '未选中符号'
  return `${kindLabel(definition.value.kind)} · ${references.value.length} 项`
})

function handleCursorPosition(event) {
  const detail = event.detail || {}
  cursorPosition.path = String(detail.path || '')
  cursorPosition.line = Math.max(1, Number(detail.line || 1))
  cursorPosition.column = Math.max(1, Number(detail.column || 1))
}

function kindLabel(kind) {
  if (kind === 'subroutine') return '子程序'
  if (kind === 'label') return '标签'
  if (kind === 'global') return '全局'
  return '符号'
}

function kindClass(kind) {
  if (kind === 'subroutine') return 'border-[#3b82f6]/45 text-[#9ecbff]'
  if (kind === 'label') return 'border-[#dcdcaa]/35 text-[#dcdcaa]'
  if (kind === 'global') return 'border-[#6a9955]/45 text-[#b5cea8]'
  return 'border-white/10 text-gray-300'
}

function referenceClasses(reference) {
  return reference.line === definition.value?.line && reference.column === definition.value?.column
    ? 'border-[#3b82f6] bg-transparent'
    : ''
}

function buildReferencePreview(reference) {
  const lines = String(activeTab.value?.content || '').split(/\r?\n/)
  return (lines[reference.line - 1] || '').trim()
}

function revealReference(reference) {
  if (!activeTab.value?.path) return
  dispatchEditorRevealLocation({
    path: activeTab.value.path,
    line: reference.line,
    column: reference.column
  })
}

onMounted(() => {
  window.addEventListener('thtk:editor-cursor-position', handleCursorPosition)
})

onBeforeUnmount(() => {
  window.removeEventListener('thtk:editor-cursor-position', handleCursorPosition)
})
</script>
