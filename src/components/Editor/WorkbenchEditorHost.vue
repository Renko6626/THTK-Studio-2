<template>
  <component
    :is="resolvedView.component"
    v-if="resolvedView"
    class="w-full h-full"
    @update-cursor="$emit('update-cursor', $event)"
  />

  <!-- 没有工作区 → 欢迎页；有工作区但没开标签 → 轻量空状态，不重复欢迎页 -->
  <WelcomeView v-else-if="!projectStore.rootPath" />

  <EmptyEditorState v-else />
</template>

<script setup>
import { computed } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { resolveEditorView } from '../../services/workbench/editorViews'
import EmptyEditorState from './EmptyEditorState.vue'
import WelcomeView from '../Welcome/WelcomeView.vue'

defineEmits(['update-cursor'])

const editorStore = useEditorStore()
const projectStore = useProjectStore()

const resolvedView = computed(() => resolveEditorView(editorStore.activeTab?.viewType))
</script>
