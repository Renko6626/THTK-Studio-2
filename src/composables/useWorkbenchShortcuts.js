import { onMounted, onBeforeUnmount } from 'vue'
import { dispatchEditorAction } from './useEditorActionBridge'

function isEditableTarget(target) {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.isContentEditable ||
    ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) ||
    Boolean(target.closest('input, textarea, select, [contenteditable="true"]'))
  )
}

export function useWorkbenchShortcuts({
  editorStore,
  projectStore,
  workbenchPanelsStore,
  showReloadNotice,
  // 由 WorkbenchRoot 注入 useProjectActions 的打开动作。快捷键自己再实现一遍的话，
  // Ctrl+O 会绕过脏标签保护和统一的错误提示。
  openFolder
}) {

  function handleGlobalKeydown(event) {
    const key = event.key.toLowerCase()
    const editingFieldFocused = isEditableTarget(event.target)

    if (
      event.key === 'F5' ||
      ((event.ctrlKey || event.metaKey) && key === 'r') ||
      ((event.ctrlKey || event.metaKey) && event.shiftKey && key === 'r')
    ) {
      event.preventDefault()
      event.stopPropagation()
      showReloadNotice('已屏蔽重载快捷键，工作区状态会自动持久化。')
      return
    }

    if (editorStore.activeTab && !editingFieldFocused) {
      if ((event.ctrlKey || event.metaKey) && key === 'f') {
        event.preventDefault()
        dispatchEditorAction('find')
        return
      }

      if ((event.ctrlKey || event.metaKey) && key === 'h') {
        event.preventDefault()
        dispatchEditorAction('replace')
        return
      }

      if (event.key === 'F3') {
        event.preventDefault()
        dispatchEditorAction(event.shiftKey ? 'findPrevious' : 'findNext')
        return
      }
    }

    if ((event.ctrlKey || event.metaKey) && key === 'o') {
      event.preventDefault()
      void openFolder()
      return
    }

    if ((event.ctrlKey || event.metaKey) && event.key === '`') {
      event.preventDefault()
      workbenchPanelsStore.toggleBottomPanel('terminal')
      return
    }

    if ((event.ctrlKey || event.metaKey) && key === 'w') {
      event.preventDefault()
      if (editorStore.activeTab && !editorStore.activeTab.isDirty) {
        editorStore.closeTab(editorStore.activePath)
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleGlobalKeydown)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', handleGlobalKeydown)
  })
}
