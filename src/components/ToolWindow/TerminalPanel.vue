<template>
  <div class="h-full flex flex-col bg-[#181818] border-t border-black">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="flex items-center gap-3 min-w-0">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-400">Terminal</span>
        <n-select
          v-model:value="terminalStore.shell"
          size="tiny"
          :options="shellOptions"
          class="w-28"
        />
        <span class="text-xs text-gray-500 truncate max-w-[22rem]">{{ terminalStore.cwd || 'No workspace opened' }}</span>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
          @click="terminalStore.clearOutput()"
        >
          清空
        </button>
        <button
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
          @click="workbenchPanelsStore.hideBottomPanel()"
        >
          隐藏
        </button>
      </div>
    </div>

    <div ref="outputRef" class="flex-1 overflow-auto px-3 py-2 font-mono text-xs leading-5 bg-[#111111]">
      <div
        v-for="line in terminalStore.lines"
        :key="line.id"
        class="whitespace-pre-wrap break-words"
        :class="lineClass(line.type)"
      >
        {{ line.text }}
      </div>
      <div v-if="terminalStore.isRunning" class="text-blue-300">running...</div>
    </div>

    <div class="p-2 border-t border-white/8 bg-[#161616]">
      <n-input
        v-model:value="command"
        type="text"
        size="small"
        :placeholder="placeholder"
        @keydown="handleKeydown"
      >
        <template #prefix>
          <span class="font-mono text-[11px] text-gray-500">{{ terminalStore.shell }}</span>
        </template>
      </n-input>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { NInput, NSelect } from 'naive-ui'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'

const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const command = ref('')
const outputRef = ref(null)

const shellOptions = [
  { label: 'PowerShell', value: 'pwsh' },
  { label: 'CMD', value: 'cmd' }
]

const placeholder = computed(() => {
  if (terminalStore.isRunning) return '命令执行中...'
  return terminalStore.cwd
    ? '输入命令后按 Enter 执行'
    : '先打开工作区，再执行项目相关命令'
})

function lineClass(type) {
  if (type === 'stderr') return 'text-[#f48771]'
  if (type === 'stdout') return 'text-gray-200'
  if (type === 'prompt') return 'text-[#4fc1ff]'
  return 'text-gray-500'
}

async function submit() {
  const value = command.value
  command.value = ''
  await terminalStore.execute(value)
}

function handleKeydown(event) {
  if (event.key === 'Enter') {
    event.preventDefault()
    submit()
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    command.value = terminalStore.navigateHistory('up', command.value)
    return
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    command.value = terminalStore.navigateHistory('down', command.value)
  }
}

watch(
  () => [terminalStore.lines.length, workbenchPanelsStore.bottomVisible, workbenchPanelsStore.activeBottomPanel],
  async () => {
    await nextTick()
    if (outputRef.value) {
      outputRef.value.scrollTop = outputRef.value.scrollHeight
    }
  }
)
</script>
