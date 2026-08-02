<template>
  <div class="h-8 px-2 flex items-center justify-between bg-[#181818] border-b border-white/6 text-[12px] text-gray-300 select-none">
    <div class="flex items-center gap-1">
      <n-dropdown
        v-for="menu in menus"
        :key="menu.key"
        trigger="hover"
        placement="bottom-start"
        :options="menu.options"
        @select="handleSelect"
      >
        <div class="px-2 h-6 rounded flex items-center hover:bg-white/8 cursor-default">
          {{ menu.label }}
        </div>
      </n-dropdown>
    </div>

    <div class="text-[11px] text-gray-500 truncate max-w-[16rem] px-2">
      {{ projectStore.rootName || 'THTK-Studio' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NDropdown, useMessage, useDialog } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useProjectSettingsStore } from '../../stores/projectSettings'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { dispatchEditorAction } from '../../composables/useEditorActionBridge'
import { useFileOperations } from '../../composables/useFileOperations'
import { useProjectActions } from '../../composables/useProjectActions'
import { useToolchainActions } from '../../composables/useToolchainActions'
import { generateAiAssistPack } from '../../api'

const editorStore = useEditorStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const projectSettingsStore = useProjectSettingsStore()
const toolchainSettingsStore = useToolchainSettingsStore()
const reportsStore = useWorkbenchReportsStore()
const { handleCreate } = useFileOperations()
const message = useMessage()
const dialog = useDialog()
const tcActions = useToolchainActions({ message })
const projectActions = useProjectActions({ message, dialog })

const hasWorkspace = computed(() => Boolean(projectStore.rootPath))
const hasActiveTab = computed(() => Boolean(editorStore.activeTab))
const hasEditableTab = computed(() => editorStore.activeTab?.viewType !== 'binary-script' && Boolean(editorStore.activeTab))
const activeExtension = computed(() => editorStore.activeTab?.path?.split('.').pop()?.toLowerCase() || '')
const activeIsEcl  = computed(() => activeExtension.value === 'ecl')
const activeIsDecl = computed(() => activeExtension.value === 'decl')
const activeIsMsg  = computed(() => activeExtension.value === 'msg')
const activeIsDmsg = computed(() => activeExtension.value === 'dmsg')
const activeIsStd  = computed(() => activeExtension.value === 'std')
const activeIsDstd = computed(() => activeExtension.value === 'dstd')
const activeIsDat  = computed(() => activeExtension.value === 'dat')

const menus = computed(() => [
  {
    key: 'file',
    label: '文件',
    options: [
      { label: '打开文件夹...', key: 'file.openFolder' },
      { type: 'divider', key: 'file.divider.1' },
      { label: '新建文件', key: 'file.newFile', disabled: !hasWorkspace.value },
      { label: '新建文件夹', key: 'file.newFolder', disabled: !hasWorkspace.value },
      { type: 'divider', key: 'file.divider.2' },
      { label: '保存', key: 'file.save', disabled: !hasEditableTab.value },
      { label: '全部保存', key: 'file.saveAll', disabled: !editorStore.tabs.length },
      { label: '关闭当前标签页', key: 'file.closeActive', disabled: !hasActiveTab.value },
      { type: 'divider', key: 'file.divider.3' },
      { label: '项目设置...', key: 'file.projectSettings', disabled: !hasWorkspace.value },
      { label: '工具链设置...', key: 'file.toolchainSettings' },
      { type: 'divider', key: 'file.divider.4' },
      { label: '刷新资源管理器', key: 'file.refresh', disabled: !hasWorkspace.value }
    ]
  },
  {
    key: 'edit',
    label: '编辑',
    options: [
      { label: '撤销', key: 'edit.undo', disabled: !hasActiveTab.value },
      { label: '重做', key: 'edit.redo', disabled: !hasEditableTab.value },
      { type: 'divider', key: 'edit.divider.1' },
      { label: '查找', key: 'edit.find', disabled: !hasEditableTab.value },
      { label: '替换', key: 'edit.replace', disabled: !hasEditableTab.value },
      { label: '查找下一个', key: 'edit.findNext', disabled: !hasEditableTab.value },
      { label: '查找上一个', key: 'edit.findPrevious', disabled: !hasEditableTab.value }
    ]
  },
  {
    key: 'selection',
    label: '选择',
    options: [
      { label: '全选', key: 'selection.selectAll', disabled: !hasEditableTab.value }
    ]
  },
  {
    key: 'view',
    label: '视图',
    options: [
      { label: workbenchPanelsStore.bottomVisible ? '隐藏底部面板' : '显示底部面板', key: 'view.toggleBottomPanel' },
      { label: workbenchPanelsStore.rightVisible ? '隐藏右侧边栏' : '显示右侧边栏', key: 'view.toggleRightSidebar' },
      { label: workbenchPanelsStore.minimapVisible ? '隐藏代码缩略图' : '显示代码缩略图', key: 'view.toggleMinimap' },
      { type: 'divider', key: 'view.divider.1' },
      { label: '显示终端', key: 'view.showTerminal' },
      { label: '显示输出', key: 'view.showOutput' },
      { label: '显示问题', key: 'view.showProblems' }
    ]
  },
  {
    key: 'script',
    label: '脚本',
    options: [
      // ECL
      { label: '反编译当前 .ecl', key: 'script.decompileEclQuick', disabled: !activeIsEcl.value },
      { label: '反编译当前 .ecl(高级…)', key: 'script.decompileEclAdvanced', disabled: !activeIsEcl.value },
      { label: '编译当前 .decl', key: 'script.compileEclQuick', disabled: !activeIsDecl.value },
      { label: '编译当前 .decl(高级…)', key: 'script.compileEclAdvanced', disabled: !activeIsDecl.value },
      { label: '生成 ECL 头文件…', key: 'script.generateEclHeader', disabled: !activeIsDecl.value },
      { type: 'divider', key: 'script-msg-div' },
      // MSG
      { label: '反编译当前 .msg', key: 'script.decompileMsg', disabled: !activeIsMsg.value },
      { label: '编译当前 .dmsg', key: 'script.compileMsg', disabled: !activeIsDmsg.value },
      { type: 'divider', key: 'script-std-div' },
      // STD
      { label: '反编译当前 .std', key: 'script.decompileStd', disabled: !activeIsStd.value },
      { label: '编译当前 .dstd', key: 'script.compileStd', disabled: !activeIsDstd.value },
      { type: 'divider', key: 'script-dat-div' },
      // DAT
      { label: '解包当前 .dat', key: 'script.extractDat' },
      { label: '打包目录为 .dat', key: 'script.packDat' },
      { type: 'divider', key: 'script-ai-div' },
      // AI pack
      { label: '生成 AI 辅助包', key: 'script.generateAiPack', disabled: !hasWorkspace.value }
    ]
  },
  {
    key: 'terminal',
    label: '终端',
    options: [
      { label: '新建终端', key: 'terminal.new' }
    ]
  }
])

async function openFolder() {
  await projectActions.openProjectFromPicker()
}

function publishAiPackResult({
  success,
  path,
  message: text
}: {
  success: boolean
  path: string | null
  message: string
}) {
  reportsStore.publishToolResult({
    ownerKey: 'ecl:ai-pack',
    source: 'toolchain',
    operation: 'ai-pack',
    scriptKind: 'ecl',
    title: '生成 AI 辅助包',
    path,
    success,
    message: text,
    diagnostics: []
  })
  workbenchPanelsStore.showBottomPanel('output')
}

async function runGenerateAiPack() {
  try {
    const result = await generateAiAssistPack(false)
    const refLines = result.referenceFiles.map((file) => `已刷新 ${file}`)

    if (result.skillExisted && !result.skillWritten) {
      // references 已刷新，但保留了用户的 SKILL.md —— 先如实汇报，再询问是否覆盖。
      publishAiPackResult({
        success: true,
        path: result.skillPath,
        message: ['SKILL.md 已存在，保留用户版本', ...refLines].join('\n')
      })
      dialog.warning({
        title: 'SKILL.md 已存在',
        content:
          '该项目已有 SKILL.md。references 已刷新。是否用最新模板覆盖 SKILL.md?这会丢失你对 SKILL.md 的自定义修改。',
        positiveText: '覆盖',
        negativeText: '保留',
        onPositiveClick: async () => {
          try {
            const forced = await generateAiAssistPack(true)
            publishAiPackResult({
              success: true,
              path: forced.skillPath,
              message: [
                'SKILL.md 已用最新模板覆盖',
                ...forced.referenceFiles.map((file) => `已刷新 ${file}`)
              ].join('\n')
            })
          } catch (error) {
            publishAiPackResult({ success: false, path: null, message: String(error) })
          }
        }
      })
      return
    }

    // 全新生成（skillWritten）或无需写入的常规成功
    publishAiPackResult({
      success: true,
      path: result.skillPath,
      message: [result.skillWritten ? 'SKILL.md 已生成' : 'SKILL.md 已存在，保留用户版本', ...refLines].join('\n')
    })
  } catch (error) {
    publishAiPackResult({ success: false, path: null, message: String(error) })
  }
}

async function handleSelect(key: string) {
  switch (key) {
    case 'file.openFolder':
      await openFolder()
      break
    case 'file.newFile':
      if (projectStore.rootPath) handleCreate(projectStore.rootPath, 'file')
      break
    case 'file.newFolder':
      if (projectStore.rootPath) handleCreate(projectStore.rootPath, 'dir')
      break
    case 'file.save': {
      const ok = await editorStore.saveActiveFile()
      if (ok) message.success('已保存当前文件')
      else message.error('保存失败')
      break
    }
    case 'file.saveAll': {
      const ok = await editorStore.saveAllFiles()
      if (ok) message.success('已保存全部文件')
      else message.error('部分文件保存失败')
      break
    }
    case 'file.closeActive':
      if (editorStore.activeTab?.isDirty) {
        message.warning('当前标签存在未保存修改，请使用标签关闭按钮确认关闭')
      } else {
        editorStore.closeActiveTab()
      }
      break
    case 'file.projectSettings':
      projectSettingsStore.open()
      break
    case 'file.toolchainSettings':
      toolchainSettingsStore.open()
      break
    case 'file.refresh':
      await projectStore.refresh()
      break
    case 'edit.undo':
      dispatchEditorAction('undo')
      break
    case 'edit.redo':
      dispatchEditorAction('redo')
      break
    case 'edit.find':
      dispatchEditorAction('find')
      break
    case 'edit.replace':
      dispatchEditorAction('replace')
      break
    case 'edit.findNext':
      dispatchEditorAction('findNext')
      break
    case 'edit.findPrevious':
      dispatchEditorAction('findPrevious')
      break
    case 'selection.selectAll':
      dispatchEditorAction('selectAll')
      break
    case 'view.toggleBottomPanel':
      workbenchPanelsStore.toggleBottomPanel()
      break
    case 'view.toggleRightSidebar':
      workbenchPanelsStore.toggleRightPanel()
      break
    case 'view.toggleMinimap':
      workbenchPanelsStore.toggleMinimap()
      break
    case 'view.showTerminal':
      workbenchPanelsStore.showBottomPanel('terminal')
      break
    case 'view.showOutput':
      workbenchPanelsStore.showBottomPanel('output')
      break
    case 'view.showProblems':
      workbenchPanelsStore.showBottomPanel('problems')
      break
    case 'script.decompileEclQuick':
      tcActions.runDecompileEclQuick(editorStore.activeTab?.path)
      break
    case 'script.decompileEclAdvanced':
      tcActions.runDecompileEclAdvanced(editorStore.activeTab?.path)
      break
    case 'script.compileEclQuick':
      tcActions.runCompileEclQuick(editorStore.activeTab)
      break
    case 'script.compileEclAdvanced':
      tcActions.runCompileEclAdvanced(editorStore.activeTab)
      break
    case 'script.generateEclHeader':
      tcActions.runGenerateEclHeader(editorStore.activeTab)
      break
    case 'script.decompileMsg':
      tcActions.runDecompileMsg(editorStore.activeTab?.path)
      break
    case 'script.compileMsg':
      tcActions.runCompileMsg(editorStore.activeTab)
      break
    case 'script.decompileStd':
      tcActions.runDecompileStd(editorStore.activeTab?.path)
      break
    case 'script.compileStd':
      tcActions.runCompileStd(editorStore.activeTab)
      break
    case 'script.extractDat':
      tcActions.runExtractDat(activeIsDat.value ? editorStore.activeTab?.path : null)
      break
    case 'script.packDat':
      tcActions.runPackDat()
      break
    case 'script.generateAiPack':
      runGenerateAiPack()
      break
    case 'terminal.new':
      workbenchPanelsStore.showBottomPanel('terminal')
      terminalStore.openSession()
      break
  }
}
</script>
