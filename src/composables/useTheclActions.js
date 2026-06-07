import { computed } from 'vue'
import { useMessage } from 'naive-ui'
import { useEditorStore } from '../stores/editor'
import { useProjectStore } from '../stores/project'
import { useWorkbenchPanelsStore } from '../stores/workbenchPanels'
import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import {
  createTheclRequest,
  executeThecl,
  publishTheclResult
} from '../services/toolchains/thecl'

/** 从项目配置填充 thecl 请求的默认值 */
function applyProjectDefaults(request, projectStore) {
  const pc = projectStore.projectConfig
  if (!pc) return request

  return {
    ...request,
    version: request.version || pc.gameVersion || '',
    mapPaths: request.mapPaths?.length ? request.mapPaths : (pc.mapPaths || []),
    useShiftJis: request.useShiftJis ?? (pc.encoding === 'shift-jis')
  }
}

function getExtension(path) {
  return path?.split('.').pop()?.toLowerCase() || ''
}

export function useTheclActions() {
  const message = useMessage()
  const editorStore = useEditorStore()
  const projectStore = useProjectStore()
  const workbenchPanelsStore = useWorkbenchPanelsStore()
  const reportsStore = useWorkbenchReportsStore()

  const activeExtension = computed(() => getExtension(editorStore.activeTab?.path))
  const canCompileActiveSource = computed(() => activeExtension.value === 'decl')
  const canDecompileActiveBinary = computed(() => activeExtension.value === 'ecl')
  const canGenerateActiveHeader = computed(() => activeExtension.value === 'decl')

  async function runTheclRequest(request, {
    requireSave = false,
    successMessage,
    openOutput = false
  } = {}) {
    const activeTab = editorStore.tabs.find(tab => tab.path === request?.inputPath) || editorStore.activeTab
    if (!request?.inputPath) {
      message.warning('当前没有可处理的脚本文件')
      return null
    }

    if (requireSave && activeTab?.path === request.inputPath && activeTab.isDirty) {
      const saved = await editorStore.saveActiveFile()
      if (!saved) {
        message.error('保存当前文件失败，已取消操作')
        return null
      }
    }

    editorStore.compiling = true

    try {
      const result = await executeThecl(request)
      publishTheclResult(reportsStore, request, result)

      workbenchPanelsStore.showBottomPanel(
        result?.diagnostics?.length ? 'problems' : 'output'
      )

      if (result?.success) {
        await projectStore.refresh()

        if (openOutput && result.outputPath) {
          await editorStore.openFile({ path: result.outputPath })
        }

        if (successMessage) {
          message.success(successMessage)
        }
      } else {
        message.error('thecl 执行失败，请查看输出或问题面板')
      }

      return result
    } catch (error) {
      reportsStore.publishToolResult({
        ownerKey: `thecl:${request.mode}:${request.inputPath}`,
        source: 'thecl',
        operation: request.mode,
        scriptKind: 'ecl',
        title: activeTab?.name || request.inputPath,
        path: request.inputPath,
        success: false,
        message: String(error),
        diagnostics: []
      })
      workbenchPanelsStore.showBottomPanel('output')
      message.error('thecl 调用失败')
      return null
    } finally {
      editorStore.compiling = false
    }
  }

  async function runTheclForActive({
    mode,
    requireSave = false,
    successMessage,
    openOutput = false
  }) {
    const activeTab = editorStore.activeTab
    if (!activeTab?.path) {
      message.warning('当前没有可处理的脚本文件')
      return null
    }

    const request = applyProjectDefaults(
      createTheclRequest({ mode, inputPath: activeTab.path }),
      projectStore
    )

    return runTheclRequest(request, {
      requireSave,
      successMessage,
      openOutput
    })
  }

  async function compileActiveSource() {
    if (!canCompileActiveSource.value) {
      message.warning('当前标签不是 .decl 源文件')
      return null
    }

    return runTheclForActive({
      mode: 'compile',
      requireSave: true,
      successMessage: '编译完成'
    })
  }

  async function decompileActiveBinary() {
    if (!canDecompileActiveBinary.value) {
      message.warning('当前标签不是 .ecl 二进制脚本')
      return null
    }

    return runTheclForActive({
      mode: 'decompile',
      successMessage: '反编译完成',
      openOutput: true
    })
  }

  async function generateHeaderForActiveSource() {
    if (!canGenerateActiveHeader.value) {
      message.warning('当前标签不是 .decl 源文件')
      return null
    }

    return runTheclForActive({
      mode: 'header',
      requireSave: true,
      successMessage: '头文件已生成',
      openOutput: true
    })
  }

  return {
    canCompileActiveSource,
    canDecompileActiveBinary,
    canGenerateActiveHeader,
    runTheclRequest,
    compileActiveSource,
    decompileActiveBinary,
    generateHeaderForActiveSource
  }
}
