import { h } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { NButton } from 'naive-ui'
import type { DialogApi, MessageApi } from 'naive-ui'
import { useEditorStore } from '../stores/editor'
import { useExplorerClipboardStore } from '../stores/explorerClipboard'
import { useExplorerViewStore } from '../stores/explorerView'
import { useProjectStore } from '../stores/project'
import { useRecentProjectsStore } from '../stores/recentProjects'
import { pathsEqual } from '../utils/pathNormalize'

/** 用户在"有未保存修改"确认框里的选择 */
type DirtySwitchChoice = 'save' | 'discard' | 'cancel'

interface ProjectActionsDeps {
  message: MessageApi
  dialog: DialogApi
}

interface OpenOptions {
  /** 会话恢复用：失败时不弹错误提示，由调用方决定怎么呈现 */
  silent?: boolean
}

/** 与 editorStore.closeTabsUnderPath 同口径：判断某个文件是否属于该项目根 */
function isUnderRoot(filePath: string, root: string): boolean {
  if (!filePath || !root) return false
  return filePath === root || filePath.startsWith(`${root}\\`) || filePath.startsWith(`${root}/`)
}

/**
 * 正在进行中的打开操作。必须是模块级：四个组件各自调用 useProjectActions()，
 * 闭包互不共享，放在闭包里等于没有保护。
 *
 * 需要跨越原生选择器和确认框两个 await —— 这段窗口里 projectStore.isLoading
 * 还是 false，欢迎页那种 :disabled="isLoading" 拦不住。两次并发打开会让后端按
 * 调用顺序提交项目根、前端按解析顺序提交，出现文件树是 A 而工具链/MCP/终端
 * 工作目录指向 B 的错位。
 */
let openInFlight = false

/**
 * 项目打开 / 切换的唯一入口。
 *
 * 菜单、快捷键、文件树和欢迎页原本各写一套加载逻辑，错误提示、脏标签保护和
 * 切换后的清理各不相同。所有入口统一走这里，行为才可能一致。
 *
 * 调用方需要传入 naive-ui 的 message / dialog 实例（它们只能在 setup 里取）。
 */
export function useProjectActions({ message, dialog }: ProjectActionsDeps) {
  const editorStore = useEditorStore()
  const explorerClipboardStore = useExplorerClipboardStore()
  const explorerViewStore = useExplorerViewStore()
  const projectStore = useProjectStore()
  const recentProjectsStore = useRecentProjectsStore()

  /**
   * 有未保存修改时的三向选择。用 Promise 包一层，避免把后续流程拆进回调。
   * settled 兜底：不依赖 naive-ui 各回调的触发顺序，谁先到算谁。
   */
  function confirmDirtySwitch(dirtyCount: number): Promise<DirtySwitchChoice> {
    return new Promise<DirtySwitchChoice>((resolve) => {
      let settled = false
      let instance: ReturnType<DialogApi['warning']> | null = null
      const finish = (choice: DirtySwitchChoice) => {
        if (settled) return
        settled = true
        resolve(choice)
      }
      const choose = (choice: DirtySwitchChoice) => {
        finish(choice)
        instance?.destroy()
      }

      // 自绘三个按钮而不是用 positiveText / negativeText：默认只渲染两个按钮，
      // 取消要靠右上角 × 或 Esc。而低强调的那个按钮（用户肌肉记忆里的"取消"位）
      // 恰好是"放弃并切换"——点错就丢改动。销毁性操作不能藏在默认位置。
      instance = dialog.warning({
        title: '有未保存的修改',
        content: `${dirtyCount} 个文件尚未保存。切换项目会关闭当前项目的所有标签。`,
        closable: true,
        maskClosable: false,
        onClose: () => finish('cancel'),
        onEsc: () => finish('cancel'),
        action: () =>
          h('div', { class: 'flex items-center gap-2' }, [
            h(NButton, { size: 'small', quaternary: true, onClick: () => choose('cancel') },
              { default: () => '取消' }),
            h(NButton, { size: 'small', onClick: () => choose('discard') },
              { default: () => '放弃并切换' }),
            h(NButton, { size: 'small', type: 'primary', onClick: () => choose('save') },
              { default: () => '保存并切换' })
          ])
      })
    })
  }

  /**
   * 打开指定路径的项目。返回是否真的完成了打开 / 切换。
   * silent 用于会话恢复：恢复失败应该落到欢迎页，而不是一开机就糊一个红条。
   */
  async function openProjectPath(path: string, { silent = false }: OpenOptions = {}): Promise<boolean> {
    if (!path) return false
    if (openInFlight) return false

    openInFlight = true
    try {
      return await runOpen(path, silent)
    } finally {
      openInFlight = false
    }
  }

  async function runOpen(path: string, silent: boolean): Promise<boolean> {
    const previousRoot = projectStore.rootPath
    const isSwitching = Boolean(previousRoot) && !pathsEqual(previousRoot, path)

    // 只有真正切换项目才会关掉旧标签，重开同一个项目不会丢东西，不必打扰用户。
    // 只数将被关掉的那些——项目外打开的文件不会被关，拿它们凑数会让用户在
    // 其实没有风险的时候看到确认框。
    const atRiskCount = isSwitching && previousRoot
      ? editorStore.tabs.filter(tab => tab.isDirty && isUnderRoot(tab.path, previousRoot)).length
      : 0

    if (atRiskCount > 0) {
      const choice = await confirmDirtySwitch(atRiskCount)

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
      // 只清理属于旧项目的标签；项目外打开的文件保持不动
      editorStore.closeTabsUnderPath(previousRoot)
      explorerViewStore.clearSelection()
      // 剪贴板里存的是旧项目的绝对路径，留着会让下一次粘贴跨项目搬文件
      explorerClipboardStore.clear()
    } else if (previousRoot) {
      // 重开同一个项目：文件树整棵重建了，旧的选中项已经不对应任何节点
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
  async function openProjectFromPicker(): Promise<boolean> {
    // 选择器本身也要占住重入锁：否则连按两次 Ctrl+O 会开出两个系统对话框
    if (openInFlight) return false

    openInFlight = true
    let selected
    try {
      selected = await open({ directory: true, multiple: false })
    } finally {
      openInFlight = false
    }

    if (!selected) return false
    return openProjectPath(String(selected))
  }

  /** 从最近项目列表移除一条，并提示结果 */
  async function removeRecentProject(path: string): Promise<void> {
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
