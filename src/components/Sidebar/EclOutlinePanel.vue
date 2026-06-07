<template>
  <div class="h-full min-w-0 flex flex-col">
    <div class="h-8 shrink-0 flex items-center justify-between gap-3 mb-3">
      <div class="text-xs uppercase tracking-[0.08em] text-gray-500 leading-none">
        ECL 大纲
      </div>
      <button
        type="button"
        class="h-6 shrink-0 px-2 rounded-sm border border-transparent bg-transparent text-[11px] text-gray-500 leading-none hover:text-gray-200 hover:border-[#3b82f6]/60"
        @click="toggleAllSections"
      >
        {{ allCollapsed ? '展开全部' : '折叠全部' }}
      </button>
    </div>

    <div class="mb-3 shrink-0">
      <input
        ref="searchInput"
        v-model="searchQuery"
        type="text"
        placeholder="搜索函数、全局或标签"
        class="w-full h-8 px-3 rounded border border-white/8 bg-[#1a1a1a] text-sm text-gray-100 outline-none placeholder:text-gray-500 focus:border-[#3b82f6]/70"
      >
    </div>

    <div v-if="!isEclTab" class="text-sm text-gray-500 leading-6">
      当前活动标签不是 ECL 源文件，无法显示函数和标签大纲。
    </div>

    <div v-else class="flex-1 min-h-0 min-w-0 overflow-auto space-y-3 pr-1">
      <section
        v-for="section in sections"
        :key="section.key"
        class="rounded border border-white/8 bg-[#1c1c1c] overflow-hidden"
      >
        <button
          type="button"
          class="w-full px-3 py-2 flex items-center justify-between gap-3 bg-[#1c1c1c] border-b border-transparent hover:border-[#3b82f6]/35 hover:bg-[#202224]"
          @click="toggleSection(section.key)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-[11px] text-gray-500 shrink-0">
              {{ isCollapsed(section.key) ? '▸' : '▾' }}
            </span>
            <span class="text-[11px] uppercase tracking-[0.08em] text-gray-300 truncate">
              {{ section.label }}
            </span>
          </div>
          <span class="text-[11px] px-1.5 py-0.5 rounded border border-white/8 text-gray-400 shrink-0">
            {{ section.items.length }}
          </span>
        </button>

        <div v-if="!isCollapsed(section.key)" class="px-3 py-2 bg-[#181818] space-y-1">
          <div v-if="!section.items.length" class="px-2 py-1 text-xs text-gray-600">
            无
          </div>

          <button
            v-for="item in section.items"
            :key="`${section.key}:${item.name}:${item.line}`"
            type="button"
            class="w-full text-left px-3 py-1.5 rounded-sm border border-transparent flex items-center justify-between gap-3 bg-transparent hover:border-[#3b82f6]/45 hover:bg-[#202224]"
            :class="itemClasses(item)"
            @click="revealSymbol(item)"
          >
            <div class="min-w-0 flex flex-col">
              <span class="truncate text-sm text-gray-100">
                <template v-for="(segment, index) in highlightSegments(item.name)" :key="`${item.name}-${index}`">
                  <span
                    :class="segment.match ? 'bg-[#264f78] text-white rounded px-[1px]' : ''"
                  >{{ segment.text }}</span>
                </template>
              </span>
              <span v-if="item.detail" class="truncate text-[11px] text-gray-500">{{ item.detail }}</span>
            </div>
            <span class="text-[11px] text-gray-500 shrink-0">Ln {{ item.line }}</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { collectEclDocumentSymbolEntriesFromText } from '../../services/languages/ecl/document-symbols'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'

const editorStore = useEditorStore()
const searchInput = ref(null)
const collapsedSections = reactive({
  subroutines: false,
  globals: false,
  labels: false
})
const searchQuery = ref('')
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

const symbols = computed(() => {
  if (!isEclTab.value) {
    return { subroutines: [], globals: [], labels: [] }
  }
  return collectEclDocumentSymbolEntriesFromText(activeTab.value?.content || '')
})

function matchesSearch(item, query) {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return true

  return [item.name, item.detail, String(item.line)]
    .filter(Boolean)
    .some((value) => String(value).toLowerCase().includes(normalizedQuery))
}

const sections = computed(() => ([
  {
    key: 'subroutines',
    label: '函数 / 子程序',
    items: symbols.value.subroutines.filter((item) => matchesSearch(item, searchQuery.value))
  },
  {
    key: 'globals',
    label: '全局定义',
    items: symbols.value.globals.filter((item) => matchesSearch(item, searchQuery.value))
  },
  {
    key: 'labels',
    label: '标签',
    items: symbols.value.labels.filter((item) => matchesSearch(item, searchQuery.value))
  }
]))

const allCollapsed = computed(() => sections.value.every((section) => collapsedSections[section.key]))
const activeSymbol = computed(() => {
  if (!activeTab.value?.path || cursorPosition.path !== activeTab.value.path) {
    return null
  }

  const allItems = [
    ...symbols.value.subroutines,
    ...symbols.value.globals,
    ...symbols.value.labels
  ].sort((left, right) => left.line - right.line || left.column - right.column)

  let current = null
  for (const item of allItems) {
    if (item.line <= cursorPosition.line) {
      current = item
      continue
    }
    break
  }

  return current
})

watch(isEclTab, async (value) => {
  if (!value) return
  await nextTick()
  searchInput.value?.focus()
}, { immediate: true })

function handleCursorPosition(event) {
  const detail = event.detail || {}
  cursorPosition.path = String(detail.path || '')
  cursorPosition.line = Math.max(1, Number(detail.line || 1))
  cursorPosition.column = Math.max(1, Number(detail.column || 1))
}

function isCollapsed(key) {
  return Boolean(collapsedSections[key])
}

function toggleSection(key) {
  collapsedSections[key] = !collapsedSections[key]
}

function toggleAllSections() {
  const nextValue = !allCollapsed.value
  sections.value.forEach((section) => {
    collapsedSections[section.key] = nextValue
  })
}

function itemClasses(item) {
  return activeSymbol.value?.name === item.name && activeSymbol.value?.line === item.line
    ? 'border-[#3b82f6] bg-transparent'
    : ''
}

function highlightSegments(text) {
  const query = searchQuery.value.trim()
  if (!query) return [{ text, match: false }]

  const source = String(text || '')
  const normalizedSource = source.toLowerCase()
  const normalizedQuery = query.toLowerCase()
  const segments = []
  let cursor = 0

  while (cursor < source.length) {
    const matchIndex = normalizedSource.indexOf(normalizedQuery, cursor)
    if (matchIndex === -1) {
      segments.push({ text: source.slice(cursor), match: false })
      break
    }

    if (matchIndex > cursor) {
      segments.push({ text: source.slice(cursor, matchIndex), match: false })
    }

    segments.push({
      text: source.slice(matchIndex, matchIndex + query.length),
      match: true
    })

    cursor = matchIndex + query.length
  }

  return segments.filter((segment) => segment.text.length > 0)
}

function revealSymbol(symbol) {
  if (!activeTab.value?.path) return
  dispatchEditorRevealLocation({
    path: activeTab.value.path,
    line: symbol.line,
    column: symbol.column
  })
}

onMounted(() => {
  window.addEventListener('thtk:editor-cursor-position', handleCursorPosition)
  nextTick(() => {
    searchInput.value?.focus()
  })
})

onBeforeUnmount(() => {
  window.removeEventListener('thtk:editor-cursor-position', handleCursorPosition)
})
</script>
