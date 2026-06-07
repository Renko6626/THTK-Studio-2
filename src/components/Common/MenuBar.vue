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

<script setup>
import { computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { NDropdown, useMessage, useDialog } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useBuildDialogStore } from '../../stores/buildDialog'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { dispatchEditorAction } from '../../composables/useEditorActionBridge'
import { useFileOperations } from '../../composables/useFileOperations'
import { generateAiAssistPack } from '../../api'

const editorStore = useEditorStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const buildDialogStore = useBuildDialogStore()
const toolchainSettingsStore = useToolchainSettingsStore()
const reportsStore = useWorkbenchReportsStore()
const { handleCreate } = useFileOperations()
const message = useMessage()
const dialog = useDialog()

const hasWorkspace = computed(() => Boolean(projectStore.rootPath))
const hasActiveTab = computed(() => Boolean(editorStore.activeTab))
const hasEditableTab = computed(() => editorStore.activeTab?.viewType !== 'binary-script' && Boolean(editorStore.activeTab))
const activeExtension = computed(() => editorStore.activeTab?.path?.split('.').pop()?.toLowerCase() || '')
const canCompileActiveSource = computed(() => activeExtension.value === 'decl')
const canDecompileActiveBinary = computed(() => activeExtension.value === 'ecl')
const canGenerateActiveHeader = computed(() => activeExtension.value === 'decl')

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
      { label: '编译当前 ECL 源文件', key: 'script.compileEcl', disabled: !canCompileActiveSource.value },
      { label: '反编译当前 ECL 二进制', key: 'script.decompileEcl', disabled: !canDecompileActiveBinary.value },
      { label: '为当前 ECL 生成头文件', key: 'script.generateHeader', disabled: !canGenerateActiveHeader.value },
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
  const selected = await open({ directory: true })
  if (selected) {
    await projectStore.loadProject(selected)
    message.success(`已打开 ${projectStore.rootName}`)
  }
}

function openTheclDialog(mode) {
  buildDialogStore.openTheclDialog({
    mode,
    inputPath: editorStore.activeTab?.path || '',
    useShiftJis: true
  })
}

function publishAiPackResult({ success, path, message: text }) {
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

async function handleSelect(key) {
  switch (key) {
    case 'file.openFolder':
      await openFolder()
      break
    case 'file.newFile':
      handleCreate(projectStore.rootPath, 'file')
      break
    case 'file.newFolder':
      handleCreate(projectStore.rootPath, 'dir')
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
    case 'script.compileEcl':
      openTheclDialog('compile')
      break
    case 'script.decompileEcl':
      openTheclDialog('decompile')
      break
    case 'script.generateHeader':
      openTheclDialog('header')
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
