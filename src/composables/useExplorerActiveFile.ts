import { watch, nextTick, type Ref } from 'vue'
import type { useEditorStore } from '../stores/editor'
import type { useProjectStore } from '../stores/project'
import type { useExplorerViewStore } from '../stores/explorerView'
import { normalizePath, pathsEqual } from '../utils/pathNormalize'
import type { FileNode } from '../types'

export function useExplorerActiveFile(editor: ReturnType<typeof useEditorStore>, project: ReturnType<typeof useProjectStore>, explorer: ReturnType<typeof useExplorerViewStore>, expanded: Ref<string[]>, persist: () => void, reveal?: (path: string) => void) {
  function find(nodes: FileNode[], path: string): FileNode | undefined {
    for (const node of nodes) { if (pathsEqual(node.path, path)) return node; const child = node.children && find(node.children, path); if (child) return child }
  }
  watch([() => editor.activePath, () => project.rootPath], async ([path, root], _, cleanup) => {
    let current = true; cleanup(() => { current = false })
    const base = normalizePath(root || ''), active = normalizePath(path || '')
    if (!path || !root || !active.startsWith(base.endsWith('/') ? base : `${base}/`)) { explorer.clearSelection(); return }
    explorer.setSelectedPaths([find(project.files, path)?.path || path])
    const parts = active.slice(base.length).split('/').filter(Boolean).slice(0, -1)
    let parent = base
    for (const part of parts) {
      parent = `${parent.replace(/\/$/, '')}/${part}`
      const node = find(project.files, parent)
      if (!node?.is_dir) break
      try { if (!node.children) await project.loadChildren(node.path) }
      catch { return } // Leave the existing selection; a manual refresh can report missing files.
      if (!current) return
      if (!expanded.value.includes(node.path)) { expanded.value = [...expanded.value, node.path]; persist() }
    }
    if (current) {
      const selected = find(project.files, path)?.path || path
      explorer.setSelectedPaths([selected])
      await nextTick()
      if (current) reveal?.(selected)
    }
  }, { immediate: true })
}
