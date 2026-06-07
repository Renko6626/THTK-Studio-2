<template>
  <component
    :is="resolvedView.component"
    v-if="resolvedView"
    class="w-full h-full"
    @update-cursor="$emit('update-cursor', $event)"
  />

  <EmptyEditorState v-else />
</template>

<script setup>
import { computed } from 'vue'
import { useEditorStore } from '../../stores/editor'
import { resolveEditorView } from '../../services/workbench/editorViews'
import EmptyEditorState from './EmptyEditorState.vue'

defineEmits(['update-cursor'])

const editorStore = useEditorStore()

const resolvedView = computed(() => resolveEditorView(editorStore.activeTab?.viewType))
</script>
