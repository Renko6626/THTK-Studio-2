import type { Component } from 'vue'
import TheclBuildForm from '../../components/Dialogs/forms/TheclBuildForm.vue'
import { createTheclRequest, executeThecl, publishTheclResult } from './thecl'
import { createDefaultMsgPayload } from './msgMetadata'
import type { EditorTab } from '../../stores/editor'
import MsgBuildForm from '../../components/Dialogs/forms/MsgBuildForm.vue'
import {
  createDefaultTheclPayload,
  THECL_MODE_LABELS,
  THECL_MODE_OPTIONS,
  type SelectOption
} from './theclMetadata'
import type {
  BuildDialogPayload,
  EclResult,
  TheclBuildPayload,
  TheclMode,
  TheclRequest,
  ToolchainId
} from '../../types'

/**
 * `descriptor.execute` 拿到的上下文。
 * 目前只需要 useTheclActions 的 runTheclRequest；其余工具接构建对话框时再扩。
 */
export interface ToolchainExecuteContext {
  runTheclRequest: (
    request: TheclRequest,
    options?: { requireSave?: boolean; openOutput?: boolean; successMessage?: string }
  ) => Promise<EclResult | null>
  /** thmsg 走 runFromPayload，不经过 thecl 的 request 形状 */
  runDecompileMsg?: (path: string, encoding: string | null) => Promise<unknown>
  runCompileMsg?: (tab: EditorTab | null, encoding: string | null) => Promise<unknown>
}

/**
 * 注册表条目。这是新增工具链的契约——加一个工具就在 TOOLCHAIN_REGISTRY 里加一条。
 *
 * 只有 thecl 实现了完整的构建对话框，其余四个是 stub，因此除 id / label /
 * exeName / supportsBuildDialog / defaultPayload 之外全部可选。不要为了让 stub
 * 编过而把这些字段放宽成 any——可选属性已经如实表达了"还没实现"。
 */
export interface ToolchainDescriptor {
  id: ToolchainId
  label: string
  exeName: string
  supportsBuildDialog: boolean
  defaultPayload: () => BuildDialogPayload
  buildDialogTitle?: string
  buildDialogSubtitle?: string
  buildFormComponent?: Component
  modeLabels?: Record<TheclMode, string>
  modeOptions?: SelectOption[]
  createRequest?: (payload: TheclBuildPayload) => TheclRequest
  execute?: (
    context: ToolchainExecuteContext,
    request: TheclRequest,
    payload: TheclBuildPayload
  ) => Promise<EclResult | null>
  executeDirect?: typeof executeThecl
  publishResult?: typeof publishTheclResult
  /**
   * 自带执行逻辑的工具链入口，直接吃载荷。
   *
   * thecl 的 createRequest / execute 是围绕 TheclRequest 设计的，thmsg 的请求
   * 形状完全不同（有 encoding、没有 eclmap 和 -r/-s/-x）。与其把那两个签名
   * 泛化到两边都别扭，不如让不合身的工具走自己的入口——thecl 那条已经跑通的
   * 路一行不动。
   */
  runFromPayload?: (
    context: ToolchainExecuteContext,
    payload: BuildDialogPayload
  ) => Promise<{ success: boolean } | null>
}

function inferTheclSuccessMessage(mode: TheclMode | string): string {
  if (mode === 'compile') return '编译完成'
  if (mode === 'decompile') return '反编译完成'
  if (mode === 'header') return '头文件已生成'
  return '执行完成'
}

/** 尚未接构建对话框的工具链的最小载荷 */
function stubPayload(tool: Exclude<ToolchainId, 'thecl' | 'thmsg'>): () => BuildDialogPayload {
  return () => ({ tool, inputPath: '' })
}

export const TOOLCHAIN_REGISTRY: Record<ToolchainId, ToolchainDescriptor> = {
  thecl: {
    id: 'thecl',
    label: 'Enemy Script Compiler',
    exeName: 'thecl.exe',
    supportsBuildDialog: true,
    buildDialogTitle: '构建配置',
    buildDialogSubtitle: '为当前脚本选择模式、版本和 thecl 选项',
    defaultPayload: createDefaultTheclPayload,
    buildFormComponent: TheclBuildForm,
    modeLabels: THECL_MODE_LABELS,
    modeOptions: THECL_MODE_OPTIONS,
    createRequest(payload) {
      return createTheclRequest({
        mode: payload.mode,
        version: payload.version,
        inputPath: payload.inputPath,
        outputPath: payload.outputPath || null,
        mapPaths: payload.mapPaths || [],
        useShiftJis: payload.useShiftJis,
        rawDump: payload.rawDump,
        simpleCreation: payload.simpleCreation,
        showOffsets: payload.showOffsets
      })
    },
    async execute(context, request, payload) {
      return context.runTheclRequest(request, {
        requireSave: payload.mode !== 'decompile',
        openOutput: payload.mode !== 'compile',
        successMessage: inferTheclSuccessMessage(payload.mode)
      })
    },
    executeDirect: executeThecl,
    publishResult: publishTheclResult
  },
  thmsg: {
    id: 'thmsg',
    label: 'Message Script Tool',
    buildDialogTitle: 'MSG 构建配置',
    buildDialogSubtitle: '选择解包 / 打包、输出路径与游戏文本编码',
    defaultPayload: createDefaultMsgPayload,
    buildFormComponent: MsgBuildForm,
    async runFromPayload(context, payload) {
      if (payload.tool !== 'thmsg') return null
      // 空串表示"跟随项目配置"，后端据此回退；不能传空串过去当编码名
      const encoding = payload.encoding || null
      const outputPath = payload.outputPath.trim() || null
      if (payload.mode === 'decompile') {
        const result = await context.runDecompileMsg?.(payload.inputPath, encoding)
        return { success: Boolean((result as { success?: boolean } | null)?.success) }
      }
      const result = await context.runCompileMsg?.(
        { path: payload.inputPath, outputPath } as unknown as EditorTab,
        encoding
      )
      return { success: Boolean((result as { success?: boolean } | null)?.success) }
    },
    exeName: 'thmsg.exe',
    supportsBuildDialog: true
  },
  thanm: {
    id: 'thanm',
    label: 'Animation Tool',
    exeName: 'thanm.exe',
    supportsBuildDialog: false,
    defaultPayload: stubPayload('thanm')
  },
  thstd: {
    id: 'thstd',
    label: 'Stage Data Tool',
    exeName: 'thstd.exe',
    supportsBuildDialog: false,
    defaultPayload: stubPayload('thstd')
  },
  thdat: {
    id: 'thdat',
    label: 'Archive Tool',
    exeName: 'thdat.exe',
    supportsBuildDialog: false,
    defaultPayload: stubPayload('thdat')
  }
}

export function getToolchainDescriptor(
  tool: string | null | undefined
): ToolchainDescriptor | null {
  if (!tool) return null
  return TOOLCHAIN_REGISTRY[tool as ToolchainId] || null
}

export function getRegisteredToolchains(): ToolchainDescriptor[] {
  return Object.values(TOOLCHAIN_REGISTRY)
}

export function createDefaultBuildPayload(tool: string = 'thecl'): BuildDialogPayload {
  const descriptor = getToolchainDescriptor(tool)
  if (descriptor) return descriptor.defaultPayload()
  // 未知的 tool 名：保持原有的宽松行为，回落成最小载荷。
  // 断言是必要的——tool 不在 ToolchainId 里，但运行时确实可能是任意字符串。
  return { tool, inputPath: '' } as BuildDialogPayload
}
