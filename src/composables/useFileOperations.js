import { ref, reactive } from 'vue'
import { useMessage } from 'naive-ui'
import { createDir, createFile, renameEntry, deleteEntry } from '../api'
import { useProjectStore } from '../stores/project'
import { useEditorStore } from '../stores/editor'
import { normalizePath } from '../utils/pathNormalize'

const inputState = reactive({
  type: null,
  targetPath: null,
  fileType: null,
  defaultValue: ''
})

// 禁止的字符：路径分隔符、Windows 保留字符、控制字符
const INVALID_NAME_RE = /[/\\:*?"<>|\x00-\x1f]/
const RESERVED_NAMES = new Set(['', '.', '..'])

/**
 * 校验文件/目录名。成功返回 null；失败返回错误消息字符串。
 */
export function validateFileName(name) {
  if (typeof name !== 'string') return '文件名无效'
  const trimmed = name.trim()
  if (trimmed !== name) return '文件名前后不能有空白'
  if (RESERVED_NAMES.has(trimmed)) return '文件名不能为空或为 . / ..'
  if (INVALID_NAME_RE.test(trimmed)) return '文件名不能包含 / \\ : * ? " < > | 或控制字符'
  return null
}

/**
 * 将一个展开路径列表中所有以 oldPath 为前缀（或完全等于 oldPath）的键，
 * 替换为以 newPath 为前缀。用于重命名/移动后保留子树展开状态。
 *
 * 同时识别 '/' 和 '\\' 两种分隔符，以兼容跨平台路径混用场景。
 */
export function remapExpandedKeys(keys, oldPath, newPath) {
  if (!Array.isArray(keys) || !oldPath || !newPath || oldPath === newPath) {
    return keys
  }
  return keys.map(key => {
    if (key === oldPath) return newPath
    for (const sep of ['/', '\\']) {
      const prefix = oldPath + sep
      if (key.startsWith(prefix)) {
        return newPath + sep + key.slice(prefix.length)
      }
    }
    return key
  })
}

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
    if (node.lossy) {
      message.error('该文件名含非 UTF-8 字符，IDE 暂不支持对其操作。请用系统文件管理器处理。')
      return
    }
    inputState.type = 'rename'
    inputState.targetPath = node.path
    inputState.defaultValue = node.name
  }

  function handleDelete(node, dialog) {
    if (node.lossy) {
      message.error('该文件名含非 UTF-8 字符，IDE 暂不支持对其操作。请用系统文件管理器处理。')
      return
    }
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

  /**
   * 提交内联输入。
   *
   * @param {string} value 用户输入的名称
   * @param {object} [extras]
   * @param {(oldPath: string, newPath: string) => void} [extras.onRenamed]
   *        重命名成功后回调，参数为旧/新绝对路径。供调用方更新 selectedKeys、
   *        expandedKeys 等本地状态。
   */
  async function submitInput(value, extras = {}) {
    if (isSubmitting.value || inputState.type === null) {
      return
    }

    // 空值视为取消（保持原行为）
    if (!value || !value.trim()) {
      cancelInput()
      return
    }

    // 校验：非法字符 / 保留名 / 前后空白
    // 注意：此处用原 value（未 trim）来检测前后空白
    const validationError = validateFileName(value)
    if (validationError) {
      // 输入仍然保留焦点，让用户可继续修改；不调用后端、不取消输入态
      message.error(validationError)
      return
    }

    const val = value.trim()

    // 同名重命名 = 无操作；直接取消，不打后端
    if (inputState.type === 'rename' && val === inputState.defaultValue) {
      cancelInput()
      return
    }

    isSubmitting.value = true

    try {
      if (inputState.type === 'create') {
        const newPath = normalizePath(`${inputState.targetPath}${separator}${val}`)
        if (inputState.fileType === 'file') await createFile(newPath)
        else await createDir(newPath)
      }
      else if (inputState.type === 'rename') {
        const oldPath = inputState.targetPath
        const parentDir = oldPath.substring(0, oldPath.lastIndexOf(separator))
        const newPath = normalizePath(`${parentDir}${separator}${val}`)

        await renameEntry(oldPath, newPath)
        editorStore.handlePathRename(oldPath, newPath)
        // 把新路径回传给调用方，让它修正 selectedKeys / expandedKeys
        try {
          extras?.onRenamed?.(oldPath, newPath)
        } catch { /* 调用方异常不影响主流程 */ }
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
