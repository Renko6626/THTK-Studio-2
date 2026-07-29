<template>
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
  <ProjectSettingsDialog />
</template>

<script setup lang="ts">
/**
 * 工作台根组件。
 *
 * 刻意放在 App.vue 的 n-config/dialog/message provider **内部**：naive-ui 的
 * useMessage / useDialog 只能在 provider 的后代里调用，而 App.vue 自己的 setup
 * 在它自己的 provider 外面，取不到。工作台级的 composable（快捷键、会话恢复等）
 * 需要弹提示和确认框，所以整体下沉到这一层。
 */
import { computed, reactive, ref, onBeforeUnmount } from 'vue'
import { useDialog, useMessage } from 'naive-ui'
import MenuBar from '../Common/MenuBar.vue'
import WorkbenchLayout from './WorkbenchLayout.vue'
import FileTree from '../Sidebar/FileTree.vue'
import RightSidebar from '../Sidebar/RightSidebar.vue'
import TabGroup from '../Editor/TabGroup.vue'
import WorkbenchEditorHost from '../Editor/WorkbenchEditorHost.vue'
import BuildConfigDialog from '../Dialogs/BuildConfigDialog.vue'
import ToolchainSettingsDialog from '../Dialogs/ToolchainSettingsDialog.vue'
import ProjectSettingsDialog from '../Dialogs/ProjectSettingsDialog.vue'
import BottomPanelHost from '../ToolWindow/BottomPanelHost.vue'
import { useEditorStore } from '../../stores/editor'
import { useExplorerViewStore } from '../../stores/explorerView'
import { useProjectStore } from '../../stores/project'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useBeforeUnloadGuard } from '../../composables/useBeforeUnloadGuard'
import { useEclSemanticVocabulary } from '../../composables/useEclSemanticVocabulary'
import { useFileWatcher } from '../../composables/useFileWatcher'
import { useMcpBridge } from '../../composables/useMcpBridge'
import { useProjectActions } from '../../composables/useProjectActions'
import { useWorkbenchSession } from '../../composables/useWorkbenchSession'
import { useWorkbenchShortcuts } from '../../composables/useWorkbenchShortcuts'
import { resolveEditorView } from '../../services/workbench/editorViews'

const editorStore = useEditorStore()
const explorerViewStore = useExplorerViewStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()

const message = useMessage()
const dialog = useDialog()
const projectActions = useProjectActions({ message, dialog })

const cursorStats = reactive({ line: 1, col: 1 })
const reloadNotice = ref('')
let reloadNoticeTimer: number | null = null

const activeEditorModeLabel = computed(() => {
  const activeTab = editorStore.activeTab
  if (!activeTab) return 'TXT'
  return resolveEditorView(activeTab.viewType)?.statusLabel?.(activeTab) || 'TXT'
})

function updateCursorStats({ line, col }: { line: number; col: number }) {
  cursorStats.line = line
  cursorStats.col = col
}

function showReloadNotice(text: string) {
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
  showReloadNotice,
  openProjectPath: projectActions.openProjectPath
})

useWorkbenchShortcuts({
  editorStore,
  workbenchPanelsStore,
  showReloadNotice,
  // Ctrl+O 必须和菜单、文件树、欢迎页走同一条打开流程，否则快捷键会绕过脏标签保护
  openFolder: projectActions.openProjectFromPicker
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
