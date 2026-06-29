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

<script setup>
import { computed, ref, h, nextTick, watch, onBeforeUnmount } from 'vue'
import { NTree, NSpin, NButton, NIcon, NDropdown, NInput, useDialog, useMessage } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
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
import { useFileOperations, remapExpandedKeys } from '../../composables/useFileOperations'
import { useFileTreeActions } from '../../composables/useFileTreeActions'
import { useFileTreeDnD } from '../../composables/useFileTreeDnD'
import { renderFileIcon } from '../../utils/renderFileIcon'
import { renameEntry } from '../../api'

const projectStore = useProjectStore()
const editorStore = useEditorStore()
const explorerClipboardStore = useExplorerClipboardStore()
const explorerViewStore = useExplorerViewStore()
const dialog = useDialog()
const message = useMessage()

// ---- Composables ----

const { inputState, handleCreate, handleRename, submitInput, cancelInput } = useFileOperations()

const selectedKeys = ref([])

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
const expandedKeys = ref([])
let expandSaveTimer = null

// 记录最近一次完成恢复时对应的 rootPath，避免重复对同一项目恢复
const lastRestoredFor = ref(null)
let restoringPromise = null

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

function handleExpand(keys) {
  expandedKeys.value = keys
  persistExpandedKeys()
}

function handleSelectKeys(keys) {
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
    const allPaths = new Set()
    function collectPaths(nodes) {
      for (const node of nodes) {
        allPaths.add(node.path)
        if (node.children) collectPaths(node.children)
      }
    }
    collectPaths(projectStore.files)
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
  draggingNode,
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

async function handleLazyLoad(node) {
  if (!node.is_dir) return
  await projectStore.loadChildren(node.path)
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

function injectTempNode(nodes) {
  return nodes.map(node => {
    const newNode = { ...node }

    if (inputState.type === 'create' && node.path === inputState.targetPath) {
      const tempNode = {
        path: tempKey.value,
        name: '',
        is_dir: inputState.fileType === 'dir',
        extension: inputState.fileType === 'file' ? 'txt' : null,
        isTemp: true,
        isLeaf: true,
      }
      newNode.children = newNode.children ? [tempNode, ...newNode.children] : [tempNode]
      newNode.children = newNode.children.map(child => {
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
  let rawData = projectStore.files

  if (inputState.type === 'create' && inputState.targetPath === projectStore.rootPath) {
    const tempNode = {
      path: tempKey.value,
      name: '',
      is_dir: inputState.fileType === 'dir',
      isTemp: true,
      isLeaf: true,
    }
    rawData = [tempNode, ...rawData]
  }

  return injectTempNode(rawData)
})

// ---- 节点渲染 ----

function transformNode(node) {
  return {
    ...node,
    prefix: (ctx) => renderFileIcon(node, ctx?.expanded ?? false)
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
  onRenamed: (oldPath, newPath) => {
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

const renderLabel = ({ option }) => {
  const isCreating = option.isTemp
  const isRenaming = inputState.type === 'rename' && option.path === inputState.targetPath

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

  return option.name
}

// ---- 节点交互 ----

const nodeProps = ({ option }) => ({
  onClick(e) {
    const isMultiSelect = e?.ctrlKey || e?.metaKey || e?.shiftKey
    if (!isMultiSelect) {
      selectedKeys.value = [option.path]
      explorerViewStore.setSelectedPaths([option.path])
    }
    if (!option.is_dir && !option.isTemp && !isMultiSelect) editorStore.openFile(option)
  },
  onContextmenu(e) {
    if (!selectedKeys.value.includes(option.path)) {
      selectedKeys.value = [option.path]
      explorerViewStore.setSelectedPaths([option.path])
    }
    handleContextMenu(e, option)
  },
  onDragover(event) {
    if (option.is_dir) {
      event?.preventDefault?.()
      if (event?.dataTransfer) event.dataTransfer.dropEffect = 'move'
    }
  }
})

// ---- 右键菜单处理 ----

function onEmptyAreaRightClick(e) {
  if (!projectStore.rootPath) return
  selectedKeys.value = []
  explorerViewStore.clearSelection()
  handleContextMenu(e, { path: projectStore.rootPath, is_dir: true, name: 'root' })
}

function handleMenuSelect(key) {
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

function quickCreate(type) {
  if (projectStore.rootPath) handleCreate(projectStore.rootPath, type)
}

async function openFolder() {
  const selected = await open({ directory: true })
  if (selected) {
    selectedKeys.value = []
    explorerViewStore.clearSelection()
    await projectStore.loadProject(selected)
  }
}

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
