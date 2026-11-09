<template>
  <div class="timeline-panel">
    <div v-if="!supported" class="timeline-notice">打开 MSG、END 或 STD 源码查看时间线。</div>
    <template v-else>
      <div class="timeline-toolbar">
        <span class="timeline-heading">{{ entries.length }} 个 entry</span>
        <button type="button" class="timeline-toolbar-action" :disabled="activeLine === null" @click="locateCursor">定位当前</button>
        <button type="button" class="timeline-toolbar-action" :disabled="!entries.length" @click="toggleEntries">{{ allEntriesCollapsed ? '展开全部' : '折叠全部' }}</button>
      </div>
      <div class="timeline-caption">源码时间与流程</div>
      <div v-if="error" class="timeline-error">{{ error }}</div>
      <div v-else-if="loading && !analysis" class="timeline-notice">正在读取脚本结构…</div>
      <div v-else-if="!entries.length" class="timeline-notice">当前源码中还没有可显示的 entry。</div>
      <div v-else ref="scrollArea" class="timeline-scroll" :aria-busy="loading">
        <section v-for="entry in entries" :key="`${entry.line}:${entry.name}`" class="timeline-entry">
          <div class="timeline-entry-heading" :class="{ 'timeline-entry-current': activeLine !== null && activeLine >= entry.line && activeLine <= entry.endLine }">
            <button type="button" class="timeline-entry-fold" :aria-expanded="!isEntryCollapsed(entry)" :aria-label="`${isEntryCollapsed(entry) ? '展开' : '折叠'} ${entry.name}`" @click="toggle(entryKey(entry))">
              <svg viewBox="0 0 16 16" width="14" height="14" :class="{ 'timeline-chevron-open': !isEntryCollapsed(entry) }" aria-hidden="true"><path d="m6 4 4 4-4 4" /></svg>
            </button>
            <button type="button" class="timeline-entry-name" :title="entry.name" @click="reveal(entry.line)">{{ entry.name }}</button>
            <span class="timeline-entry-line">L{{ entry.line }}</span>
          </div>
          <div v-if="!isEntryCollapsed(entry)" class="timeline-entry-body">
            <TruthTimelineNodes :nodes="entry.nodes" :collapsed="collapsed" :active-line="activeLine" @toggle="toggle" @reveal="reveal" />
            <div v-if="!entry.nodes.length" class="timeline-notice">暂无事件</div>
          </div>
        </section>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { analyzeTruthSource } from '../../api'
import type { TruthAnalysis, TruthTimelineEntry, TruthTimelineNode } from '../../types'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'
import TruthTimelineNodes from './TruthTimelineNodes.vue'

const editor = useEditorStore()
const supported = computed(() => /\.(msgspec|endspec|stdspec)$/i.test(editor.activeTab?.path || ''))
const analysis = ref<TruthAnalysis | null>(null)
const entries = computed(() => analysis.value?.timeline || [])
const error = ref(''), loading = ref(false)
const collapsed = ref(new Set<string>())
const cursor = ref<{ path: string; line: number } | null>(null)
const activeLine = computed(() => cursor.value?.path === editor.activeTab?.path ? cursor.value?.line ?? null : null)
const scrollArea = ref<HTMLElement | null>(null)

watch([() => editor.activeTab?.path, () => editor.activeTab?.content], async ([path], [oldPath], onCleanup) => {
  let current = true
  onCleanup(() => { current = false })
  if (path !== oldPath) { analysis.value = null; collapsed.value = new Set() }
  error.value = ''
  loading.value = false
  if (!supported.value) { analysis.value = null; return }
  loading.value = true
  try {
    const value = await analyzeTruthSource(editor.activeTab?.content || '')
    if (current) analysis.value = value
  } catch (e) { if (current) error.value = String(e) }
  finally { if (current) loading.value = false }
}, { immediate: true })

function entryKey(entry: TruthTimelineEntry) { return `entry:${entry.line}:${entry.name}` }
function isEntryCollapsed(entry: TruthTimelineEntry) { return collapsed.value.has(entryKey(entry)) }
const allEntriesCollapsed = computed(() => entries.value.length > 0 && entries.value.every(isEntryCollapsed))
function toggle(key: string) {
  const next = new Set(collapsed.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  collapsed.value = next
}
function toggleEntries() {
  const next = new Set(collapsed.value)
  const expand = allEntriesCollapsed.value
  for (const entry of entries.value) {
    if (expand) next.delete(entryKey(entry))
    else next.add(entryKey(entry))
  }
  collapsed.value = next
}
function cursorChanged(event: Event) { cursor.value = (event as CustomEvent<{ path: string; line: number }>).detail }
onMounted(() => window.addEventListener('thtk:editor-cursor-position', cursorChanged))
onBeforeUnmount(() => window.removeEventListener('thtk:editor-cursor-position', cursorChanged))

async function locateCursor() {
  const line = activeLine.value
  if (line === null) return
  const next = new Set(collapsed.value)
  const expand = (nodes: TruthTimelineNode[]) => {
    for (const node of nodes) {
      if (line >= node.line && line <= node.endLine) { next.delete(`node:${node.id}`); expand(node.children) }
    }
  }
  for (const entry of entries.value) {
    if (line >= entry.line && line <= entry.endLine) { next.delete(entryKey(entry)); expand(entry.nodes) }
  }
  collapsed.value = next
  await nextTick()
  const current = scrollArea.value?.querySelector('[aria-current="location"]') || scrollArea.value?.querySelector('.timeline-entry-current')
  current?.scrollIntoView({ block: 'nearest' })
}
function reveal(line: number) {
  if (editor.activeTab?.path) dispatchEditorRevealLocation({ path: editor.activeTab.path, line, column: 1 })
}
</script>

<style scoped>
.timeline-panel { height: 100%; min-height: 0; min-width: 0; display: flex; flex-direction: column; color: #ccc; font-size: 12px; }
button { appearance: none; border: 0; background: transparent; color: inherit; font: inherit; cursor: pointer; padding: 0; text-align: left; }
button:focus-visible { outline: 1px solid #0078d4; outline-offset: -1px; }
.timeline-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 5px 8px; flex-shrink: 0; min-height: 25px; padding-bottom: 3px; }
.timeline-heading { color: #9d9d9d; font-size: 11px; margin-right: auto; white-space: nowrap; }
.timeline-toolbar-action { color: #9d9d9d; font-size: 10px; white-space: nowrap; padding: 3px 0; }
.timeline-toolbar-action:hover:not(:disabled) { color: #e7e7e7; }
.timeline-toolbar-action:disabled { opacity: .4; cursor: default; }
.timeline-caption { color: #858585; font-size: 10px; padding-bottom: 5px; flex-shrink: 0; }
.timeline-scroll { flex: 1; min-height: 0; min-width: 0; overflow: auto; padding-right: 2px; }
.timeline-entry + .timeline-entry { margin-top: 10px; padding-top: 8px; border-top: 1px solid #454545; }
.timeline-entry-heading { display: flex; align-items: center; gap: 5px; min-width: 0; min-height: 24px; color: #e7e7e7; }
.timeline-entry-current { color: #9cdcfe; }
.timeline-entry-fold { width: 18px; height: 24px; flex: none; display: flex; align-items: center; justify-content: center; color: #9d9d9d; }
.timeline-entry-fold svg { fill: none; stroke: currentColor; stroke-width: 1.5; }
.timeline-chevron-open { transform: rotate(90deg); }
.timeline-entry-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; padding: 3px 0; }
.timeline-entry-name:hover { text-decoration: underline; }
.timeline-entry-line { color: #858585; font-size: 10px; white-space: nowrap; }
.timeline-entry-body { padding: 1px 0 0 5px; }
.timeline-notice { color: #9d9d9d; font-size: 12px; line-height: 20px; padding: 5px 0; }
.timeline-error { color: #f48771; overflow-wrap: anywhere; line-height: 20px; }
</style>
