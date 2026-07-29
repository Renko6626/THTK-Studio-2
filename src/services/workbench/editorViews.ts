import type { Component } from 'vue'
import MonacoEditor from '../../components/Editor/MonacoEditor.vue'
import BinaryScriptView from '../../components/Editor/BinaryScriptView.vue'
import type { EditorTab, EditorViewType } from '../../stores/editor'

export interface WorkbenchEditorView {
  id: EditorViewType
  component: Component
  /** 状态栏右侧显示的模式标签 */
  statusLabel: (tab: EditorTab | null | undefined) => string
}

export const WORKBENCH_EDITOR_VIEWS: Record<EditorViewType, WorkbenchEditorView> = {
  text: {
    id: 'text',
    component: MonacoEditor,
    statusLabel: (tab) => (tab?.language || 'txt').toUpperCase()
  },
  'binary-script': {
    id: 'binary-script',
    component: BinaryScriptView,
    statusLabel: (tab) => `${(tab?.extension || 'bin').toUpperCase()} BIN`
  }
}

export function resolveEditorView(
  viewType: string | null | undefined
): WorkbenchEditorView | null {
  if (!viewType) return null
  return WORKBENCH_EDITOR_VIEWS[viewType as EditorViewType] || null
}
