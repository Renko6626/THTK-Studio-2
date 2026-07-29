import { copyEntry, renameEntry, deleteEntry, getFileClipboard, setFileClipboard, statEntry } from '../api'
import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import type { Ref } from 'vue'
import type { DialogApi, MessageApi } from 'naive-ui'
import type { FileNode } from '../types'
import type { useEditorStore } from '../stores/editor'
import type { useExplorerClipboardStore } from '../stores/explorerClipboard'
import type { useExplorerViewStore } from '../stores/explorerView'
import type { useProjectStore } from '../stores/project'

/**
 * 粘贴操作作用的条目。
 *
 * 来自文件树时是完整的 FileNode；来自**系统剪贴板**时只能拿到 path，
 * 其余字段由 statEntry 探测补齐（拿不到 size / category / lossy 等），
 * 所以这里只要求这三个字段。
 */
export type PasteEntry = Pick<FileNode, 'path' | 'name' | 'is_dir'>

export interface FileTreeActionsDeps {
  selectedKeys: Ref<string[]>
  projectStore: ReturnType<typeof useProjectStore>
  editorStore: ReturnType<typeof useEditorStore>
  explorerClipboardStore: ReturnType<typeof useExplorerClipboardStore>
  explorerViewStore: ReturnType<typeof useExplorerViewStore>
  dialog: DialogApi
  message: MessageApi
}

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
}: FileTreeActionsDeps) {

  // ---- 路径工具 ----

  function getParentPath(path: string): string {
    const normalized = path.replace(/[\\/]+$/, '')
    const lastSlash = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))
    return lastSlash > 0 ? normalized.slice(0, lastSlash) : normalized
  }

  function isPathWithin(path: string, root: string): boolean {
    return path === root || path.startsWith(`${root}\\`) || path.startsWith(`${root}/`)
  }

  function joinPath(dir: string, name: string): string {
    const separator = dir.includes('\\') ? '\\' : '/'
    return `${dir.replace(/[\\/]+$/, '')}${separator}${name}`
  }

  function splitName(name: string): { stem: string; ext: string } {
    const dotIndex = name.lastIndexOf('.')
    if (dotIndex <= 0) return { stem: name, ext: '' }
    return { stem: name.slice(0, dotIndex), ext: name.slice(dotIndex) }
  }

  function findNodeByPath(nodes: FileNode[], path: string): FileNode | null {
    for (const node of nodes) {
      if (node.path === path) return node
      if (node.children?.length) {
        const match = findNodeByPath(node.children, path)
        if (match) return match
      }
    }
    return null
  }

  function getExistingNamesForDir(destinationDir: string): Set<string> {
    if (destinationDir === projectStore.rootPath) {
      return new Set(projectStore.files.map(child => child.name.toLowerCase()))
    }
    const destinationNode = findNodeByPath(projectStore.files, destinationDir)
    return new Set((destinationNode?.children || []).map(child => child.name.toLowerCase()))
  }

  function getActionEntries(target?: FileNode | null): FileNode[] {
    const selectedEntries = selectedKeys.value
      .map(path => findNodeByPath(projectStore.files, path))
      // 选中项可能指向已被删除的节点，findNodeByPath 返回 null
      .filter((entry): entry is FileNode => entry !== null)

    if (!selectedEntries.length) return target ? [target] : []
    if (target?.path && selectedEntries.some(entry => entry.path === target.path)) return selectedEntries
    return target ? [target] : selectedEntries
  }

  function resolveDestinationDir(targetNode: FileNode): string {
    if (targetNode.is_dir) return targetNode.path
    return getParentPath(targetNode.path)
  }

  function canCopyEntryIntoDir(entry: PasteEntry | null, destinationDir: string | null): boolean {
    if (!entry?.path || !destinationDir) return false
    if (destinationDir === entry.path) return false
    if (entry.is_dir && isPathWithin(destinationDir, entry.path)) return false
    return true
  }

  function canMoveEntryIntoDir(entry: PasteEntry | null, destinationDir: string | null): boolean {
    if (!entry?.path || !destinationDir) return false
    if (destinationDir === entry.path) return false
    if (entry.is_dir && isPathWithin(destinationDir, entry.path)) return false
    return getParentPath(entry.path) !== destinationDir
  }

  function makeUniqueDestinationName(
    baseName: string,
    destinationDir: string,
    existingNames: Set<string> | null = null
  ): string {
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

  async function copyPath(path: string) {
    try {
      await navigator.clipboard.writeText(path)
      message.success(path.includes('\n') ? '文件路径列表已复制' : '文件路径已复制')
    } catch {
      message.error('复制路径失败')
    }
  }

  async function copyEntries(entries: FileNode[]) {
    if (!entries.length) return
    const lossyEntry = entries.find(e => e.lossy)
    if (lossyEntry) {
      message.error('该文件名含非 UTF-8 字符，IDE 暂不支持对其操作。请用系统文件管理器处理。')
      return
    }
    explorerClipboardStore.setCopy(entries)
    try { await setFileClipboard(entries.map(e => e.path)) } catch { /* best-effort */ }
    message.success(entries.length > 1 ? `已复制 ${entries.length} 个项目` : '已复制')
  }

  async function cutEntries(entries: FileNode[]) {
    if (!entries.length) return
    const lossyEntry = entries.find(e => e.lossy)
    if (lossyEntry) {
      message.error('该文件名含非 UTF-8 字符，IDE 暂不支持对其操作。请用系统文件管理器处理。')
      return
    }
    explorerClipboardStore.setCut(entries)
    try { await setFileClipboard(entries.map(e => e.path)) } catch { /* best-effort */ }
    message.success(entries.length > 1 ? `已剪切 ${entries.length} 个项目` : '已剪切')
  }

  async function deleteEntries(entries: FileNode[]) {
    if (!entries.length) return
    const lossyEntry = entries.find(e => e.lossy)
    if (lossyEntry) {
      message.error('该文件名含非 UTF-8 字符，IDE 暂不支持对其操作。请用系统文件管理器处理。')
      return
    }

    const label = entries.length === 1 ? entries[0].name : `${entries.length} 个项目`
    dialog.warning({
      title: '确认删除',
      content: `确定要永久删除 ${label} 吗？`,
      positiveText: '删除',
      negativeText: '取消',
      onPositiveClick: async () => {
        const reportsStore = useWorkbenchReportsStore()
        const succeeded = []
        const failed = []

        projectStore.isLoading = true
        try {
          for (const entry of [...entries].sort((a, b) => b.path.length - a.path.length)) {
            try {
              await deleteEntry(entry.path)
              editorStore.closeTabsUnderPath(entry.path)
              succeeded.push(entry.name)
            } catch (err) {
              failed.push({ name: entry.name, error: String(err) })
            }
          }

          selectedKeys.value = []
          explorerViewStore.clearSelection()
          await projectStore.refresh().catch(() => {})
        } finally {
          projectStore.isLoading = false
        }

        if (succeeded.length === 0 && failed.length === 0) return

        const summary = failed.length > 0
          ? `已删除 ${succeeded.length} 个，失败 ${failed.length} 个`
          : `已删除 ${succeeded.length} 个`
        const allSucceeded = failed.length === 0 && succeeded.length > 0
        const allFailed = succeeded.length === 0 && failed.length > 0

        reportsStore.publishToolResult({
          ownerKey: `delete:${entries[0].path}`,
          source: 'file-tree',
          operation: 'delete',
          scriptKind: 'fs',
          title: allSucceeded ? '删除完成' : (allFailed ? '删除失败' : '删除部分完成'),
          path: entries[0].path,
          success: allSucceeded,
          message: failed.length > 0
            ? `${summary}\n首个错误：${failed[0].error}`
            : summary,
          diagnostics: []
        })

        if (allSucceeded) message.success(summary)
        else if (allFailed) message.error(`删除失败：${failed[0].error}`)
        else message.warning(summary)
      }
    })
  }

  async function pasteIntoTarget(targetNode: FileNode) {
    const reportsStore = useWorkbenchReportsStore()
    const isCut = explorerClipboardStore.isCut
    const failed = []
    let entries = []
    let usingInternal = false

    if (explorerClipboardStore.entries.length) {
      // 内部剪贴板：is_dir 已在 setCopy/setCut 时正确写入
      usingInternal = true
      entries = explorerClipboardStore.entries.map(e => ({ ...e }))
    } else {
      // 系统剪贴板：路径来自 OS 文件管理器，is_dir 未知 → 必须 stat 探测
      let systemPaths: string[] = []
      try {
        const systemClipboard = await getFileClipboard()
        systemPaths = systemClipboard?.paths || []
      } catch {
        systemPaths = []
      }

      for (const path of systemPaths) {
        const name = path.split(/[\\/]/).pop() || path
        try {
          const stat = await statEntry(path)
          if (!stat?.exists) {
            failed.push({ name, error: '源路径不存在' })
            continue
          }
          entries.push({ path, name, is_dir: !!stat.isDir })
        } catch (err) {
          failed.push({ name, error: String(err) })
        }
      }
    }

    if (!entries.length && !failed.length) return

    const destinationDir = resolveDestinationDir(targetNode)
    const existingNames = getExistingNamesForDir(destinationDir)
    const succeeded = []

    projectStore.isLoading = true
    try {
      for (const entry of entries) {
        try {
          // 剪切到原目录：直接跳过（不算成功也不算失败）
          if (isCut && destinationDir === getParentPath(entry.path)) continue

          const canPlace = isCut
            ? canMoveEntryIntoDir(entry, destinationDir)
            : canCopyEntryIntoDir(entry, destinationDir)

          if (!canPlace) {
            failed.push({ name: entry.name, error: '目标位置无法放置（自身/子目录/同位置）' })
            continue
          }

          const destinationName = makeUniqueDestinationName(entry.name, destinationDir, existingNames)
          existingNames.add(destinationName.toLowerCase())
          const destinationPath = joinPath(destinationDir, destinationName)

          if (isCut) {
            await renameEntry(entry.path, destinationPath)
            editorStore.handlePathRename(entry.path, destinationPath)
          } else {
            await copyEntry(entry.path, destinationPath)
          }
          succeeded.push({ name: entry.name, destinationPath })
        } catch (err) {
          failed.push({ name: entry.name, error: String(err) })
        }
      }

      // 全部成功且为剪切才清空内部剪贴板；任一失败则保留供用户重试
      if (usingInternal && isCut && failed.length === 0 && succeeded.length > 0) {
        explorerClipboardStore.clear()
      }

      selectedKeys.value = []
      explorerViewStore.clearSelection()

      // 即便全部失败也要 refresh：后端可能已部分修改
      await projectStore.refresh().catch(() => {})
    } finally {
      projectStore.isLoading = false
    }

    const total = succeeded.length + failed.length
    if (total === 0) return

    const summary = failed.length > 0
      ? `已粘贴 ${succeeded.length} 个，失败 ${failed.length} 个`
      : `已粘贴 ${succeeded.length} 个`
    const allSucceeded = failed.length === 0 && succeeded.length > 0
    const allFailed = succeeded.length === 0 && failed.length > 0

    reportsStore.publishToolResult({
      ownerKey: `paste:${targetNode.path}`,
      source: 'file-tree',
      operation: 'paste',
      scriptKind: 'fs',
      title: allSucceeded ? '粘贴完成' : (allFailed ? '粘贴失败' : '粘贴部分完成'),
      path: targetNode.path,
      success: allSucceeded,
      message: failed.length > 0
        ? `${summary}\n首个错误：${failed[0].error}`
        : summary,
      diagnostics: []
    })

    if (allSucceeded) message.success(summary)
    else if (allFailed) message.error(`粘贴失败：${failed[0].error}`)
    else message.warning(summary)
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
