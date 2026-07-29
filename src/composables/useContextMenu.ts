// src/composables/useContextMenu.ts
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
import type { Ref } from 'vue'
import type { DropdownOption } from 'naive-ui'
import { useProjectStore } from '../stores/project'
import { useExplorerClipboardStore } from '../stores/explorerClipboard'
import type { FileNode } from '../types'

/**
 * 右键菜单作用的节点。
 *
 * `isRoot` 是可选的：现有代码里写了 `!!node.isRoot ||` 作为"双保险"，但全仓
 * **没有任何地方设置过这个字段**，所以那一半恒为 false，实际生效的只有
 * path === rootPath 的比较。保留它是为了不改行为；真要收拾应该单独提交。
 */
export type ContextMenuNode = Partial<FileNode> &
  Pick<FileNode, 'path' | 'is_dir'> & { isRoot?: boolean }

export interface ContextMenuOptions {
  selectedKeys?: Ref<string[]>
}

function getParentPath(path: string): string {
  const normalized = path.replace(/[\\/]+$/, '')
  const lastSlash = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))
  return lastSlash > 0 ? normalized.slice(0, lastSlash) : normalized
}

export function useContextMenu({ selectedKeys }: ContextMenuOptions = {}) {
  const projectStore = useProjectStore()
  const explorerClipboardStore = useExplorerClipboardStore()

  const showMenu = ref(false)
  const menuX = ref(0)
  const menuY = ref(0)
  const targetNode = ref<ContextMenuNode | null>(null)

  // 触发右键菜单
  const handleContextMenu = (e: MouseEvent, nodeOption: ContextMenuNode) => {
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
  const menuOptions = computed<DropdownOption[]>(() => {
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
            disabled: (selectedKeys?.value?.length ?? 1) > 1,
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
