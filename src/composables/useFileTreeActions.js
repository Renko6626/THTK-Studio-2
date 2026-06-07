import { copyEntry, renameEntry, deleteEntry, getFileClipboard, setFileClipboard } from '../api'

/**
 * 文件树的剪切/复制/粘贴/删除操作
 */
export function useFileTreeActions({
  selectedKeys,
  projectStore,
  editorStore,
  explorerClipboardStore,
  explorerViewStore,
  dialog,
  message
}) {

  // ---- 路径工具 ----

  function getParentPath(path) {
    const normalized = path.replace(/[\\/]+$/, '')
    const lastSlash = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))
    return lastSlash > 0 ? normalized.slice(0, lastSlash) : normalized
  }

  function isPathWithin(path, root) {
    return path === root || path.startsWith(`${root}\\`) || path.startsWith(`${root}/`)
  }

  function joinPath(dir, name) {
    const separator = dir.includes('\\') ? '\\' : '/'
    return `${dir.replace(/[\\/]+$/, '')}${separator}${name}`
  }

  function splitName(name) {
    const dotIndex = name.lastIndexOf('.')
    if (dotIndex <= 0) return { stem: name, ext: '' }
    return { stem: name.slice(0, dotIndex), ext: name.slice(dotIndex) }
  }

  function findNodeByPath(nodes, path) {
    for (const node of nodes) {
      if (node.path === path) return node
      if (node.children?.length) {
        const match = findNodeByPath(node.children, path)
        if (match) return match
      }
    }
    return null
  }

  function getExistingNamesForDir(destinationDir) {
    if (destinationDir === projectStore.rootPath) {
      return new Set(projectStore.files.map(child => child.name.toLowerCase()))
    }
    const destinationNode = findNodeByPath(projectStore.files, destinationDir)
    return new Set((destinationNode?.children || []).map(child => child.name.toLowerCase()))
  }

  function getActionEntries(target) {
    const selectedEntries = selectedKeys.value
      .map(path => findNodeByPath(projectStore.files, path))
      .filter(Boolean)

    if (!selectedEntries.length) return target ? [target] : []
    if (target?.path && selectedEntries.some(entry => entry.path === target.path)) return selectedEntries
    return target ? [target] : selectedEntries
  }

  function resolveDestinationDir(targetNode) {
    if (targetNode.is_dir) return targetNode.path
    return getParentPath(targetNode.path)
  }

  function canCopyEntryIntoDir(entry, destinationDir) {
    if (!entry?.path || !destinationDir) return false
    if (destinationDir === entry.path) return false
    if (entry.is_dir && isPathWithin(destinationDir, entry.path)) return false
    return true
  }

  function canMoveEntryIntoDir(entry, destinationDir) {
    if (!entry?.path || !destinationDir) return false
    if (destinationDir === entry.path) return false
    if (entry.is_dir && isPathWithin(destinationDir, entry.path)) return false
    return getParentPath(entry.path) !== destinationDir
  }

  function makeUniqueDestinationName(baseName, destinationDir, existingNames = null) {
    const names = existingNames || getExistingNamesForDir(destinationDir)
    if (!names.has(baseName.toLowerCase())) return baseName

    const { stem, ext } = splitName(baseName)
    let index = 1
    while (true) {
      const candidate = index === 1 ? `${stem} copy${ext}` : `${stem} copy ${index}${ext}`
      if (!names.has(candidate.toLowerCase())) return candidate
      index += 1
    }
  }

  // ---- 操作 ----

  async function copyPath(path) {
    try {
      await navigator.clipboard.writeText(path)
      message.success(path.includes('\n') ? '文件路径列表已复制' : '文件路径已复制')
    } catch {
      message.error('复制路径失败')
    }
  }

  async function copyEntries(entries) {
    if (!entries.length) return
    explorerClipboardStore.setCopy(entries)
    try { await setFileClipboard(entries.map(e => e.path)) } catch { /* best-effort */ }
    message.success(entries.length > 1 ? `已复制 ${entries.length} 个项目` : '已复制')
  }

  async function cutEntries(entries) {
    if (!entries.length) return
    explorerClipboardStore.setCut(entries)
    try { await setFileClipboard(entries.map(e => e.path)) } catch { /* best-effort */ }
    message.success(entries.length > 1 ? `已剪切 ${entries.length} 个项目` : '已剪切')
  }

  async function deleteEntries(entries) {
    if (!entries.length) return

    const label = entries.length === 1 ? entries[0].name : `${entries.length} 个项目`
    dialog.warning({
      title: '确认删除',
      content: `确定要永久删除 ${label} 吗？`,
      positiveText: '删除',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          for (const entry of [...entries].sort((a, b) => b.path.length - a.path.length)) {
            await deleteEntry(entry.path)
            editorStore.closeTabsUnderPath(entry.path)
          }
          selectedKeys.value = []
          explorerViewStore.clearSelection()
          await projectStore.refresh()
        } catch (error) {
          message.error(String(error))
        }
      }
    })
  }

  async function pasteIntoTarget(targetNode) {
    let entries = explorerClipboardStore.entries

    if (!entries.length) {
      try {
        const systemClipboard = await getFileClipboard()
        entries = (systemClipboard?.paths || []).map(path => ({
          path,
          name: path.split(/[\\/]/).pop(),
          is_dir: false
        }))
      } catch {
        entries = []
      }
    }
    if (!entries.length) return

    const destinationDir = resolveDestinationDir(targetNode)
    const existingNames = getExistingNamesForDir(destinationDir)
    let movedAny = false

    try {
      for (const entry of entries) {
        if (explorerClipboardStore.isCut && destinationDir === getParentPath(entry.path)) continue

        const canPlace = explorerClipboardStore.isCut
          ? canMoveEntryIntoDir(entry, destinationDir)
          : canCopyEntryIntoDir(entry, destinationDir)

        if (!canPlace) continue

        const destinationName = makeUniqueDestinationName(entry.name, destinationDir, existingNames)
        existingNames.add(destinationName.toLowerCase())
        const destinationPath = joinPath(destinationDir, destinationName)

        if (explorerClipboardStore.isCut) {
          await renameEntry(entry.path, destinationPath)
          editorStore.handlePathRename(entry.path, destinationPath)
          movedAny = true
        } else {
          await copyEntry(entry.path, destinationPath)
        }
      }

      if (explorerClipboardStore.isCut && movedAny) explorerClipboardStore.clear()

      selectedKeys.value = []
      explorerViewStore.clearSelection()
      await projectStore.refresh()
    } catch (error) {
      message.error(String(error))
    }
  }

  return {
    // 路径工具（DnD 也需要用）
    getParentPath,
    isPathWithin,
    joinPath,
    findNodeByPath,
    getExistingNamesForDir,
    getActionEntries,
    canMoveEntryIntoDir,
    makeUniqueDestinationName,
    // 操作
    copyPath,
    copyEntries,
    cutEntries,
    deleteEntries,
    pasteIntoTarget
  }
}
