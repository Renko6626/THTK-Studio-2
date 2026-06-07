// src/composables/useContextMenu.js
import { ref, computed, nextTick, h } from 'vue'
import { NIcon } from 'naive-ui'
import {
  Edit24Regular,
  Delete24Regular,
  Add24Regular,
  FolderAdd24Regular,
  Copy24Regular,
  Cut24Regular,
  ClipboardPaste24Regular
} from '@vicons/fluent'
import { useProjectStore } from '../stores/project'
import { useExplorerClipboardStore } from '../stores/explorerClipboard'

function getParentPath(path) {
  const normalized = path.replace(/[\\/]+$/, '')
  const lastSlash = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))
  return lastSlash > 0 ? normalized.slice(0, lastSlash) : normalized
}

export function useContextMenu() {
  const projectStore = useProjectStore()
  const explorerClipboardStore = useExplorerClipboardStore()

  const showMenu = ref(false)
  const menuX = ref(0)
  const menuY = ref(0)
  const targetNode = ref(null)

  // 触发右键菜单
  const handleContextMenu = (e, nodeOption) => {
    e.preventDefault()
    e.stopPropagation?.()

    showMenu.value = false
    nextTick(() => {
      targetNode.value = nodeOption
      menuX.value = e.clientX
      menuY.value = e.clientY
      showMenu.value = true
    })
  }

  // 点击外部关闭
  const handleClickOutside = () => {
    showMenu.value = false
  }

  // 生成菜单选项
  const menuOptions = computed(() => {
    const node = targetNode.value
    if (!node) return []

    // ✅ 根目录判定（双保险：path 或 isRoot）
    const isRoot =
      !!node.isRoot ||
      (!!projectStore.rootPath && node.path === projectStore.rootPath)

    const pasteTargetPath = node.is_dir ? node.path : getParentPath(node.path)
    const placingIntoOwnDescendant = explorerClipboardStore.entries.some((entry) => {
      if (!entry?.is_dir || !entry.path) return false
      return (
        pasteTargetPath === entry.path ||
        pasteTargetPath.startsWith(`${entry.path}\\`) ||
        pasteTargetPath.startsWith(`${entry.path}/`)
      )
    })

    const canPaste =
      explorerClipboardStore.hasEntry &&
      !!projectStore.rootPath &&
      !!pasteTargetPath &&
      !placingIntoOwnDescendant

    // 基础操作：重命名/删除（根目录不允许）
    const baseOptions = isRoot
      ? []
      : [
          {
            label: '剪切',
            key: 'cut',
            icon: () => h(NIcon, null, { default: () => h(Cut24Regular) })
          },
          {
            label: '复制',
            key: 'copy',
            icon: () => h(NIcon, null, { default: () => h(Copy24Regular) })
          },
          {
            label: '重命名',
            key: 'rename',
            icon: () => h(NIcon, null, { default: () => h(Edit24Regular) })
          },
          {
            label: '复制文件路径',
            key: 'copy_path'
          },
          {
            label: '删除',
            key: 'delete',
            icon: () =>
              h(
                NIcon,
                { color: '#d03050' },
                { default: () => h(Delete24Regular) }
              )
          }
        ]

    // 只有文件夹可以新建（根目录也算文件夹，所以这里照常允许）
    if (node.is_dir) {
      const createOptions = [
        {
          label: '新建文件',
          key: 'new_file',
          icon: () => h(NIcon, null, { default: () => h(Add24Regular) })
        },
        {
          label: '新建文件夹',
          key: 'new_dir',
          icon: () => h(NIcon, null, { default: () => h(FolderAdd24Regular) })
        },
        {
          label: explorerClipboardStore.isCut ? '粘贴并移动' : '粘贴',
          key: 'paste',
          disabled: !canPaste,
          icon: () => h(NIcon, null, { default: () => h(ClipboardPaste24Regular) })
        }
      ]

      // 如果没有 baseOptions（比如根目录），就不要插 divider
      return baseOptions.length
        ? [...createOptions, { type: 'divider' }, ...baseOptions]
        : createOptions
    }

    return [
      {
        label: explorerClipboardStore.isCut ? '粘贴并移动到当前目录' : '粘贴到当前目录',
        key: 'paste',
        disabled: !canPaste,
        icon: () => h(NIcon, null, { default: () => h(ClipboardPaste24Regular) })
      },
      { type: 'divider' },
      ...baseOptions
    ]
  })

  return {
    showMenu,
    menuX,
    menuY,
    targetNode,
    menuOptions,
    handleContextMenu,
    handleClickOutside
  }
}
