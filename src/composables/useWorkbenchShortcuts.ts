import { onMounted, onBeforeUnmount } from 'vue'
import { dispatchEditorAction } from './useEditorActionBridge'
import type { useEditorStore } from '../stores/editor'
import type { useWorkbenchPanelsStore } from '../stores/workbenchPanels'

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.isContentEditable ||
    ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName) ||
    Boolean(target.closest('input, textarea, select, [contenteditable="true"]'))
  )
}

export interface WorkbenchShortcutsDeps {
  editorStore: ReturnType<typeof useEditorStore>
  workbenchPanelsStore: ReturnType<typeof useWorkbenchPanelsStore>
  showReloadNotice: (text: string) => void
  /**
   * 由 WorkbenchRoot 注入 useProjectActions 的打开动作。快捷键自己再实现一遍的话，
   * Ctrl+O 会绕过脏标签保护和统一的错误提示。
   */
  openFolder: () => Promise<boolean>
}

export function useWorkbenchShortcuts({
  editorStore,
  workbenchPanelsStore,
  showReloadNotice,
  openFolder
}: WorkbenchShortcutsDeps) {

  function handleGlobalKeydown(event: KeyboardEvent) {
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
      // 用 activeTab.path 而不是 activePath：两者在此必然相等
      // （activeTab 就是按 activePath 查出来的），但只有前者 TS 能确定非空
      const activeTab = editorStore.activeTab
      if (activeTab && !activeTab.isDirty) {
        editorStore.closeTab(activeTab.path)
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
