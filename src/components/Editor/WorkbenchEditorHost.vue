<template>
  <div v-if="resolvedView" class="w-full h-full min-h-0 flex flex-col">
  <TruthScriptToolbar v-if="truthSource" />
  <div class="flex-1 min-h-0">
  <KeepAlive>
    <component
      :is="resolvedView.component"
      v-if="resolvedView"
      class="w-full h-full"
      @update-cursor="$emit('update-cursor', $event)"
    />
  </KeepAlive>
  </div>
  </div>

  <!-- 没有工作区 → 欢迎页；有工作区但没开标签 → 轻量空状态，不重复欢迎页 -->
  <WelcomeView v-if="!resolvedView && !projectStore.rootPath" />

  <EmptyEditorState v-else-if="!resolvedView" />
</template>

<script setup lang="ts">
import TruthScriptToolbar from './TruthScriptToolbar.vue'
import { truthKindForPath } from '../../services/languages/truth/kinds'
import { computed } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { resolveEditorView } from '../../services/workbench/editorViews'
import EmptyEditorState from './EmptyEditorState.vue'
import WelcomeView from '../Welcome/WelcomeView.vue'

defineEmits<{
  'update-cursor': [stats: { line: number; col: number }]
}>()

const editorStore = useEditorStore()
const projectStore = useProjectStore()

const truthSource = computed(() => truthKindForPath(editorStore.activeTab?.path))
const resolvedView = computed(() => resolveEditorView(editorStore.activeTab?.viewType))
</script>
