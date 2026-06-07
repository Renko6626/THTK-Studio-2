<template>
  <div class="h-9 bg-[#1e1e1e] flex items-center overflow-hidden border-b border-white/6 select-none relative z-10">
    <n-tabs
      v-if="editorStore.tabs.length > 0"
      v-model:value="editorStore.activePath"
      type="card"
      closable
      size="small"
      tab-style="min-width: 120px; height: 34px; border: none; background: transparent;"
      @close="handleClose"
    >
      <n-tab-pane
        v-for="tab in editorStore.tabs"
        :key="tab.path"
        :name="tab.path"
      >
        <template #tab>
          <div class="flex items-center gap-2 text-xs">
            <span class="inline-flex items-center justify-center flex-none">
              <img :src="getFileIconUrl(tab)" alt="" draggable="false" class="w-4 h-4 block" />
            </span>
            <span class="truncate" :class="{'text-white': tab.path === editorStore.activePath, 'text-gray-400': tab.path !== editorStore.activePath}">
              {{ tab.name }}
            </span>
            <div v-if="tab.isDirty" class="w-2 h-2 rounded-full bg-[#4fc1ff] ml-1"></div>
          </div>
        </template>
      </n-tab-pane>
    </n-tabs>
    
    <!-- 没有打开文件时的占位 -->
    <div v-else class="w-full h-full bg-[#1e1e1e]"></div>
  </div>
</template>

<script setup>
import { NTabs, NTabPane, useDialog } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import { getFileIconUrl } from '../../utils/renderFileIcon'

const editorStore = useEditorStore()
const dialog = useDialog()

function handleClose(path) {
  const tab = editorStore.tabs.find(item => item.path === path)
  if (!tab) return

  if (!tab.isDirty) {
    editorStore.closeTab(path)
    return
  }

  dialog.warning({
    title: '关闭未保存文件',
    content: `“${tab.name}” 还有未保存修改，仍然关闭吗？`,
    positiveText: '关闭',
    negativeText: '取消',
    onPositiveClick: () => {
      editorStore.closeTab(path)
    }
  })
}
</script>

<style>
/* 深度修改 Naive UI Tabs 样式以完全匹配 VSCode */
.n-tabs .n-tabs-nav {
  background-color: #1e1e1e;
}
.n-tabs .n-tabs-tab {
  background-color: #2d2d2d !important;
  border-right: 1px solid #252526 !important;
  border-radius: 0 !important;
  padding: 0 10px !important;
  max-width: 220px;
}
.n-tabs .n-tabs-tab--active {
  background-color: #1e1e1e !important;
  border-top: 2px solid #4fc1ff !important;
}
.n-tabs .n-tabs-tab__close {
  color: #8a8a8a !important;
}
.n-tabs .n-tabs-tab__label {
  overflow: hidden;
  width: 100%;
}
</style>
