import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { useEditorStore } from '../stores/editor'
import { useBuildDialogStore } from '../stores/buildDialog'
import { useProjectStore } from '../stores/project'
import { usePublishToolchainResult } from './useToolchainResult'
import {
  decompileEclFile,
  compileEclFile,
  decompileMsgFile,
  compileMsgFile,
  decompileStdFile,
  compileStdFile,
  extractDatFile,
  packDatFile,
  exportRawDmsg,
  exportRawDstd
} from '../api'
import type { MessageApi } from 'naive-ui'
import type { EditorTab } from '../stores/editor'

export interface ToolchainActionsOptions {
  /**
   * naive-ui useMessage 实例，用于 dirty 保存失败等提示。
   * 不传则相应位置静默（只 console 打印）。
   */
  message?: MessageApi
}

/**
 * 工具链动作合集 —— thecl / thmsg / thstd / thdat 的"快速路径"统一入口。
 *
 * 每个 run* 内部完整跑完:dirty 保存 → 调 API → publishToolchainResult → auto-open / refresh →
 * catch 异常也走 publishToolchainResult 报失败卡片。MenuBar 与 BinaryScriptView 共用同一份语义。
 *
 */
export function useToolchainActions({ message }: ToolchainActionsOptions = {}) {
  const editorStore  = useEditorStore()
  const buildDialog  = useBuildDialogStore()
  const projectStore = useProjectStore()
  const { publishToolchainResult } = usePublishToolchainResult()

  // 内部:tab dirty 时尝试保存,失败提示用户并取消操作
  async function ensureSavedIfDirty(tab: EditorTab | null | undefined) {
    if (!tab || !tab.isDirty) return true
    try {
      const saved = await editorStore.saveActiveFile()
      if (!saved) {
        message?.error('保存当前文件失败,操作已取消')
        return false
      }
      return true
    } catch (e) {
      message?.error(`保存失败:${String(e)}`)
      return false
    }
  }

  async function autoOpen(path: string | null | undefined) {
    if (!path) return
    try {
      await editorStore.openFile({ path })
    } catch (e) {
      console.warn('自动打开产物失败', path, e)
    }
  }

  async function refreshTree() {
    try {
      await projectStore.refresh()
    } catch (e) {
      console.warn('刷新文件树失败', e)
    }
  }

  // ===== ECL =====

  async function runDecompileEclQuick(path: string | null | undefined) {
    if (!path) return
    try {
      const result = await decompileEclFile({ sourcePath: path, mapPaths: [] })
      publishToolchainResult({
        tool: 'thecl',
        operation: 'decompile',
        inputPath: path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
      if (result?.success && result.outputPath && result.outputPath !== path) {
        await autoOpen(result.outputPath)
      }
    } catch (error) {
      publishToolchainResult({
        tool: 'thecl',
        operation: 'decompile',
        inputPath: path,
        success: false,
        message: String(error)
      })
    }
  }

  function runDecompileEclAdvanced(path: string | null | undefined) {
    if (!path) return
    buildDialog.openDialog({
      tool: 'thecl',
      mode: 'decompile',
      inputPath: path,
      version: '',
      useShiftJis: true
    })
  }

  async function runCompileEclQuick(tab: EditorTab | null | undefined) {
    if (!tab?.path) return
    if (!(await ensureSavedIfDirty(tab))) return
    try {
      const result = await compileEclFile({ sourcePath: tab.path, mapPaths: [] })
      publishToolchainResult({
        tool: 'thecl',
        operation: 'compile',
        inputPath: tab.path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
    } catch (error) {
      publishToolchainResult({
        tool: 'thecl',
        operation: 'compile',
        inputPath: tab.path,
        success: false,
        message: String(error)
      })
    }
  }

  function runCompileEclAdvanced(tab: EditorTab | null | undefined) {
    if (!tab?.path) return
    buildDialog.openDialog({
      tool: 'thecl',
      mode: 'compile',
      inputPath: tab.path,
      version: '',
      useShiftJis: true
    })
  }

  function runGenerateEclHeader(tab: EditorTab | null | undefined) {
    if (!tab?.path) return
    buildDialog.openDialog({
      tool: 'thecl',
      mode: 'header',
      inputPath: tab.path,
      version: '',
      useShiftJis: true
    })
  }

  // ===== MSG =====

  async function runDecompileMsg(
    path: string | null | undefined,
    encoding: string | null = null
  ) {
    if (!path) return
    try {
      const result = await decompileMsgFile({ inputPath: path, encoding })
      publishToolchainResult({
        tool: 'thmsg',
        operation: 'decompile',
        inputPath: path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
      if (result?.success && result.outputPath && result.outputPath !== path) {
        await autoOpen(result.outputPath)
      }
    } catch (error) {
      publishToolchainResult({
        tool: 'thmsg',
        operation: 'decompile',
        inputPath: path,
        success: false,
        message: String(error)
      })
    }
  }

  async function runCompileMsg(
    tab: EditorTab | null | undefined,
    encoding: string | null = null
  ) {
    if (!tab?.path) return
    if (!(await ensureSavedIfDirty(tab))) return
    try {
      const result = await compileMsgFile({ inputPath: tab.path, encoding })
      publishToolchainResult({
        tool: 'thmsg',
        operation: 'compile',
        inputPath: tab.path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
    } catch (error) {
      publishToolchainResult({
        tool: 'thmsg',
        operation: 'compile',
        inputPath: tab.path,
        success: false,
        message: String(error)
      })
    }
  }

  // ===== STD =====

  async function runDecompileStd(path: string | null | undefined) {
    if (!path) return
    try {
      const result = await decompileStdFile({ inputPath: path })
      publishToolchainResult({
        tool: 'thstd',
        operation: 'decompile',
        inputPath: path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
      if (result?.success && result.outputPath && result.outputPath !== path) {
        await autoOpen(result.outputPath)
      }
    } catch (error) {
      publishToolchainResult({
        tool: 'thstd',
        operation: 'decompile',
        inputPath: path,
        success: false,
        message: String(error)
      })
    }
  }

  async function runCompileStd(tab: EditorTab | null | undefined) {
    if (!tab?.path) return
    if (!(await ensureSavedIfDirty(tab))) return
    try {
      const result = await compileStdFile({ inputPath: tab.path })
      publishToolchainResult({
        tool: 'thstd',
        operation: 'compile',
        inputPath: tab.path,
        outputPath: result?.outputPath || null,
        success: Boolean(result?.success),
        message: result?.message || ''
      })
    } catch (error) {
      publishToolchainResult({
        tool: 'thstd',
        operation: 'compile',
        inputPath: tab.path,
        success: false,
        message: String(error)
      })
    }
  }

  // ===== DAT =====

  // /p/th17.dat → /p/th17 (默认抽取目录:与归档同级的同名兄弟目录)
  function deriveDefaultExtractDir(archivePath: string): string {
    const lastSep = Math.max(archivePath.lastIndexOf('/'), archivePath.lastIndexOf('\\'))
    const dir = archivePath.slice(0, lastSep + 1)
    const stem = archivePath.slice(lastSep + 1).replace(/\.dat$/i, '')
    return dir + stem
  }

  async function runExtractDat(archivePath?: string | null) {
    if (!archivePath) {
      const picked = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: 'Touhou DAT', extensions: ['dat'] }]
      })
      if (!picked) return
      archivePath = picked
    }
    const defaultDir = deriveDefaultExtractDir(archivePath)
    const targetDir = await openDialog({ directory: true, defaultPath: defaultDir })
    if (!targetDir) return
    try {
      const result = await extractDatFile({ archivePath, targetDir })
      publishToolchainResult({
        tool: 'thdat',
        operation: 'extract',
        inputPath: archivePath,
        outputPath: targetDir,
        success: Boolean(result?.success),
        message: result?.message || '',
        extra: typeof result?.fileCount === 'number' ? `(${result.fileCount} 个文件)` : ''
      })
      await refreshTree()
    } catch (error) {
      publishToolchainResult({
        tool: 'thdat',
        operation: 'extract',
        inputPath: archivePath,
        success: false,
        message: String(error)
      })
    }
  }

  /**
   * 导出原始格式：把磁盘上的方言文件翻译回 ins_N 形式。
   *
   * 存在的理由是 thmsg / thstd 没有 -m，指令名是 IDE 加的——项目里的
   * .dmsg / .dstd 直接跑命令行会在每条带名字的指令上失败。
   */
  async function runExportRaw(kind: 'msg' | 'std', path: string | null | undefined) {
    if (!path) return
    const ext = kind === 'msg' ? 'dmsg' : 'dstd'
    const outputPath = await saveDialog({
      defaultPath: path.replace(new RegExp(`\\.${ext}$`), `.raw.${ext}`),
      filters: [{ name: `原始 ${ext}`, extensions: [ext] }]
    })
    if (!outputPath) return
    try {
      const written =
        kind === 'msg'
          ? await exportRawDmsg(path, outputPath)
          : await exportRawDstd(path, outputPath)
      publishToolchainResult({
        tool: kind === 'msg' ? 'thmsg' : 'thstd',
        operation: 'export-raw',
        inputPath: path,
        outputPath: written,
        success: true,
        message: `已导出原始格式，可直接用命令行编译：${written}`
      })
    } catch (error) {
      publishToolchainResult({
        tool: kind === 'msg' ? 'thmsg' : 'thstd',
        operation: 'export-raw',
        inputPath: path,
        outputPath: null,
        success: false,
        message: String(error)
      })
    }
  }

  async function runPackDat() {
    const sourceDir = await openDialog({ directory: true })
    if (!sourceDir) return
    const archivePath = await saveDialog({
      filters: [{ name: 'Touhou DAT', extensions: ['dat'] }]
    })
    if (!archivePath) return
    try {
      const result = await packDatFile({ sourceDir, archivePath })
      publishToolchainResult({
        tool: 'thdat',
        operation: 'pack',
        inputPath: sourceDir,
        outputPath: archivePath,
        success: Boolean(result?.success),
        message: result?.message || '',
        extra: typeof result?.fileCount === 'number' ? `(${result.fileCount} 个文件)` : ''
      })
      await refreshTree()
    } catch (error) {
      publishToolchainResult({
        tool: 'thdat',
        operation: 'pack',
        inputPath: sourceDir,
        success: false,
        message: String(error)
      })
    }
  }

  return {
    // ECL
    runDecompileEclQuick,
    runDecompileEclAdvanced,
    runCompileEclQuick,
    runCompileEclAdvanced,
    runGenerateEclHeader,
    // MSG
    runDecompileMsg,
    runCompileMsg,
    runExportRaw,
    // STD
    runDecompileStd,
    runCompileStd,
    // DAT
    runExtractDat,
    runPackDat
  }
}
