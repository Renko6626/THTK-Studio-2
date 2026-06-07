<template>
  <div ref="container" class="w-full h-full overflow-hidden"></div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, shallowRef } from 'vue'
import * as monaco from 'monaco-editor'
import { useEditorStore } from '../../stores/editor'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { useWorkbenchReportsStore } from '../../stores/workbenchReports'
import { useMessage } from 'naive-ui'
import { ensureEclLanguageRegistered, eclThemeName, inferMonacoLanguageId } from '../../services/languages/ecl/register'
import {
  clearEclStaticDiagnostics,
  createEclStaticProblemEntries,
  getEclStaticProblemOwnerKey,
  updateEclStaticDiagnostics
} from '../../services/languages/ecl/static-diagnostics'
import { clearToolchainDiagnostics, syncToolchainDiagnosticsToModels } from '../../services/languages/ecl/toolchain-diagnostics'

const emit = defineEmits(['update-cursor'])
const editorStore = useEditorStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const reportsStore = useWorkbenchReportsStore()
const message = useMessage()
const container = ref(null)

// 使用 shallowRef 避免 Vue 深度代理 Monaco 对象导致性能问题
const editorInstance = shallowRef(null)
const editorActionMap = {
  undo: 'undo',
  redo: 'redo',
  find: 'actions.find',
  replace: 'editor.action.startFindReplaceAction',
  findNext: 'editor.action.nextMatchFindAction',
  findPrevious: 'editor.action.previousMatchFindAction',
  selectAll: 'editor.action.selectAll'
}

// 模型缓存 Map: Map<path, ITextModel>
// 这样切换 Tab 时不会丢失 Undo/Redo 历史
const models = new Map()
const diagnosticTimers = new Map()

function scheduleModelDiagnostics(model) {
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

function syncStaticProblemsForModel(model) {
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

function clearStaticProblemsForModel(model) {
  if (!model) return
  reportsStore.replaceProblems(getEclStaticProblemOwnerKey(model.uri?.fsPath), [])
}

function handleEditorAction(event) {
  if (!editorInstance.value) return
  const action = editorActionMap[event.detail?.action]
  if (!action) return
  editorInstance.value.focus()
  const editorAction = editorInstance.value.getAction?.(action)
  if (editorAction?.run) {
    void editorAction.run()
    return
  }
  editorInstance.value.trigger('menu-bar', action, null)
}

function handleRevealLocation(event) {
  const detail = event.detail || {}
  if (!editorInstance.value || !detail.path) return
  if (editorStore.activePath !== detail.path) return

  const lineNumber = Math.max(1, Number(detail.line || 1))
  const column = Math.max(1, Number(detail.column || 1))

  editorInstance.value.focus()
  editorInstance.value.revealPositionInCenter({ lineNumber, column })
  editorInstance.value.setPosition({ lineNumber, column })
}

onMounted(() => {
  if (!container.value) return

  ensureEclLanguageRegistered()

  // 1. 创建编辑器实例 (注意 model 设为 null，稍后设置)
  editorInstance.value = monaco.editor.create(container.value, {
    model: null, 
    theme: eclThemeName,
    automaticLayout: true, // 关键：自动适应父容器 flex 变化
    fontSize: 13,
    fontFamily: "'Cascadia Code', 'Fira Code', Consolas, 'Courier New', monospace",
    fontLigatures: true,
    wordWrap: 'off',
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
    renderWhitespace: 'selection',
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
    if (editorStore.activePath) {
      const model = editorInstance.value.getModel()
      if (model) {
        editorStore.updateContent(editorStore.activePath, model.getValue())
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
    handleEditorAction({ detail: { action: 'find' } })
  })

  editorInstance.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyH, () => {
    handleEditorAction({ detail: { action: 'replace' } })
  })

  editorInstance.value.addCommand(monaco.KeyCode.F3, () => {
    handleEditorAction({ detail: { action: 'findNext' } })
  })

  editorInstance.value.addCommand(monaco.KeyMod.Shift | monaco.KeyCode.F3, () => {
    handleEditorAction({ detail: { action: 'findPrevious' } })
  })

  // 初始加载
  updateEditorModel()
  window.addEventListener('thtk:editor-action', handleEditorAction)
  window.addEventListener('thtk:editor-reveal-location', handleRevealLocation)
})

// 监听 Tab 切换
watch(
  () => editorStore.activeTab,
  (newTab) => {
    updateEditorModel()
  },
  { deep: false } // 不需要深度监听
)

watch(
  () => reportsStore.problemEntries,
  (entries) => {
    syncToolchainDiagnosticsToModels(models, entries)
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

// 核心逻辑：切换 Model
function updateEditorModel() {
  if (!editorInstance.value) return
  const tab = editorStore.activeTab

  if (!tab) {
    const currentModel = editorInstance.value.getModel()
    if (currentModel) {
      clearEclStaticDiagnostics(currentModel)
      clearStaticProblemsForModel(currentModel)
    }
    editorInstance.value.setModel(null)
    return
  }

  let model = models.get(tab.path)

  // 如果 Model 不存在，创建一个新的
  if (!model) {
    const langId = inferMonacoLanguageId(tab)

    model = monaco.editor.createModel(
      tab.content,
      langId,
      monaco.Uri.file(tab.path) // 给 Monaco 一个虚拟路径
    )
    
    models.set(tab.path, model)
    syncToolchainDiagnosticsToModels(models, reportsStore.problemEntries)
  } else {
    const nextLanguageId = inferMonacoLanguageId(tab)
    if (model.getLanguageId() !== nextLanguageId) {
      monaco.editor.setModelLanguage(model, nextLanguageId)
    }
    // 如果 Store 里的内容被外部更新了（比如反编译覆盖），需要同步回 Model
    // 注意：setValue 会清空 Undo 栈，仅在必要时调用
    if (model.getValue() !== tab.content) {
      model.setValue(tab.content)
    }
  }

  // 切换编辑器显示的 Model
  editorInstance.value.setModel(model)
  scheduleModelDiagnostics(model)
  syncStaticProblemsForModel(model)
  
  // 恢复焦点
  editorInstance.value.focus()
  
  // 更新一次光标状态
  const pos = editorInstance.value.getPosition()
  if (pos) {
    emit('update-cursor', { line: pos.lineNumber, col: pos.column })
  }
}

onBeforeUnmount(() => {
  window.removeEventListener('thtk:editor-action', handleEditorAction)
  window.removeEventListener('thtk:editor-reveal-location', handleRevealLocation)
  diagnosticTimers.forEach((timerId) => window.clearTimeout(timerId))
  diagnosticTimers.clear()
  // 销毁所有 Model 和编辑器
  models.forEach(model => {
    clearEclStaticDiagnostics(model)
    clearToolchainDiagnostics(model)
    clearStaticProblemsForModel(model)
    model.dispose()
  })
  models.clear()
  if (editorInstance.value) {
    editorInstance.value.dispose()
  }
})
</script>
