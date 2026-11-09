<template>
  <div class="h-full flex flex-col text-xs p-3">
    <div v-if="!supported" class="text-[#9d9d9d]">打开 truth MSG、END 或 STD 源码查看时序。</div>
    <template v-else>
      <div class="pb-2 text-[#9d9d9d]">按脚本列出源码事件；循环和分支内不推算运行时间。</div>
      <div v-if="error" class="text-[#f48771]">{{ error }}</div>
      <div class="overflow-auto">
        <div v-for="(event, index) in analysis.events" :key="index" class="py-1 cursor-pointer hover:bg-white/8" @click="reveal(event.line)">
          <span class="text-[#9d9d9d] mr-2">{{ event.script }}</span>
          <span class="text-[#4ec9b0] mr-2">{{ event.time === null ? '时间未求值' : `${event.time} 帧` }}</span>
          <span class="font-mono">{{ event.name }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { analyzeTruthSource } from '../../api'
import type { TruthAnalysis } from '../../types'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'
const editor = useEditorStore()
const supported = computed(() => /\.(msgspec|endspec|stdspec)$/i.test(editor.activeTab?.path || ''))
const analysis = ref<TruthAnalysis>({ scripts: [], constants: [], events: [], labels: [], jumps: [] })
const error = ref('')
watch([() => editor.activeTab?.content, () => editor.activeTab?.path], async (_, __, onCleanup) => {
  let active = true
  onCleanup(() => { active = false })
  analysis.value = { scripts: [], constants: [], events: [], labels: [], jumps: [] }
  error.value = ''
  if (!supported.value) return
  try { const value = await analyzeTruthSource(editor.activeTab?.content || ''); if (active) analysis.value = value }
  catch (e) { if (active) error.value = String(e) }
}, { immediate: true })
function reveal(line: number) { if (editor.activeTab?.path) dispatchEditorRevealLocation({ path: editor.activeTab.path, line, column: 1 }) }
</script>
