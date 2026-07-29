<template>
  <div class="h-full bg-[#1f1f1f] border-r border-black select-none" @contextmenu="onEmptyAreaRightClick">
    <!-- 顶部栏 -->
    <div class="px-3 py-2 text-[11px] font-bold text-gray-400 uppercase tracking-[0.08em] flex justify-between items-center group border-b border-white/6">
      <div class="min-w-0">
        <div>文件管理器</div>
        <div class="text-[10px] font-normal normal-case tracking-normal text-gray-500 truncate max-w-[180px]">
          Explorer
        </div>
      </div>
      <!-- 常驻可见(原 opacity-0 group-hover 模式导致按钮不可发现) -->
      <div class="opacity-60 hover:opacity-100 transition-opacity flex gap-2">
        <n-icon class="cursor-pointer hover:text-white" title="打开文件夹" @click="openFolder">
          <Folder24Regular />
        </n-icon>
        <!-- 顶部快捷新建按钮 -->
        <n-icon class="cursor-pointer hover:text-white" title="新建文件" @click="quickCreate('file')">
          <Add24Regular />
        </n-icon>
        <n-icon class="cursor-pointer hover:text-white" title="刷新" @click="projectStore.refresh()">
          <ArrowClockwise24Regular />
        </n-icon>
      </div>
    </div>

    <div
      v-if="projectStore.rootPath"
      class="px-3 py-2 border-b border-white/6 transition-colors"
      :class="rootDropActive ? 'bg-[#264f78]/80' : 'bg-white/[0.03]'"
      @dragenter.prevent="handleRootDragEnter"
      @dragover.prevent="handleRootDragOver"
      @dragleave="handleRootDragLeave"
      @drop.prevent="handleRootDrop"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="min-w-0">
          <div class="text-[10px] font-semibold uppercase tracking-[0.08em]" :class="rootDropActive ? 'text-blue-100' : 'text-gray-400'">
            工作区根目录
          </div>
          <div class="text-[11px] truncate" :class="rootDropActive ? 'text-blue-50' : 'text-gray-500'">
            {{ projectStore.rootPath }}
          </div>
        </div>
        <div
          class="shrink-0 rounded border px-2 py-1 text-[10px] uppercase tracking-[0.08em]"
          :class="rootDropActive ? 'border-blue-300/70 text-blue-50 bg-blue-400/10' : 'border-white/10 text-gray-500'"
        >
          {{ rootDropActive ? '释放到根目录' : '可拖放' }}
        </div>
      </div>
    </div>

    <n-spin v-if="projectStore.rootPath" :show="projectStore.isLoading">
      <n-tree
        block-line
        selectable
        multiple
        draggable
        expand-on-click
        :data="displayTreeData"
        key-field="path"
        label-field="name"
        :node-props="nodeProps"
        :render-label="renderLabel"
        :expanded-keys="expandedKeys"
        :selected-keys="selectedKeys"
        :allow-drop="allowDrop"
        :on-load="handleLazyLoad"
        @update:expanded-keys="handleExpand"
        @update:selected-keys="handleSelectKeys"
        @dragstart="handleTreeDragStart"
        @dragover="handleTreeDragOver"
        @dragend="handleTreeDragEnd"
        @drop="handleTreeDrop"
        class="bg-transparent px-1 py-1"
        />

    </n-spin>

    <!-- 右键菜单 -->
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="menuX"
      :y="menuY"
      :options="menuOptions"
      :show="showMenu"
      @clickoutside="handleClickOutside"
      @select="handleMenuSelect"
    />

    <!-- 空状态 -->
    <div v-if="!projectStore.rootPath" class="p-4 text-center mt-10">
      <n-button secondary size="small" @click="openFolder">打开文件夹</n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, h, nextTick, watch, onBeforeUnmount } from 'vue'
import { NTree, NSpin, NButton, NIcon, NDropdown, NInput, useDialog, useMessage } from 'naive-ui'
import type { TreeOption } from 'naive-ui'
import { useProjectStore } from '../../stores/project'
import { useEditorStore } from '../../stores/editor'
import { useExplorerClipboardStore } from '../../stores/explorerClipboard'
import { useExplorerViewStore } from '../../stores/explorerView'
import {
  ArrowClockwise24Regular,
  Add24Regular,
  Folder24Regular
} from '@vicons/fluent'
import { useContextMenu } from '../../composables/useContextMenu'
import type { ContextMenuNode } from '../../composables/useContextMenu'
import { useFileOperations, remapExpandedKeys } from '../../composables/useFileOperations'
import { useFileTreeActions } from '../../composables/useFileTreeActions'
import { useFileTreeDnD } from '../../composables/useFileTreeDnD'
import { useProjectActions } from '../../composables/useProjectActions'
import { renderFileIcon } from '../../utils/renderFileIcon'
import { renameEntry } from '../../api'
import type { FileCategory, FileNode } from '../../types'

/**
 * 喂给 n-tree 的节点。除真实 FileNode 外，还会插入一个**临时节点**用于内联
 * 新建 / 重命名的输入框——它只有 path / name / is_dir / isLeaf，没有磁盘元信息。
 */
/**
 * 把后端的 FileNode 树转成 n-tree 能吃的形状。
 * 唯一的实质差别是 children：后端未展开目录发的是 null（Rust Option<Vec>），
 * 而 naive-ui 的 TreeOption.children 只接受数组或 undefined。
 */
function toTreeNodes(nodes: FileNode[]): TreeNode[] {
  return nodes.map((node) => ({
    ...node,
    children: node.children ? toTreeNodes(node.children) : undefined
  }))
}

interface TreeNode {
  path: string
  name: string
  is_dir: boolean
  size?: number | null
  extension?: string | null
  category?: FileCategory
  lossy?: boolean
  isLeaf?: boolean
  children?: TreeNode[]
  /** 内联输入框用的临时节点，不对应磁盘上的文件 */
  isTemp?: boolean
  /** naive-ui 的 TreeOption 需要索引签名 */
  [key: string]: unknown
}

const projectStore = useProjectStore()
const editorStore = useEditorStore()
const explorerClipboardStore = useExplorerClipboardStore()
const explorerViewStore = useExplorerViewStore()
const dialog = useDialog()
const message = useMessage()
const projectActions = useProjectActions({ message, dialog })

// ---- Composables ----

const { inputState, handleCreate, handleRename, submitInput, cancelInput } = useFileOperations()

const selectedKeys = ref<string[]>([])

const { showMenu, menuX, menuY, targetNode, menuOptions, handleContextMenu, handleClickOutside } = useContextMenu({ selectedKeys })

const fileTreeActions = useFileTreeActions({
  selectedKeys,
  projectStore,
  editorStore,
  explorerClipboardStore,
  explorerViewStore,
  dialog,
  message
})

// ---- 展开状态管理 + 持久化 ----

const EXPANDED_STORAGE_KEY = 'thtk-studio:explorer-expanded'
const expandedKeys = ref<string[]>([])
let expandSaveTimer: number | null = null

// 记录最近一次完成恢复时对应的 rootPath，避免重复对同一项目恢复
const lastRestoredFor = ref<string | null>(null)
let restoringPromise: Promise<void> | null = null

function persistExpandedKeys() {
  if (expandSaveTimer) window.clearTimeout(expandSaveTimer)
  expandSaveTimer = window.setTimeout(() => {
    try {
      window.localStorage.setItem(EXPANDED_STORAGE_KEY, JSON.stringify({
        rootPath: projectStore.rootPath,
        keys: expandedKeys.value
      }))
    } catch { /* ignore */ }
  }, 300)
}

async function restoreExpandedKeys() {
  // 仅在该 rootPath 尚未恢复、且文件树已加载时执行
  const currentRoot = projectStore.rootPath
  if (!currentRoot) return
  if (lastRestoredFor.value === currentRoot) return
  if (!projectStore.files || !projectStore.files.length) return
  if (restoringPromise) return restoringPromise

  restoringPromise = (async () => {
    try {
      const raw = window.localStorage.getItem(EXPANDED_STORAGE_KEY)
      if (!raw) {
        lastRestoredFor.value = currentRoot
        return
      }
      const saved = JSON.parse(raw)
      if (saved?.rootPath !== currentRoot || !Array.isArray(saved.keys) || !saved.keys.length) {
        lastRestoredFor.value = currentRoot
        return
      }

      // 按路径深度排序（父目录先于子目录），逐层预加载
      const sortedKeys = [...saved.keys].sort((a, b) => a.length - b.length)
      const validKeys = []

      for (const dirPath of sortedKeys) {
        // rootPath 中途切换则放弃本次恢复
        if (projectStore.rootPath !== currentRoot) return
        try {
          await projectStore.loadChildren(dirPath)
          validKeys.push(dirPath)
        } catch {
          // 目录可能已不存在，跳过
        }
      }

      if (projectStore.rootPath === currentRoot) {
        expandedKeys.value = validKeys
        lastRestoredFor.value = currentRoot
      }
    } catch { /* ignore */ } finally {
      restoringPromise = null
    }
  })()

  return restoringPromise
}

function handleExpand(keys: string[]) {
  expandedKeys.value = keys
  persistExpandedKeys()
}

function handleSelectKeys(keys: string[]) {
  selectedKeys.value = keys
  explorerViewStore.setSelectedPaths(keys)
}

// rootPath 改变时把恢复标记清掉，让新项目能再次触发恢复
watch(() => projectStore.rootPath, (newRoot) => {
  if (newRoot !== lastRestoredFor.value) {
    lastRestoredFor.value = null
  }
})

// 文件树就绪后，触发持久化展开状态的恢复（避免 watch(rootPath) 与 loadProject
// 内 getFileTree 之间的竞态——rootPath 先被赋值，files 才到位）。
// 同时过滤掉已不存在的展开路径。
watch(
  () => projectStore.files,
  () => {
    // 首次/换项目时恢复（内部带 lastRestoredFor 守卫，不会重复恢复）
    if (
      projectStore.rootPath &&
      projectStore.files.length > 0 &&
      lastRestoredFor.value !== projectStore.rootPath
    ) {
      void restoreExpandedKeys()
      // restore 自己会把现存路径写入 expandedKeys，后面的过滤逻辑不必再跑
      return
    }

    if (!expandedKeys.value.length || !projectStore.files.length) return
    const allPaths = new Set<string>()
    function collectPaths(nodes: TreeNode[]) {
      for (const node of nodes) {
        allPaths.add(node.path)
        if (node.children) collectPaths(node.children)
      }
    }
    collectPaths(toTreeNodes(projectStore.files))
    const filtered = expandedKeys.value.filter(key => allPaths.has(key))
    if (filtered.length !== expandedKeys.value.length) {
      expandedKeys.value = filtered
      persistExpandedKeys()
    }
  },
  { flush: 'post' }
)

// ---- DnD ----

const {
  rootDropActive,
  handleTreeDragStart,
  handleTreeDragOver,
  handleTreeDragEnd,
  allowDrop,
  handleTreeDrop,
  handleRootDragEnter,
  handleRootDragOver,
  handleRootDragLeave,
  handleRootDrop
} = useFileTreeDnD({
  expandedKeys,
  selectedKeys,
  projectStore,
  editorStore,
  explorerViewStore,
  message,
  persistExpandedKeys,
  canMoveEntryIntoDir: fileTreeActions.canMoveEntryIntoDir,
  getExistingNamesForDir: fileTreeActions.getExistingNamesForDir,
  makeUniqueDestinationName: fileTreeActions.makeUniqueDestinationName,
  joinPath: fileTreeActions.joinPath,
  renameEntry
})

// ---- 懒加载 ----

async function handleLazyLoad(node: TreeOption): Promise<unknown> {
  const dir = node as unknown as TreeNode
  if (!dir.is_dir) return
  return projectStore.loadChildren(dir.path)
}

// ---- 临时节点注入（新建文件时） ----

const tempKey = ref('')

watch(() => inputState.type, (t) => {
  if (t === 'create') tempKey.value = `__TEMP_CREATING__::${Date.now()}`
  else tempKey.value = ''
})

watch(() => inputState.type, async (newType) => {
  if (newType === 'create' && inputState.targetPath) {
    // 确保目标目录的 children 已加载（懒加载场景下可能尚未加载）
    const targetDir = fileTreeActions.findNodeByPath(projectStore.files, inputState.targetPath)
    if (targetDir && targetDir.is_dir && !targetDir.children) {
      try { await projectStore.loadChildren(inputState.targetPath) } catch { /* ignore */ }
    }
    if (!expandedKeys.value.includes(inputState.targetPath)) {
      expandedKeys.value = [...expandedKeys.value, inputState.targetPath]
      persistExpandedKeys()
    }
  }
})

function injectTempNode(nodes: TreeNode[]): TreeNode[] {
  return nodes.map(node => {
    const newNode: TreeNode = { ...node }

    if (inputState.type === 'create' && node.path === inputState.targetPath) {
      const tempNode: TreeNode = {
        path: tempKey.value,
        name: '',
        is_dir: inputState.fileType === 'dir',
        extension: inputState.fileType === 'file' ? 'txt' : null,
        isTemp: true,
        isLeaf: true,
      }
      newNode.children = newNode.children ? [tempNode, ...newNode.children] : [tempNode]
      newNode.children = newNode.children.map((child: TreeNode) => {
        if (child.path === tempKey.value) return transformNode(child)
        return transformNode(injectTempNode([child])[0])
      })
    } else if (newNode.children) {
      newNode.children = injectTempNode(newNode.children)
    }

    return transformNode(newNode)
  })
}

const displayTreeData = computed(() => {
  let rawData: TreeNode[] = toTreeNodes(projectStore.files)

  if (inputState.type === 'create' && inputState.targetPath === projectStore.rootPath) {
    const tempNode: TreeNode = {
      path: tempKey.value,
      name: '',
      is_dir: inputState.fileType === 'dir',
      isTemp: true,
      isLeaf: true
    }
    rawData = [tempNode, ...rawData]
  }

  return injectTempNode(rawData)
})

// ---- 节点渲染 ----

function transformNode(node: TreeNode): TreeNode {
  return {
    ...node,
    prefix: (ctx?: { expanded?: boolean }) => renderFileIcon(node, ctx?.expanded ?? false)
  }
}

const currentInputValue = ref('')

watch(
  () => [inputState.type, inputState.targetPath],
  ([type]) => {
    if (type === 'rename') currentInputValue.value = inputState.defaultValue || ''
    else if (type === 'create') currentInputValue.value = ''
    else currentInputValue.value = ''
  },
  { immediate: true }
)

// 重命名成功后，把本地的选中/展开状态从旧路径迁移到新路径，
// 避免 n-tree 因键已变而把节点视为未选中 / 无法保留子树展开。
const submitExtras = {
  onRenamed: (oldPath: string, newPath: string) => {
    if (!oldPath || !newPath || oldPath === newPath) return

    selectedKeys.value = [newPath]
    explorerViewStore.setSelectedPaths([newPath])

    const remapped = remapExpandedKeys(expandedKeys.value, oldPath, newPath)
    // 仅在确实变了时再赋值，避免触发不必要的持久化
    let changed = remapped.length !== expandedKeys.value.length
    if (!changed) {
      for (let i = 0; i < remapped.length; i++) {
        if (remapped[i] !== expandedKeys.value[i]) { changed = true; break }
      }
    }
    if (changed) {
      expandedKeys.value = remapped
      persistExpandedKeys()
    }
  }
}

const renderLabel = ({ option }: { option: TreeOption }) => {
  const node = option as unknown as TreeNode
  const isCreating = node.isTemp
  const isRenaming = inputState.type === 'rename' && node.path === inputState.targetPath

  if (isCreating || isRenaming) {
    return h(NInput, {
      autofocus: true,
      size: 'tiny',
      value: currentInputValue.value,
      placeholder: '名称...',
      'onUpdate:value': (v) => { currentInputValue.value = v },
      onClick: (e) => e.stopPropagation(),
      onBlur: () => { submitInput(currentInputValue.value, submitExtras) },
      onKeydown: (e) => {
        if (e.key === 'Enter') {
          e.preventDefault()
          e.stopPropagation()
          submitInput(currentInputValue.value, submitExtras)
        }
        if (e.key === 'Escape') {
          e.preventDefault()
          e.stopPropagation()
          cancelInput()
        }
      },
      onVnodeMounted: (vnode) => {
        nextTick(() => {
          vnode.component?.exposed?.focus()
          if (isRenaming) vnode.component?.exposed?.select()
        })
      }
    })
  }

  return (option.lossy ? '⚠️ ' : '') + option.name
}

// ---- 节点交互 ----

const nodeProps = ({ option }: { option: TreeOption }) => {
  // TreeOption 带索引签名，属性都是 unknown；树的数据源就是 TreeNode，
  // 在这里收窄一次，后面的处理器直接用具体类型。
  const node = option as unknown as TreeNode
  return {
    onClick(e: MouseEvent) {
      const isMultiSelect = e?.ctrlKey || e?.metaKey || e?.shiftKey
      if (!isMultiSelect) {
        selectedKeys.value = [node.path]
        explorerViewStore.setSelectedPaths([node.path])
      }
      // TreeNode 的索引签名让它与具名类型不兼容，但字段是超集
      if (!node.is_dir && !node.isTemp && !isMultiSelect) {
        void editorStore.openFile(node as unknown as FileNode)
      }
    },
    onContextmenu(e: MouseEvent) {
      if (!selectedKeys.value.includes(node.path)) {
        selectedKeys.value = [node.path]
        explorerViewStore.setSelectedPaths([node.path])
      }
      handleContextMenu(e, node as unknown as ContextMenuNode)
    },
    onDragover(event: DragEvent) {
      if (node.is_dir) {
        event?.preventDefault?.()
        if (event?.dataTransfer) event.dataTransfer.dropEffect = 'move'
      }
    }
  }
}

// ---- 右键菜单处理 ----

function onEmptyAreaRightClick(e: MouseEvent) {
  if (!projectStore.rootPath) return
  selectedKeys.value = []
  explorerViewStore.clearSelection()
  handleContextMenu(e, { path: projectStore.rootPath, is_dir: true, name: 'root' })
}

function handleMenuSelect(key: string) {
  handleClickOutside()
  const node = targetNode.value
  if (!node) return
  const entries = fileTreeActions.getActionEntries(node)

  switch (key) {
    case 'new_file': handleCreate(node.path, 'file'); break
    case 'new_dir': handleCreate(node.path, 'dir'); break
    case 'cut': void fileTreeActions.cutEntries(entries); break
    case 'copy': void fileTreeActions.copyEntries(entries); break
    case 'paste': void fileTreeActions.pasteIntoTarget(node); break
    case 'copy_path': void fileTreeActions.copyPath(entries.map(e => e.path).join('\n')); break
    case 'delete': void fileTreeActions.deleteEntries(entries); break
    case 'rename':
      if (entries.length > 1) message.info('多选状态下暂不支持批量重命名')
      else handleRename(node)
      break
  }
}

function quickCreate(type: 'file' | 'dir') {
  if (projectStore.rootPath) handleCreate(projectStore.rootPath, type)
}

async function openFolder() {
  await projectActions.openProjectFromPicker()
}

// 项目根一变（无论从哪个入口触发：菜单、Ctrl+O、欢迎页还是这里），树的本地选中
// 都指向已经不存在的节点。放在 watch 里而不是 openFolder 里，才能覆盖全部入口。
watch(() => projectStore.rootPath, () => {
  selectedKeys.value = []
})

onBeforeUnmount(() => {
  if (expandSaveTimer) window.clearTimeout(expandSaveTimer)
})
</script>

<style>
/* 稍微调整 Input 在 Tree 里的样式 */
.n-tree .n-input {
  width: 100%;
  min-width: 100px;
}
.n-tree-node--selected {
  background-color: #37373d !important;
}
.n-tree .n-tree-node-content__prefix {
  width: 30px;
  display: inline-flex;
  justify-content: center;
}
.n-tree .n-tree-node {
  min-height: 24px;
}
.n-tree .n-tree-node-content {
  font-size: 12px;
}
.n-tree .n-tree-node:not(.n-tree-node--selected):hover {
  background-color: rgba(255, 255, 255, 0.05);
}
</style>
