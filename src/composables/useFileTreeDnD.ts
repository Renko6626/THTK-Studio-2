import { ref } from 'vue'
import type { Ref } from 'vue'
import type { MessageApi, TreeDragInfo, TreeDropInfo, TreeOption } from 'naive-ui'
import { remapExpandedKeys } from './useFileOperations'
import type { FileNode } from '../types'
import type { useEditorStore } from '../stores/editor'
import type { useExplorerViewStore } from '../stores/explorerView'
import type { useProjectStore } from '../stores/project'

/**
 * naive-ui 给回调传的是 TreeOption（带索引签名的松散结构），不是我们的 FileNode。
 * 树的数据源确实是 FileNode，所以这里在使用处收窄。
 */
function asFileNode(option: TreeOption | null | undefined): FileNode | null {
  return (option as unknown as FileNode) || null
}

/**
 * FileTree.vue 注入的依赖。那几个 helper 目前定义在 FileTree.vue 里，
 * 等 Task 11 迁完组件后可以考虑把它们下沉到本文件或 utils。
 */
export interface FileTreeDnDDeps {
  expandedKeys: Ref<string[]>
  selectedKeys: Ref<string[]>
  projectStore: ReturnType<typeof useProjectStore>
  editorStore: ReturnType<typeof useEditorStore>
  explorerViewStore: ReturnType<typeof useExplorerViewStore>
  message: MessageApi
  persistExpandedKeys: () => void
  canMoveEntryIntoDir: (entry: FileNode | null, destinationDir: string | null) => boolean
  getExistingNamesForDir: (destinationDir: string) => Set<string>
  makeUniqueDestinationName: (
    name: string,
    destinationDir: string,
    existingNames: Set<string>
  ) => string
  joinPath: (dir: string, name: string) => string
  renameEntry: (oldPath: string, newPath: string) => Promise<void>
}

/**
 * 文件树拖放逻辑
 */
export function useFileTreeDnD({
  expandedKeys,
  selectedKeys,
  projectStore,
  editorStore,
  explorerViewStore,
  message,
  persistExpandedKeys,
  canMoveEntryIntoDir,
  getExistingNamesForDir,
  makeUniqueDestinationName,
  joinPath,
  renameEntry
}: FileTreeDnDDeps) {
  const draggingNode = ref<FileNode | null>(null)
  const rootDropActive = ref(false)

  function handleTreeDragStart({ node, event }: TreeDragInfo) {
    draggingNode.value = asFileNode(node)
    event?.dataTransfer?.setData('text/plain', String(node?.path ?? ''))
    if (event?.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
    }
  }

  function handleTreeDragOver() {}

  function handleTreeDragEnd() {
    draggingNode.value = null
    rootDropActive.value = false
  }

  function allowDrop({
    node,
    dropPosition
  }: {
    node: TreeOption
    dropPosition: 'before' | 'inside' | 'after'
  }): boolean {
    const target = asFileNode(node)
    return (
      dropPosition === 'inside' &&
      !!target?.is_dir &&
      !!draggingNode.value?.path &&
      !!draggingNode.value?.name &&
      canMoveEntryIntoDir(draggingNode.value, target.path)
    )
  }

  async function handleTreeDrop({ node, dragNode: rawDragNode, dropPosition }: TreeDropInfo) {
    const target = asFileNode(node)
    const dragNode = asFileNode(rawDragNode)
    if (dropPosition !== 'inside' || !target?.is_dir || !dragNode?.path) {
      draggingNode.value = null
      return
    }

    const destinationDir = target.path
    if (!canMoveEntryIntoDir(dragNode, destinationDir)) {
      draggingNode.value = null
      return
    }

    const existingNames = getExistingNamesForDir(destinationDir)
    const destinationName = makeUniqueDestinationName(dragNode.name, destinationDir, existingNames)
    const destinationPath = joinPath(destinationDir, destinationName)

    try {
      await renameEntry(dragNode.path, destinationPath)
      editorStore.handlePathRename(dragNode.path, destinationPath)
      selectedKeys.value = [destinationPath]
      explorerViewStore.setSelectedPaths([destinationPath])
      // 同步迁移展开状态：dragNode 自身或其子目录可能在 expandedKeys 中
      const remapped = remapExpandedKeys(expandedKeys.value, dragNode.path, destinationPath)
      let nextExpanded = remapped
      if (!nextExpanded.includes(destinationDir)) {
        nextExpanded = [...nextExpanded, destinationDir]
      }
      expandedKeys.value = nextExpanded
      persistExpandedKeys()
      try {
        await projectStore.refresh()
      } catch {
        message.warning('文件树刷新失败，请手动点击刷新按钮同步')
      }
      message.success(`已移动到 ${destinationName}`)
    } catch (error) {
      message.error(String(error))
    } finally {
      draggingNode.value = null
      rootDropActive.value = false
    }
  }

  // 根目录拖放区域

  function handleRootDragEnter(event: DragEvent) {
    if ((event?.dataTransfer?.files?.length ?? 0) > 0 && !draggingNode.value) return
    rootDropActive.value = canMoveEntryIntoDir(draggingNode.value, projectStore.rootPath)
  }

  function handleRootDragOver(event: DragEvent) {
    const allowed = canMoveEntryIntoDir(draggingNode.value, projectStore.rootPath)
    rootDropActive.value = allowed
    if (allowed && event?.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
  }

  function handleRootDragLeave(event: DragEvent) {
    const currentTarget = event?.currentTarget
    if (currentTarget instanceof Node && currentTarget.contains(event.relatedTarget as Node | null)) return
    rootDropActive.value = false
  }

  async function handleRootDrop(event: DragEvent) {
    if ((event?.dataTransfer?.files?.length ?? 0) > 0 && !draggingNode.value) {
      rootDropActive.value = false
      message.info('暂不支持从系统拖入文件，请用顶部"打开文件夹"按钮')
      return
    }
    if (!draggingNode.value || !projectStore.rootPath) {
      rootDropActive.value = false
      return
    }

    const dragNode = draggingNode.value
    const destinationDir = projectStore.rootPath

    if (!canMoveEntryIntoDir(dragNode, destinationDir)) {
      rootDropActive.value = false
      draggingNode.value = null
      return
    }

    const existingNames = getExistingNamesForDir(destinationDir)
    const destinationName = makeUniqueDestinationName(dragNode.name, destinationDir, existingNames)
    const destinationPath = joinPath(destinationDir, destinationName)

    try {
      await renameEntry(dragNode.path, destinationPath)
      editorStore.handlePathRename(dragNode.path, destinationPath)
      selectedKeys.value = [destinationPath]
      explorerViewStore.setSelectedPaths([destinationPath])
      // 同步迁移展开状态（dragNode 及其子目录可能在 expandedKeys 中）
      expandedKeys.value = remapExpandedKeys(expandedKeys.value, dragNode.path, destinationPath)
      persistExpandedKeys()
      try {
        await projectStore.refresh()
      } catch {
        message.warning('文件树刷新失败，请手动点击刷新按钮同步')
      }
      message.success(`已移动到 ${destinationName}`)
    } catch (error) {
      message.error(String(error))
    } finally {
      rootDropActive.value = false
      draggingNode.value = null
    }
  }

  return {
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
  }
}
