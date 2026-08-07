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
      <WorkbenchEditorHost />
    </template>

    <template #bottom-panel>
      <BottomPanelHost />
    </template>

    <template #right-sidebar>
      <RightSidebar />
    </template>

    <!--
      不提供 statusbar 插槽：底部那条 24px 的蓝条挤占的是终端高度，而终端里
      常驻跑着 claude / codex 这类 agent，纵向空间比 Ln/Col、编码这些常驻信息
      值钱。WorkbenchLayout 的插槽本身保留（有 v-if $slots 守着，零成本），
      将来想加回来只需重新提供这个 template。
    -->
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
import { useProjectStore } from '../../stores/project'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useWorkbenchZoomStore } from '../../stores/workbenchZoom'
import { useBeforeUnloadGuard } from '../../composables/useBeforeUnloadGuard'
import { useEclSemanticVocabulary } from '../../composables/useEclSemanticVocabulary'
import { useFileWatcher } from '../../composables/useFileWatcher'
import { useMcpBridge } from '../../composables/useMcpBridge'
import { useProjectActions } from '../../composables/useProjectActions'
import { useWorkbenchSession } from '../../composables/useWorkbenchSession'
import { useWorkbenchShortcuts } from '../../composables/useWorkbenchShortcuts'

const editorStore = useEditorStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
// 恢复上次的缩放。放这里而不是各组件里，避免多处重复恢复
void useWorkbenchZoomStore().restore()

const message = useMessage()
const dialog = useDialog()
const projectActions = useProjectActions({ message, dialog })


/**
 * 外部改动导致文件重载时的提示。
 *
 * 原来显示在底部状态栏里 2.5 秒后自行消失；状态栏删掉后改走 toast——
 * 这条信息不能跟着栏一起没掉，用户需要知道自己看的内容被外部换过。
 */
function showReloadNotice(text: string) {
  message.info(text)
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

</script>
