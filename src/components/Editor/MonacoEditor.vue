<template>
  <div ref="container" class="w-full h-full overflow-hidden"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, onActivated, watch, shallowRef, nextTick } from 'vue'
import * as monaco from 'monaco-editor'
import { useEditorStore } from '../../stores/editor'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { useMessage } from 'naive-ui'
import { ensureEclLanguageRegistered } from '../../services/languages/ecl/register'
import { EditorModelSessions } from '../../services/workbench/editorModels'
import { editorActionIds, type EditorAction } from '../../composables/useEditorActionBridge'
import { ensureMsgLanguageRegistered } from '../../services/languages/msg/register'
import { workbenchThemeName } from '../../services/languages/theme'
import { ensureStdLanguageRegistered } from '../../services/languages/std/register'
import {
  clearEclStaticDiagnostics,
  createEclStaticProblemEntries,
  getEclStaticProblemOwnerKey,
  updateEclStaticDiagnostics
} from '../../services/languages/ecl/static-diagnostics'
import { clearToolchainDiagnostics, syncToolchainDiagnosticsToModels } from '../../services/languages/ecl/toolchain-diagnostics'

const emit = defineEmits<{
  'update-cursor': [stats: { line: number; col: number }]
}>()
const editorStore = useEditorStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const reportsStore = useWorkbenchReportsStore()
const message = useMessage()
const container = ref<HTMLElement | null>(null)

// 使用 shallowRef 避免 Vue 深度代理 Monaco 对象导致性能问题
const editorInstance = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)
const diagnosticTimers = new Map<string, number>()
const modelSessions = new EditorModelSessions((model) => {
  const key = model.uri.toString()
  const timer = diagnosticTimers.get(key)
  if (timer !== undefined) window.clearTimeout(timer)
  diagnosticTimers.delete(key)
  clearEclStaticDiagnostics(model)
  clearToolchainDiagnostics(model)
  clearStaticProblemsForModel(model)
})
let displayedPath: string | null = null

function allModels() {
  return [...modelSessions.sessions.values()].map(session => session.model)
}

function scheduleModelDiagnostics(model: monaco.editor.ITextModel | null | undefined) {
  if (!model) return
  const uriKey = model.uri.toString()
  if (diagnosticTimers.has(uriKey)) {
    window.clearTimeout(diagnosticTimers.get(uriKey))
  }

  const timer = window.setTimeout(() => {
    diagnosticTimers.delete(uriKey)
    updateEclStaticDiagnostics(model)
    syncStaticProblemsForModel(model)
  }, 120)

  diagnosticTimers.set(uriKey, timer)
}

function syncStaticProblemsForModel(model: monaco.editor.ITextModel | null | undefined) {
  if (!model) return
  const ownerKey = getEclStaticProblemOwnerKey(model.uri?.fsPath)
  if (model.getLanguageId() !== 'ecl') {
    reportsStore.replaceProblems(ownerKey, [])
    return
  }

  reportsStore.replaceProblems(
    ownerKey,
    createEclStaticProblemEntries(model.uri?.fsPath, model.getValue())
  )
}

function clearStaticProblemsForModel(model: monaco.editor.ITextModel | null | undefined) {
  if (!model) return
  reportsStore.replaceProblems(getEclStaticProblemOwnerKey(model.uri?.fsPath), [])
}

async function runEditorAction(actionKey: EditorAction | string | undefined) {
  const editor = editorInstance.value
  if (!editor || editorStore.activeTab?.viewType !== 'text') return
  const action = actionKey && Object.hasOwn(editorActionIds, actionKey)
    ? editorActionIds[actionKey as EditorAction]
    : undefined
  if (!action) return
  editor.focus()
  const editorAction = editor.getAction?.(action)
  if (editorAction?.run) {
    try {
      await editorAction.run()
    } catch (error) {
      message.error(`编辑操作失败：${String(error)}`)
    }
    return
  }
  editor.trigger('menu-bar', action, null)
}

function handleEditorAction(event: Event) {
  const detail = (event as CustomEvent<{ action?: string }>).detail
  runEditorAction(detail?.action)
}

function handleRevealLocation(event: Event) {
  const editor = editorInstance.value
  const detail = (event as CustomEvent<{ path?: string; line?: number; column?: number }>).detail
  if (!editor || !detail?.path) return
  if (editorStore.activePath !== detail.path) return

  const lineNumber = Math.max(1, Number(detail.line || 1))
  const column = Math.max(1, Number(detail.column || 1))

  editor.focus()
  editor.revealPositionInCenter({ lineNumber, column })
  editor.setPosition({ lineNumber, column })
}

onMounted(() => {
  if (!container.value) return

  ensureEclLanguageRegistered()
  ensureStdLanguageRegistered()
  ensureMsgLanguageRegistered()

  // 1. 创建编辑器实例 (注意 model 设为 null，稍后设置)
  editorInstance.value = monaco.editor.create(container.value, {
    model: null, 
    theme: workbenchThemeName,
    automaticLayout: true, // 关键：自动适应父容器 flex 变化
    fontSize: 13,
    fontFamily: "'Cascadia Code', 'Fira Code', Consolas, 'Courier New', monospace",
    fontLigatures: true,
    wordWrap: workbenchPanelsStore.wordWrap ? 'on' : 'off',
    wordBasedSuggestions: 'off',
    suggestOnTriggerCharacters: true,
    minimap: {
      enabled: workbenchPanelsStore.minimapVisible,
      side: 'right',
      size: 'fit',
      showSlider: 'mouseover',
      renderCharacters: false,
      maxColumn: 100,
      scale: 2
    },
    scrollBeyondLastLine: false,
    fixedOverflowWidgets: true,
    renderWhitespace: workbenchPanelsStore.renderWhitespace ? 'all' : 'selection',
    renderLineHighlight: 'all',
    guides: {
      indentation: true,
      bracketPairs: true
    },
    bracketPairColorization: {
      enabled: true
    },
    smoothScrolling: true,
    padding: { top: 8, bottom: 12 },
    lineNumbersMinChars: 4,
    tabSize: 2,
    scrollbar: {
      verticalScrollbarSize: 10,
      horizontalScrollbarSize: 10
    },
    suggest: {
      showWords: false,
      showSnippets: true
    }
  })

  // 2. 绑定事件：光标移动
  editorInstance.value.onDidChangeCursorPosition((e) => {
    window.dispatchEvent(new CustomEvent('thtk:editor-cursor-position', {
      detail: {
        path: editorStore.activePath,
        line: e.position.lineNumber,
        column: e.position.column
      }
    }))
    emit('update-cursor', {
      line: e.position.lineNumber,
      col: e.position.column
    })
  })

  // 3. 绑定事件：内容修改
  editorInstance.value.onDidChangeModelContent(() => {
    if (displayedPath && editorStore.activePath === displayedPath) {
      const model = editorInstance.value?.getModel()
      if (model) {
        editorStore.updateContent(displayedPath, model.getValue())
        scheduleModelDiagnostics(model)
      }
    }
  })

  // 4. 绑定 Ctrl+S
  editorInstance.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, async () => {
    if (editorStore.activeTab) {
      const success = await editorStore.saveActiveFile()
      if (success) {
        message.success(`已保存: ${editorStore.activeTab.name}`)
      } else {
        message.error('保存失败')
      }
    }
  })

  editorInstance.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyF, () => {
    runEditorAction('find')
  })

  editorInstance.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyH, () => {
    runEditorAction('replace')
  })

  editorInstance.value.addCommand(monaco.KeyCode.F3, () => {
    runEditorAction('findNext')
  })

  editorInstance.value.addCommand(monaco.KeyMod.Shift | monaco.KeyCode.F3, () => {
    runEditorAction('findPrevious')
  })

  editorInstance.value.addAction({
    id: 'thtk.toggleWordWrap',
    label: '切换自动换行',
    keybindings: [monaco.KeyMod.Alt | monaco.KeyCode.KeyZ],
    run: () => workbenchPanelsStore.toggleWordWrap()
  })

  // 初始加载
  updateEditorModel()
  window.addEventListener('thtk:editor-action', handleEditorAction)
  window.addEventListener('thtk:editor-reveal-location', handleRevealLocation)
})

// 监听 Tab 切换
watch(
  () => [editorStore.activeTab?.path, editorStore.activeTab?.viewType, editorStore.activeTab?.language],
  () => {
    updateEditorModel()
  },
  { deep: false } // 不需要深度监听
)

watch(
  () => reportsStore.problemEntries,
  (entries) => {
    syncToolchainDiagnosticsToModels(allModels(), entries)
  },
  { deep: true }
)

watch(
  () => workbenchPanelsStore.minimapVisible,
  (visible) => {
    if (!editorInstance.value) return
    editorInstance.value.updateOptions({
      minimap: {
        enabled: visible,
        side: 'right',
        size: 'fit',
        showSlider: 'mouseover',
        renderCharacters: false,
        maxColumn: 100,
        scale: 2
      }
    })
  }
)

// KeepAlive 切回文本视图时，恢复可见尺寸，避免使用隐藏期间的布局。
onActivated(() => {
  void nextTick(() => editorInstance.value?.layout())
})

watch(
  () => [workbenchPanelsStore.wordWrap, workbenchPanelsStore.renderWhitespace],
  () => editorInstance.value?.updateOptions({
    wordWrap: workbenchPanelsStore.wordWrap ? 'on' : 'off',
    renderWhitespace: workbenchPanelsStore.renderWhitespace ? 'all' : 'selection'
  })
)

// 内容由外部重载时也要同步；本编辑器的输入已经写入模型，无需重新绑定。
watch(() => editorStore.activeTab?.content, () => {
  const tab = editorStore.activeTab
  if (tab?.viewType === 'text' && editorInstance.value?.getModel()?.getValue() !== tab.content) {
    updateEditorModel()
  }
})

watch(() => editorStore.tabs.map(tab => tab.path), () => {
  updateEditorModel()
  modelSessions.retain(editorStore.tabs.map(tab => tab.path))
})

function updateEditorModel() {
  const editor = editorInstance.value
  if (!editor) return
  const previous = displayedPath ? modelSessions.sessions.get(displayedPath) : undefined
  if (previous) previous.viewState = editor.saveViewState()

  const tab = editorStore.activeTab
  if (!tab || tab.viewType !== 'text') {
    editor.setModel(null)
    displayedPath = null
    return
  }

  const session = modelSessions.open(tab)
  const switching = displayedPath !== tab.path
  editor.setModel(session.model)
  displayedPath = tab.path
  if (switching && session.viewState) editor.restoreViewState(session.viewState)
  scheduleModelDiagnostics(session.model)
  syncStaticProblemsForModel(session.model)
  syncToolchainDiagnosticsToModels(allModels(), reportsStore.problemEntries)
  if (switching) editor.focus()
  const pos = editor.getPosition()
  if (pos) emit('update-cursor', { line: pos.lineNumber, col: pos.column })
}

onBeforeUnmount(() => {
  window.removeEventListener('thtk:editor-action', handleEditorAction)
  window.removeEventListener('thtk:editor-reveal-location', handleRevealLocation)
  diagnosticTimers.forEach((timerId) => window.clearTimeout(timerId))
  diagnosticTimers.clear()
  editorInstance.value?.setModel(null)
  modelSessions.dispose()
  editorInstance.value?.dispose()
})
</script>
