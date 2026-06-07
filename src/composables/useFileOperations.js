import { ref, reactive } from 'vue'
import { useMessage } from 'naive-ui'
import { createDir, createFile, renameEntry, deleteEntry } from '../api'
import { useProjectStore } from '../stores/project'
import { useEditorStore } from '../stores/editor'

const inputState = reactive({
  type: null,        
  targetPath: null,  
  fileType: null,    
  defaultValue: ''   
})

export function useFileOperations() {
  const message = useMessage()
  const projectStore = useProjectStore()
  const editorStore = useEditorStore()
  const separator = window.navigator.userAgent.includes('Windows') ? '\\' : '/'
  
  // 新增：防抖锁，防止 Enter 和 Blur 同时触发导致请求发两次
  const isSubmitting = ref(false)

  function handleCreate(parentPath, type) {
    inputState.type = 'create'
    inputState.targetPath = parentPath
    inputState.fileType = type
    inputState.defaultValue = ''
  }

  function handleRename(node) {
    inputState.type = 'rename'
    inputState.targetPath = node.path
    inputState.defaultValue = node.name
  }

  function handleDelete(node, dialog) {
    dialog.warning({
      title: '确认删除',
      content: `确定要永久删除 "${node.name}" 吗？`,
      positiveText: '删除',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          await deleteEntry(node.path)
          editorStore.closeTabsUnderPath(node.path)
          await new Promise(r => setTimeout(r, 80))
          await projectStore.refresh()
        } catch (e) {
          message.error(`删除失败: ${e}`)
        }
      }
    })
  }

  async function submitInput(value) {
    if (isSubmitting.value || inputState.type === null) {
      return
    }
    
    if (!value || !value.trim()) {
      cancelInput()
      return
    }

    const val = value.trim()
    
    if (inputState.type === 'rename' && val === inputState.defaultValue) {
      cancelInput()
      return
    }

    isSubmitting.value = true

    try {
      if (inputState.type === 'create') {
        const newPath = `${inputState.targetPath}${separator}${val}`
        if (inputState.fileType === 'file') await createFile(newPath)
        else await createDir(newPath)
      } 
      else if (inputState.type === 'rename') {
        const oldPath = inputState.targetPath
        const parentDir = oldPath.substring(0, oldPath.lastIndexOf(separator))
        const newPath = `${parentDir}${separator}${val}`
        
        await renameEntry(oldPath, newPath)
        editorStore.handlePathRename(oldPath, newPath)
      }

      cancelInput() 
      await projectStore.refresh()
      
    } catch (e) {
      message.error(String(e))
      cancelInput()
    } finally {
      isSubmitting.value = false
    }
  }

  function cancelInput() {
    inputState.type = null
    inputState.targetPath = null
    inputState.fileType = null
    isSubmitting.value = false
  }

  return {
    inputState,
    handleCreate,
    handleRename,
    handleDelete,
    submitInput,
    cancelInput
  }
}
