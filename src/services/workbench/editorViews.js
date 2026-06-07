import MonacoEditor from '../../components/Editor/MonacoEditor.vue'
import BinaryScriptView from '../../components/Editor/BinaryScriptView.vue'

export const WORKBENCH_EDITOR_VIEWS = {
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

export function resolveEditorView(viewType) {
  return WORKBENCH_EDITOR_VIEWS[viewType] || null
}
