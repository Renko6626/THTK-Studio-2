import { open } from '@tauri-apps/plugin-dialog'
import { useEditorStore } from '../stores/editor'
import { useExplorerViewStore } from '../stores/explorerView'
import { useProjectStore } from '../stores/project'
import { useRecentProjectsStore } from '../stores/recentProjects'
import { pathsEqual } from '../utils/pathNormalize'

/**
 * 项目打开 / 切换的唯一入口。
 *
 * 菜单、快捷键、文件树和欢迎页原本各写一套加载逻辑，错误提示、脏标签保护和
 * 切换后的清理各不相同。所有入口统一走这里，行为才可能一致。
 *
 * 调用方需要传入 naive-ui 的 message / dialog 实例（它们只能在 setup 里取）。
 */
export function useProjectActions({ message, dialog }) {
  const editorStore = useEditorStore()
  const explorerViewStore = useExplorerViewStore()
  const projectStore = useProjectStore()
  const recentProjectsStore = useRecentProjectsStore()

  /**
   * 有未保存修改时的三向选择。用 Promise 包一层，避免把后续流程拆进回调。
   * settled 兜底：不依赖 naive-ui 各回调的触发顺序，谁先到算谁。
   */
  function confirmDirtySwitch(dirtyCount) {
    return new Promise((resolve) => {
      let settled = false
      const finish = (choice) => {
        if (settled) return
        settled = true
        resolve(choice)
      }

      dialog.warning({
        title: '有未保存的修改',
        content: `${dirtyCount} 个文件尚未保存。切换项目会关闭当前项目的所有标签。`,
        positiveText: '保存并切换',
        negativeText: '放弃并切换',
        closable: true,
        maskClosable: false,
        onPositiveClick: () => finish('save'),
        onNegativeClick: () => finish('discard'),
        onClose: () => finish('cancel'),
        onEsc: () => finish('cancel')
      })
    })
  }

  /**
   * 打开指定路径的项目。
   *
   * @param {string} path
   * @param {{ silent?: boolean }} [options] silent 用于会话恢复：失败时不弹错误提示，
   *   由调用方决定怎么呈现（恢复失败应该落到欢迎页，而不是一开机就糊一个红条）。
   * @returns {Promise<boolean>} 是否真的完成了打开 / 切换
   */
  async function openProjectPath(path, { silent = false } = {}) {
    if (!path) return false

    const previousRoot = projectStore.rootPath
    const isSwitching = Boolean(previousRoot) && !pathsEqual(previousRoot, path)

    // 只有真正切换项目才会关掉旧标签，重开同一个项目不会丢东西，不必打扰用户
    if (isSwitching && editorStore.hasDirtyTabs) {
      const dirtyCount = editorStore.tabs.filter(tab => tab.isDirty).length
      const choice = await confirmDirtySwitch(dirtyCount)

      if (choice === 'cancel') return false
      if (choice === 'save') {
        const saved = await editorStore.saveAllFiles()
        if (!saved) {
          // 保存失败就必须停下：继续切换会把没存住的改动一起关掉
          message.error('部分文件保存失败，已取消切换')
          return false
        }
      }
    }

    try {
      await projectStore.loadProject(path)
    } catch (error) {
      if (!silent) {
        message.error(`打开项目失败: ${error}`)
      }
      return false
    }

    if (previousRoot && !pathsEqual(previousRoot, projectStore.rootPath)) {
      // 只清理属于旧项目的标签与选择；项目外打开的文件保持不动
      editorStore.closeTabsUnderPath(previousRoot)
      explorerViewStore.clearSelection()
    }

    // 已运行的 PTY 会话不强制终止，只影响后续新建的终端——
    // terminalStore.openSession 在创建时才读 projectStore.rootPath。
    await recentProjectsStore.refresh()

    if (!silent) {
      message.success(`已打开 ${projectStore.rootName}`)
    }
    return true
  }

  /** 弹原生目录选择器，再走统一的打开流程 */
  async function openProjectFromPicker() {
    const selected = await open({ directory: true, multiple: false })
    if (!selected) return false
    return openProjectPath(String(selected))
  }

  /** 从最近项目列表移除一条，并提示结果 */
  async function removeRecentProject(path) {
    try {
      await recentProjectsStore.remove(path)
    } catch (error) {
      message.error(`移除失败: ${error}`)
    }
  }

  return {
    openProjectPath,
    openProjectFromPicker,
    removeRecentProject
  }
}
