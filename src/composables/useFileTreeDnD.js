import { ref } from 'vue'

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
}) {
  const draggingNode = ref(null)
  const rootDropActive = ref(false)

  function handleTreeDragStart({ node, event }) {
    draggingNode.value = node
    event?.dataTransfer?.setData('text/plain', node?.path || '')
    if (event?.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
    }
  }

  function handleTreeDragOver() {}

  function handleTreeDragEnd() {
    draggingNode.value = null
    rootDropActive.value = false
  }

  function allowDrop({ node, dropPosition }) {
    return (
      dropPosition === 'inside' &&
      !!node?.is_dir &&
      !!draggingNode.value?.path &&
      !!draggingNode.value?.name &&
      canMoveEntryIntoDir(draggingNode.value, node.path)
    )
  }

  async function handleTreeDrop({ node, dragNode, dropPosition }) {
    if (dropPosition !== 'inside' || !node?.is_dir || !dragNode?.path) {
      draggingNode.value = null
      return
    }

    const destinationDir = node.path
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
      if (!expandedKeys.value.includes(destinationDir)) {
        expandedKeys.value = [...expandedKeys.value, destinationDir]
        persistExpandedKeys()
      }
      await projectStore.refresh()
      message.success(`已移动到 ${destinationName}`)
    } catch (error) {
      message.error(String(error))
    } finally {
      draggingNode.value = null
      rootDropActive.value = false
    }
  }

  // 根目录拖放区域

  function handleRootDragEnter() {
    rootDropActive.value = canMoveEntryIntoDir(draggingNode.value, projectStore.rootPath)
  }

  function handleRootDragOver(event) {
    const allowed = canMoveEntryIntoDir(draggingNode.value, projectStore.rootPath)
    rootDropActive.value = allowed
    if (allowed && event?.dataTransfer) {
      event.dataTransfer.dropEffect = 'move'
    }
  }

  function handleRootDragLeave(event) {
    if (event?.currentTarget?.contains?.(event.relatedTarget)) return
    rootDropActive.value = false
  }

  async function handleRootDrop() {
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
      await projectStore.refresh()
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
