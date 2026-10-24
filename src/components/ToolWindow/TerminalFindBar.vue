<template>
  <div class="shrink-0 h-9 px-2 flex items-center gap-2 border-b border-white/6 bg-[#202020] text-xs" role="search" aria-label="查找终端输出">
    <input
      ref="input"
      :value="query"
      class="min-w-0 flex-1 h-6 px-2 bg-[#303030] text-gray-100 border border-white/10 rounded outline-none focus:border-[#3b82f6]"
      placeholder="查找终端输出"
      aria-label="查找内容"
      @input="emit('update:query', ($event.target as HTMLInputElement).value)"
      @keydown="handleKeydown"
    />
    <label class="shrink-0 flex items-center gap-1 text-gray-400" title="区分大小写">
      <input type="checkbox" :checked="caseSensitive" @change="emit('update:caseSensitive', ($event.target as HTMLInputElement).checked)" />
      Aa
    </label>
    <span v-if="query && found === false" class="shrink-0 text-[#f48771]" role="status">无匹配</span>
    <button class="panel-action" title="上一项（Shift+Enter）" aria-label="上一项" :disabled="!query" @click="emit('search', 'previous')">↑</button>
    <button class="panel-action" title="下一项（Enter）" aria-label="下一项" :disabled="!query" @click="emit('search', 'next')">↓</button>
    <button class="panel-action" title="关闭查找（Escape）" aria-label="关闭查找" @click="emit('close')">×</button>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

defineProps<{ query: string; caseSensitive: boolean; found: boolean | null }>()
const emit = defineEmits<{
  'update:query': [value: string]
  'update:caseSensitive': [value: boolean]
  search: [direction: 'next' | 'previous']
  close: []
}>()
const input = ref<HTMLInputElement | null>(null)
function focus() {
  input.value?.focus()
}
function handleKeydown(event: KeyboardEvent) {
  if (event.isComposing) return
  if (event.key === 'Enter' || event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    if (event.key === 'Escape') emit('close')
    else emit('search', event.shiftKey ? 'previous' : 'next')
  }
}
onMounted(focus)
defineExpose({ focus })
</script>
