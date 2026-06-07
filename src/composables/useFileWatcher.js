import { onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { readFile } from '../api'

/**
 * 监听 Rust 端发出的文件系统变更事件，
 * 当已打开的文件被外部修改或删除时自动处理。
 *
 * - 未修改的文件：静默重新加载
 * - 有未保存修改的文件：提示用户选择
 * - 被删除的文件：通知用户
 */
export function useFileWatcher({ editorStore, projectStore, showReloadNotice }) {
  let unlisten = null
  const pendingPaths = new Set()

  async function handleFileChanges(event) {
    const changes = event.payload
    if (!Array.isArray(changes) || !changes.length) return

    let reloadedCount = 0
    let deletedCount = 0

    for (const change of changes) {
      const tab = editorStore.tabs.find(t => t.path === change.path)
      if (!tab || tab.viewType !== 'text') continue
      if (pendingPaths.has(change.path)) continue
      pendingPaths.add(change.path)

      try {
        if (change.kind === 'remove') {
          deletedCount++
        } else if (change.kind === 'modify') {
          if (tab.isDirty) {
            // 有未保存修改，询问用户
            const reload = window.confirm(
              `"${tab.name}" 已被外部修改，但你有未保存的更改。\n\n点击"确定"重新加载（丢弃你的更改），或"取消"保留你的更改。`
            )
            if (reload) {
              await reloadTab(tab)
              reloadedCount++
            }
          } else {
            // 无未保存修改，先检查内容是否真的变了（排除自身保存触发的事件）
            const changed = await reloadIfChanged(tab)
            if (changed) reloadedCount++
          }
        }
      } finally {
        pendingPaths.delete(change.path)
      }
    }

    // 任何外部变更都刷新文件树:debouncer 只能产出 modify/remove
    // (新建文件首次出现时 exists → 被报为 "modify",永远等不到 "create"),
    // 因此不能按 kind 过滤,否则终端/agent 新建的文件在树里不可见。
    // 事件已在 Rust 侧做 500ms 防抖,浅层重扫开销可接受。
    projectStore.refresh().catch(() => {
      // 根目录可能瞬时不可读(构建中等),下一次事件会再刷
    })

    if (reloadedCount > 0) {
      showReloadNotice(`已重新加载 ${reloadedCount} 个外部变更的文件`)
    }
    if (deletedCount > 0) {
      showReloadNotice(`${deletedCount} 个已打开的文件被外部删除`)
    }
  }

  async function reloadTab(tab) {
    try {
      const content = await readFile(tab.path)
      tab.content = content
      tab.originalContent = content
      tab.isDirty = false
    } catch {
      // 文件可能已被删除，忽略
    }
  }

  /** 仅当磁盘内容与当前内容不同时才重新加载，返回是否实际发生了重载 */
  async function reloadIfChanged(tab) {
    try {
      const content = await readFile(tab.path)
      if (content === tab.content) return false
      tab.content = content
      tab.originalContent = content
      tab.isDirty = false
      return true
    } catch {
      return false
    }
  }

  async function startListening() {
    unlisten = await listen('file-system-changed', handleFileChanges)
  }

  onMounted(() => {
    startListening()
  })

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })
}
