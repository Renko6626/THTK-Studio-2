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
import { generateTruthAssistPack } from '../../api'
import { computed, h, nextTick } from 'vue'
import { NDropdown, useMessage, useDialog } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useProjectSettingsStore } from '../../stores/projectSettings'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { useBuildDialogStore } from '../../stores/buildDialog'
import { useWorkbenchZoomStore } from '../../stores/workbenchZoom'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { dispatchTerminalAction, type TerminalAction } from '../../composables/useTerminalActionBridge'
import { dispatchEditorAction, editorActionIds, type EditorAction } from '../../composables/useEditorActionBridge'
import { useFileOperations } from '../../composables/useFileOperations'
import { useProjectActions } from '../../composables/useProjectActions'
import { useToolchainActions } from '../../composables/useToolchainActions'
import { useGameVersionsStore } from '../../stores/gameVersions'
import { toolAvailability } from '../../services/toolchains/gameVersions'
import { generateAiAssistPack } from '../../api'

const editorStore = useEditorStore()
const projectStore = useProjectStore()
const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const projectSettingsStore = useProjectSettingsStore()
const toolchainSettingsStore = useToolchainSettingsStore()
const buildDialogStore = useBuildDialogStore()
const zoomStore = useWorkbenchZoomStore()
const reportsStore = useWorkbenchReportsStore()
const { handleCreate } = useFileOperations()
const message = useMessage()
const dialog = useDialog()
const tcActions = useToolchainActions({ message })
const projectActions = useProjectActions({ message, dialog })

const hasWorkspace = computed(() => Boolean(projectStore.rootPath))

const gameVersionsStore = useGameVersionsStore()
gameVersionsStore.ensureLoaded()

/**
 * 五个工具支持的版本集合并不相同，所以按工具分别判断。
 *
 * 只看**项目声明的**版本，不回退全局默认：前端没有持有 AppConfig 的 store，
 * 为置灰这一件事去同步一份全局设置不划算。项目没声明版本时 toolAvailability
 * 会放行，用户仍会从后端拿到明确报错——置灰只在信息确凿时发生，
 * 这跟"信息不足一律放行"是同一条原则。
 */
const currentVersion = computed(() => projectStore.gameVersion || '')
const availability = computed(() => ({
  thecl: toolAvailability('thecl', currentVersion.value, gameVersionsStore.table),
  trumsg: toolAvailability('trumsg', currentVersion.value, gameVersionsStore.table),
  trustd: toolAvailability('trustd', currentVersion.value, gameVersionsStore.table),
  thdat: toolAvailability('thdat', currentVersion.value, gameVersionsStore.table)
}))
const hasActiveTab = computed(() => Boolean(editorStore.activeTab))
const hasEditableTab = computed(() => editorStore.activeTab?.viewType !== 'binary-script' && Boolean(editorStore.activeTab))
const activeExtension = computed(() => editorStore.activeTab?.path?.split('.').pop()?.toLowerCase() || '')
const activeIsEcl  = computed(() => activeExtension.value === 'ecl')
const activeIsDecl = computed(() => activeExtension.value === 'decl')
const activeIsMsg  = computed(() => activeExtension.value === 'msg')
const activeIsMsgSource = computed(() => ['msgspec', 'endspec'].includes(activeExtension.value))
const activeIsStd  = computed(() => activeExtension.value === 'std')
const activeIsStdSource = computed(() => activeExtension.value === 'stdspec')
const activeIsDat  = computed(() => activeExtension.value === 'dat')

/**
 * 生成一个受「当前版本是否被该工具支持」约束的菜单项。
 *
 * 两种禁用理由要区分开：`inactive` 是当前文件类型不对（用户切个标签就好，
 * 不需要解释），版本不支持则必须说清楚——否则用户面对一个灰掉的按钮
 * 完全不知道该改什么。后者把原因追加到 label 上，因为 naive-ui 的
 * dropdown 在 disabled 项上不显示 tooltip。
 */
function toolItem(
  toolId: 'thecl' | 'trumsg' | 'trustd' | 'thdat',
  label: string,
  key: string,
  inactive: boolean
) {
  const { enabled, reason } = availability.value[toolId]
  return {
    label: enabled ? label : `${label}（${reason}）`,
    key,
    disabled: inactive || !enabled
  }
}

const eclItem = (label: string, key: string, inactive: boolean) =>
  toolItem('thecl', label, key, inactive)

/** 用当前标签预填 MSG 构建对话框。编码留空即"跟随项目设置"。 */
function openMsgBuildDialog(mode: 'compile' | 'decompile') {
  const path = editorStore.activeTab?.path
  if (!path) return
  buildDialogStore.openDialog({ tool: 'trumsg', mode, inputPath: path } as never)
}

// 快捷键采用 Monaco 默认绑定；入口始终通过既有动作桥交给编辑器执行。
const isMac = navigator.userAgent.includes('Mac')
const isLinux = navigator.userAgent.includes('Linux')
const primaryKey = isMac ? '⌘' : 'Ctrl'
function editorItem(label: string, action: EditorAction, shortcut = '') {
  if (isLinux && action === 'copyLineDown') shortcut = 'Ctrl+Shift+Alt+↓'
  if (isMac && action === 'gotoLine') shortcut = 'Control+G'
  if (isMac && action === 'expandSelection') shortcut = 'Control+Shift+→'
  if (isMac && action === 'shrinkSelection') shortcut = 'Control+Shift+←'
  return {
    key: `editor.${action}`,
    disabled: !hasEditableTab.value,
    label: () => h('span', { class: 'flex items-center justify-between gap-8 w-full' }, [
      h('span', label),
      shortcut ? h('span', { class: 'text-[11px] text-gray-500' }, shortcut.replaceAll('Ctrl', primaryKey)) : null
    ])
  }
}

const terminalMenuActions: { label: string; action: TerminalAction }[] = [
  { label: '查找终端输出', action: 'find' },
  { label: '复制终端选区', action: 'copy' },
  { label: '粘贴到终端', action: 'paste' },
  { label: '全选终端输出', action: 'selectAll' },
  { label: '终端清屏', action: 'clear' },
  { label: '滚到底部', action: 'scrollBottom' },
  { label: '重命名终端', action: 'rename' },
  { label: '关闭终端', action: 'close' }
]

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
      editorItem('撤销', 'undo', 'Ctrl+Z'),
      editorItem('重做', 'redo', 'Ctrl+Shift+Z'),
      { type: 'divider', key: 'edit.divider.clipboard' },
      editorItem('剪切', 'cut', 'Ctrl+X'),
      editorItem('复制', 'copy', 'Ctrl+C'),
      editorItem('粘贴', 'paste', 'Ctrl+V'),
      { type: 'divider', key: 'edit.divider.find' },
      editorItem('查找', 'find', 'Ctrl+F'),
      editorItem('替换', 'replace', 'Ctrl+H'),
      editorItem('查找下一个', 'findNext', 'F3'),
      editorItem('查找上一个', 'findPrevious', 'Shift+F3'),
      { type: 'divider', key: 'edit.divider.lines' },
      editorItem('切换行注释', 'lineComment', 'Ctrl+/'),
      {
        label: '行操作', key: 'edit.lines', disabled: !hasEditableTab.value,
        children: [
          editorItem('增加缩进', 'indent', 'Ctrl+]'),
          editorItem('减少缩进', 'outdent', 'Ctrl+['),
          editorItem('向上移动行', 'moveLineUp', 'Alt+↑'),
          editorItem('向下移动行', 'moveLineDown', 'Alt+↓'),
          editorItem('向下复制行', 'copyLineDown', 'Shift+Alt+↓'),
          editorItem('删除行', 'deleteLine', 'Ctrl+Shift+K')
        ]
      }
    ]
  },
  {
    key: 'selection',
    label: '选择',
    options: [
      editorItem('全选', 'selectAll', 'Ctrl+A'),
      editorItem('选择整行', 'selectLine', 'Ctrl+L'),
      editorItem('选择下一个匹配项', 'selectNextMatch', 'Ctrl+D'),
      editorItem('选择全部匹配项', 'selectAllMatches', 'Ctrl+Shift+L'),
      { type: 'divider', key: 'selection.divider.cursors' },
      editorItem('在上方添加光标', 'addCursorAbove', isLinux ? 'Shift+Alt+↑' : 'Ctrl+Alt+↑'),
      editorItem('在下方添加光标', 'addCursorBelow', isLinux ? 'Shift+Alt+↓' : 'Ctrl+Alt+↓'),
      editorItem('扩大选区', 'expandSelection', 'Shift+Alt+→'),
      editorItem('缩小选区', 'shrinkSelection', 'Shift+Alt+←')
    ]
  },
  {
    key: 'view',
    label: '视图',
    options: [
      { label: workbenchPanelsStore.bottomVisible ? '隐藏底部面板' : '显示底部面板', key: 'view.toggleBottomPanel' },
      { label: workbenchPanelsStore.rightVisible ? '隐藏右侧边栏' : '显示右侧边栏', key: 'view.toggleRightSidebar' },
      { label: workbenchPanelsStore.minimapVisible ? '隐藏代码缩略图' : '显示代码缩略图', key: 'view.toggleMinimap' },
      { label: workbenchPanelsStore.wordWrap ? '关闭自动换行（Alt+Z）' : '开启自动换行（Alt+Z）', key: 'view.toggleWordWrap' },
      { label: workbenchPanelsStore.renderWhitespace ? '隐藏空白字符' : '显示空白字符', key: 'view.toggleWhitespace' },
      { type: 'divider', key: 'view.divider.editor' },
      editorItem('跳转到行 / 列', 'gotoLine', 'Ctrl+G'),
      editorItem('跳转到匹配括号', 'gotoBracket', 'Ctrl+Shift+\\'),
      editorItem('全部折叠', 'foldAll', 'Ctrl+K Ctrl+0'),
      editorItem('全部展开', 'unfoldAll', 'Ctrl+K Ctrl+J'),
      editorItem('编辑器命令面板', 'commandPalette', 'F1'),
      { type: 'divider', key: 'view.divider.zoom' },
      { label: `放大  (${zoomStore.label})`, key: 'view.zoomIn', disabled: !zoomStore.canZoomIn },
      { label: '缩小', key: 'view.zoomOut', disabled: !zoomStore.canZoomOut },
      { label: '重置缩放', key: 'view.zoomReset', disabled: zoomStore.level === 0 },
      { type: 'divider', key: 'view.divider.2' },
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
      eclItem('反编译当前 .ecl', 'script.decompileEclQuick', !activeIsEcl.value),
      eclItem('反编译当前 .ecl(高级…)', 'script.decompileEclAdvanced', !activeIsEcl.value),
      eclItem('编译当前 .decl', 'script.compileEclQuick', !activeIsDecl.value),
      eclItem('编译当前 .decl(高级…)', 'script.compileEclAdvanced', !activeIsDecl.value),
      eclItem('生成 ECL 头文件…', 'script.generateEclHeader', !activeIsDecl.value),
      { type: 'divider', key: 'script-msg-div' },
      // MSG
      toolItem('trumsg', '解包当前 .msg', 'script.decompileMsg', !activeIsMsg.value),
      toolItem('trumsg', '解包当前 .msg(高级…)', 'script.decompileMsgAdvanced', !activeIsMsg.value),
      toolItem('trumsg', '打包当前 MSG / END 源码', 'script.compileMsg', !activeIsMsgSource.value),
      toolItem('trumsg', '打包当前 MSG / END 源码(高级…)', 'script.compileMsgAdvanced', !activeIsMsgSource.value),
      { type: 'divider', key: 'script-std-div' },
      // STD
      toolItem('trustd', '反编译当前 .std', 'script.decompileStd', !activeIsStd.value),
      toolItem('trustd', '编译当前 .stdspec', 'script.compileStd', !activeIsStdSource.value),
      { label: '生成 truth Agent 指南', key: 'script.truthAssistPack' },
      { type: 'divider', key: 'script-dat-div' },
      // DAT
      toolItem('thdat', '解包当前 .dat', 'script.extractDat', false),
      toolItem('thdat', '打包目录为 .dat', 'script.packDat', false),
      { type: 'divider', key: 'script-ai-div' },
      // AI pack
      { label: '生成 AI 辅助包', key: 'script.generateAiPack', disabled: !hasWorkspace.value }
    ]
  },
  {
    key: 'terminal',
    label: '终端',
    options: [
      { label: '新建终端', key: 'terminal.new' },
      ...terminalMenuActions.map(({ label, action }) => ({
        label,
        key: `terminal.action.${action}`,
        disabled: !terminalStore.activeSession || (action === 'paste' && terminalStore.activeSession.exited)
      }))
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
  if (key.startsWith('terminal.action.')) {
    const action = terminalMenuActions.find(item => `terminal.action.${item.action}` === key)?.action
    const id = terminalStore.activeSessionId
    if (action && id != null) {
      workbenchPanelsStore.showBottomPanel('terminal')
      // 面板隐藏时可能未挂载，等它订阅动作后再发送，保留发起操作时的目标。
      await nextTick()
      dispatchTerminalAction(id, action)
    }
    return
  }
  if (key.startsWith('editor.')) {
    const action = key.slice('editor.'.length)
    if (Object.hasOwn(editorActionIds, action)) dispatchEditorAction(action as EditorAction)
    return
  }
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
    case 'view.toggleBottomPanel':
      workbenchPanelsStore.toggleBottomPanel()
      break
    case 'view.toggleRightSidebar':
      workbenchPanelsStore.toggleRightPanel()
      break
    case 'view.toggleWordWrap':
      workbenchPanelsStore.toggleWordWrap()
      break
    case 'view.toggleWhitespace':
      workbenchPanelsStore.toggleWhitespace()
      break
    case 'view.toggleMinimap':
      workbenchPanelsStore.toggleMinimap()
      break
    case 'view.zoomIn':
      void zoomStore.zoomIn()
      break
    case 'view.zoomOut':
      void zoomStore.zoomOut()
      break
    case 'view.zoomReset':
      void zoomStore.reset()
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
    case 'script.decompileMsgAdvanced':
      openMsgBuildDialog('decompile')
      break
    case 'script.compileMsgAdvanced':
      openMsgBuildDialog('compile')
      break
    case 'script.compileMsg':
      tcActions.runCompileMsg(editorStore.activeTab)
      break
    case 'script.decompileStd':
      tcActions.runDecompileStd(editorStore.activeTab?.path)
      break
    case 'script.truthAssistPack':
      generateTruthAssistPack().then(path => message.success(`已生成 ${path}`)).catch(e => message.error(String(e)))
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
