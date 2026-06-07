<template>
  <n-config-provider :theme="darkTheme">
    <n-dialog-provider>
      <n-message-provider>
        <WorkbenchLayout
          :show-left-sidebar="true"
          :show-right-sidebar="workbenchPanelsStore.rightVisible"
          :show-bottom-panel="workbenchPanelsStore.bottomVisible"
        >
          <template #left-sidebar>
            <FileTree />
          </template>

          <template #topbar>
            <div class="h-[68px] flex flex-col">
              <MenuBar />
              <TabGroup />
            </div>
          </template>

          <template #main>
            <WorkbenchEditorHost @update-cursor="updateCursorStats" />
          </template>

          <template #bottom-panel>
            <BottomPanelHost />
          </template>

          <template #right-sidebar>
            <RightSidebar />
          </template>

          <template #statusbar>
            <div class="h-full flex items-center px-3 text-xs justify-between select-none">
              <div class="flex items-center gap-4 overflow-hidden">
                <span class="hover:bg-white/10 px-1 rounded cursor-pointer">
                  Ln {{ cursorStats.line }}, Col {{ cursorStats.col }}
                </span>
                <span v-if="explorerViewStore.hasSelection">
                  Explorer: {{ explorerViewStore.selectionCount }} selected
                </span>
                <span class="truncate max-w-[26rem]">{{ projectStore.rootPath || 'No workspace' }}</span>
                <span v-if="reloadNotice" class="truncate max-w-[20rem] text-blue-100/95">
                  {{ reloadNotice }}
                </span>
              </div>
              <div class="flex items-center gap-4">
                <span v-if="terminalStore.sessionCount">{{ terminalStore.sessionCount }} 个终端</span>
                <span class="uppercase">{{ activeEditorModeLabel }}</span>
                <span>UTF-8</span>
              </div>
            </div>
          </template>
        </WorkbenchLayout>
        <BuildConfigDialog />
        <ToolchainSettingsDialog />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup>
import { computed, reactive, ref, onBeforeUnmount } from 'vue'
import { darkTheme } from 'naive-ui'
import { NMessageProvider, NDialogProvider, NConfigProvider } from 'naive-ui'
import MenuBar from './components/Common/MenuBar.vue'
import WorkbenchLayout from './components/Layout/WorkbenchLayout.vue'
import FileTree from './components/Sidebar/FileTree.vue'
import RightSidebar from './components/Sidebar/RightSidebar.vue'
import TabGroup from './components/Editor/TabGroup.vue'
import WorkbenchEditorHost from './components/Editor/WorkbenchEditorHost.vue'
import BuildConfigDialog from './components/Dialogs/BuildConfigDialog.vue'
import ToolchainSettingsDialog from './components/Dialogs/ToolchainSettingsDialog.vue'
import BottomPanelHost from './components/ToolWindow/BottomPanelHost.vue'
import { useEditorStore } from './stores/editor'
import { useExplorerViewStore } from './stores/explorerView'
import { useProjectStore } from './stores/project'
import { useTerminalStore } from './stores/terminal'
import { useWorkbenchPanelsStore } from './stores/workbenchPanels'
import { useBeforeUnloadGuard } from './composables/useBeforeUnloadGuard'
import { useEclSemanticVocabulary } from './composables/useEclSemanticVocabulary'
import { useFileWatcher } from './composables/useFileWatcher'
import { useMcpBridge } from './composables/useMcpBridge'
import { useWorkbenchSession } from './composables/useWorkbenchSession'
import { useWorkbenchShortcuts } from './composables/useWorkbenchShortcuts'
import { resolveEditorView } from './services/workbench/editorViews'

const editorStore = useEditorStore()
const explorerViewStore = useExplorerViewStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()

const cursorStats = reactive({ line: 1, col: 1 })
const reloadNotice = ref('')
let reloadNoticeTimer = null

const activeEditorModeLabel = computed(() => {
  const activeTab = editorStore.activeTab
  if (!activeTab) return 'TXT'
  return resolveEditorView(activeTab.viewType)?.statusLabel?.(activeTab) || 'TXT'
})

function updateCursorStats({ line, col }) {
  cursorStats.line = line
  cursorStats.col = col
}

function showReloadNotice(text) {
  reloadNotice.value = text
  if (reloadNoticeTimer) {
    window.clearTimeout(reloadNoticeTimer)
  }
  reloadNoticeTimer = window.setTimeout(() => {
    reloadNotice.value = ''
  }, 2500)
}

function hasDirtyTabs() {
  return editorStore.hasDirtyTabs
}

const { flushSnapshots } = useWorkbenchSession({
  projectStore,
  editorStore,
  terminalStore,
  workbenchPanelsStore,
  showReloadNotice
})

useWorkbenchShortcuts({
  editorStore,
  projectStore,
  workbenchPanelsStore,
  showReloadNotice
})

useEclSemanticVocabulary({
  projectStore,
  showReloadNotice
})

useFileWatcher({
  editorStore,
  projectStore,
  showReloadNotice
})

useMcpBridge()

useBeforeUnloadGuard({
  hasDirtyTabs,
  flushSnapshots
})

onBeforeUnmount(() => {
  if (reloadNoticeTimer) {
    window.clearTimeout(reloadNoticeTimer)
  }
})
</script>

<style>
/* 全局重置 */
body {
  margin: 0;
  padding: 0;
  background-color: #1e1e1e;
  overflow: hidden; /* 防止浏览器出现原生滚动条 */
}

/* 强制覆盖 naive-ui 可能产生的一点背景色差异 */
.n-layout-header, .n-layout-footer {
  box-sizing: border-box;
}

/* Monaco 的查找/替换说明浮层需要盖过顶部标签栏 */
.context-view,
.overflowingOverlayWidgets,
.overflowingContentWidgets,
.monaco-editor .find-widget,
.monaco-editor .suggest-widget,
.monaco-editor .monaco-hover {
  z-index: 3000 !important;
}
</style>
