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
import type {
  EclResult,
  ProjectConfig,
  ProjectConfigStatus,
  TheclMode,
  TheclRequest
} from '../types'

export interface RunTheclOptions {
  /** 执行前若当前标签是脏的，先保存；保存失败则中止 */
  requireSave?: boolean
  successMessage?: string
  /** 成功后打开产物文件 */
  openOutput?: boolean
}

interface RunTheclForActiveOptions extends RunTheclOptions {
  mode: TheclMode
}

export interface ProjectDefaultsResult {
  request: TheclRequest
  /** 项目设置未能生效时给用户的说明；正常情况为 null */
  warning: string | null
}

/**
 * 从项目配置填充 thecl 请求的默认值。
 *
 * 配置损坏（`invalid`）时前端会把 `projectConfig` 置为 null，此前这里直接
 * `return request` —— ECL 于是**静默**丢掉 mapPaths，在没有 eclmap 的情况下
 * 编译，报错完全指不到真正的原因。而 msg/std/dat 走 Rust 的尽力而为加载器，
 * 照常拿到 gameVersion 与 thtkDir，两条路径行为并不一致。
 *
 * 这里不改变"损坏时不采用项目设置"的决定——拿已知格式错误的数据去编译更糟——
 * 但把它从静默变成显式。
 */
export function applyProjectDefaults(
  request: TheclRequest,
  projectConfig: ProjectConfig | null,
  status: ProjectConfigStatus
): ProjectDefaultsResult {
  if (!projectConfig) {
    const warning =
      status === 'invalid'
        ? '项目配置文件有误，本次调用未使用项目设置——eclmap、目标版本与编码都回退到了默认值。请先在「项目设置」里修复。'
        : null
    return { request, warning }
  }

  return {
    request: {
      ...request,
      version: request.version || projectConfig.gameVersion || '',
      mapPaths: request.mapPaths?.length ? request.mapPaths : (projectConfig.mapPaths || []),
      // 编码由项目配置直接决定，不是"填空"。
      //
      // 原先写的是 `request.useShiftJis ?? (...)`，而 createTheclRequest 把
      // useShiftJis 默认成 true 且类型是非可选 boolean——`??` 永远不会回落，
      // 项目里写 encoding: "utf-8" 对这条路径完全无效，照样传 -j 给 thecl。
      //
      // 本函数只服务于快捷菜单路径（高级构建对话框直接调 runTheclRequest），
      // 那里没有用户的显式勾选可言，项目配置就是唯一权威。
      useShiftJis: projectConfig.encoding === 'shift-jis'
    },
    warning: null
  }
}

function getExtension(path: string | null | undefined): string {
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

  async function runTheclRequest(
    request: TheclRequest | null | undefined,
    { requireSave = false, successMessage, openOutput = false }: RunTheclOptions = {}
  ): Promise<EclResult | null> {
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
  }: RunTheclForActiveOptions): Promise<EclResult | null> {
    const activeTab = editorStore.activeTab
    if (!activeTab?.path) {
      message.warning('当前没有可处理的脚本文件')
      return null
    }

    const { request, warning } = applyProjectDefaults(
      createTheclRequest({ mode, inputPath: activeTab.path }),
      projectStore.projectConfig,
      projectStore.projectConfigStatus
    )

    // 配置损坏时项目设置整体没生效，先说清楚再跑——否则用户只会看到一堆
    // "unknown instruction"，完全联想不到是配置文件的问题。
    if (warning) {
      reportsStore.publishToolResult({
        ownerKey: `thecl:project-config:${projectStore.rootPath || 'workspace'}`,
        source: 'thecl',
        operation: mode,
        scriptKind: 'ecl',
        title: '项目配置未生效',
        path: projectStore.projectConfigPath || null,
        success: false,
        message: warning,
        diagnostics: []
      })
      workbenchPanelsStore.showBottomPanel('output')
    }

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
