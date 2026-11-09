<template>
  <div class="shrink-0 flex flex-wrap items-center gap-x-3 gap-y-1 px-3 py-1.5 border-b border-white/8 bg-[#252526] text-xs">
    <span class="text-gray-200">{{ kind === 'ending' ? 'END 结局' : kind === 'std' ? 'STD 背景' : 'MSG 对话' }}</span>
    <GameVersionSummary compact />
    <span class="flex-1 min-w-[80px] truncate text-gray-400" :title="status">{{ status }}</span>
    <n-button size="tiny" :loading="checking" @click="check">检查</n-button>
    <n-button size="tiny" :loading="building || editor.compiling" @click="build">打包</n-button>
    <n-button size="tiny" :disabled="building || editor.compiling" @click="options">构建选项…</n-button>
  </div>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, useMessage } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import GameVersionSummary from '../Common/GameVersionSummary.vue'
import { useBuildDialogStore } from '../../stores/buildDialog'
import { useToolchainActions } from '../../composables/useToolchainActions'
import { useTruthDiagnostics } from '../../composables/useTruthDiagnostics'
import { truthKindForPath } from '../../services/languages/truth/kinds'
const editor = useEditorStore(), dialog = useBuildDialogStore()
const actions = useToolchainActions({ message: useMessage() })
const { check, checking, status } = useTruthDiagnostics()
const kind = computed(() => truthKindForPath(editor.activeTab?.path))
const building = ref(false)
async function build() {
  if (building.value) return
  building.value = true
  try { if (kind.value === 'std') await actions.runCompileStd(editor.activeTab); else await actions.runCompileMsg(editor.activeTab) }
  finally { building.value = false }
}
function options() {
  const path = editor.activeTab?.path
  if (!path) return
  if (kind.value === 'std') dialog.openDialog({ tool: 'trustd', mode: 'compile', inputPath: path })
  else dialog.openDialog({ tool: 'trumsg', mode: 'compile', inputPath: path, scriptKind: kind.value === 'ending' ? 'ending' : 'stage' })
}
</script>
