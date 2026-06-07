import { onMounted, onBeforeUnmount } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
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
  showReloadNotice
}) {
  async function openFolder() {
    const selected = await open({ directory: true })
    if (selected) {
      await projectStore.loadProject(selected)
    }
  }

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
