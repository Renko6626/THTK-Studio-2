<template>
  <div class="h-full flex flex-col text-xs">
    <div class="mb-3 text-gray-400">{{ mode === 'outline' ? '脚本与标签' : '标签引用' }}</div>
    <div v-if="error" class="text-[#f48771]">{{ error }}</div>
    <div v-else-if="!data" class="text-gray-500">正在读取源码结构…</div>
    <div v-else class="overflow-auto space-y-2">
      <template v-if="mode === 'outline'">
        <div v-for="constant in data.constants" :key="`const-${constant.line}`"><button class="text-[#4fc1ff]" @click="reveal(constant.line)">const {{ constant.name }}</button></div>
        <div v-for="script in data.scripts" :key="script.line">
          <button class="text-[#dcdcaa] hover:underline" @click="reveal(script.line)">script {{ script.name }}</button>
          <div v-for="label in data.labels.filter(l => l.script === script.name)" :key="label.line" class="pl-4 py-1">
            <button class="text-[#d7ba7d] hover:underline" @click="reveal(label.line)">{{ label.name }} <span class="text-gray-500">第 {{ label.line }} 行</span></button>
          </div>
        </div>
      </template>
      <template v-else>
        <div v-if="!selectedLabel" class="text-gray-500">将光标放到标签或 goto 目标上查看引用。</div>
        <template v-else>
          <button class="text-[#d7ba7d]" @click="reveal(selectedLabel.line)">{{ selectedLabel.script }} / {{ selectedLabel.name }}：定义于第 {{ selectedLabel.line }} 行</button>
          <div v-for="jump in references" :key="`${jump.line}:${jump.column}`">
            <button class="text-gray-300 hover:underline" @click="reveal(jump.line, jump.column)">第 {{ jump.line }} 行</button>
          </div>
          <div v-if="!references.length" class="text-gray-500">当前文件中没有 goto 引用。</div>
        </template>
      </template>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { analyzeTruthSource } from '../../api'
import type { TruthAnalysis } from '../../types'
import { dispatchEditorRevealLocation } from '../../composables/useEditorActionBridge'
defineProps<{ mode: 'outline' | 'references' }>()
const editor = useEditorStore(), data = ref<TruthAnalysis | null>(null), error = ref('')
watch([() => editor.activeTab?.path, () => editor.activeTab?.content], async (_, __, cleanup) => {
  let active = true; cleanup(() => { active = false }); data.value = null; error.value = ''
  try { const result = await analyzeTruthSource(editor.activeTab?.content || ''); if (active) data.value = result }
  catch (e) { if (active) error.value = String(e) }
}, { immediate: true })
const cursor = ref<{path:string;line:number;column:number} | null>(null)
function cursorChanged(event: Event) { cursor.value = (event as CustomEvent).detail }
onMounted(() => window.addEventListener('thtk:editor-cursor-position', cursorChanged))
onBeforeUnmount(() => window.removeEventListener('thtk:editor-cursor-position', cursorChanged))
const selectedLabel = computed(() => {
  const position = cursor.value
  if (!position || position.path !== editor.activeTab?.path || !data.value) return null
  const line = editor.activeTab?.content.split(/\r?\n/)[position.line - 1] || ''
  const word = /[A-Za-z_]\w*/g
  let match: RegExpExecArray | null
  while ((match = word.exec(line))) {
    if (position.column >= match.index + 1 && position.column <= match.index + match[0].length + 1) {
      const scope = data.value.scripts.find(s => position.line >= s.line && position.line <= s.endLine)
      return data.value.labels.find(l => l.script === scope?.name && l.name === match![0]) || null
    }
  }
  return null
})
const references = computed(() => data.value?.jumps.filter(j => j.targetLine === selectedLabel.value?.line) || [])
function reveal(line: number, column = 1) { if (editor.activeTab?.path) dispatchEditorRevealLocation({ path: editor.activeTab.path, line, column }) }
</script>
