import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import { useWorkbenchPanelsStore } from '../stores/workbenchPanels'
import type { Diagnostic } from '../types'

/** 发布结果时用的工具标识；'ai' 不是 thtk 工具，是 AI 辅助包生成 */
export type ResultTool = 'thecl' | 'thmsg' | 'thstd' | 'thdat' | 'ai'
export type ResultOperation =
  | 'compile'
  | 'decompile'
  | 'header'
  | 'extract'
  | 'pack'
  | 'generate'
  /** 导出不含 IDE 方言的原始 .dmsg / .dstd，供命令行使用 */
  | 'export-raw'

export interface PublishToolchainResultArgs {
  tool: ResultTool
  operation: ResultOperation
  /** 操作的源路径，决定 ownerKey（同一文件重复操作会覆盖旧卡片） */
  inputPath?: string | null
  outputPath?: string | null
  success: boolean
  /** stderr / info 文本 */
  message?: string
  /** 标题附加内容，如 "(N 个文件)" */
  extra?: string
  diagnostics?: Diagnostic[]
}

// 单一来源:(tool, operation) → 卡片标题动词部分
const ACTION_LABELS: Record<ResultTool, Partial<Record<ResultOperation, string>>> = {
  thecl: { compile: '编译 .ecl', decompile: '反编译 .ecl', header: '生成 ECL 头文件' },
  thmsg: { compile: '打包 .msg', decompile: '解包 .msg', 'export-raw': '导出原始 .dmsg' },
  thstd: { compile: '编译 .std', decompile: '反编译 .std', 'export-raw': '导出原始 .dstd' },
  thdat: { extract: '解包 .dat', pack: '打包 .dat' },
  ai:    { generate: '生成 AI 辅助包' }
}

const SCRIPT_KIND: Record<ResultTool, string> = {
  thecl: 'ecl',
  thmsg: 'msg',
  thstd: 'std',
  thdat: 'dat',
  ai:    'ecl'
}

export function usePublishToolchainResult() {
  const reports = useWorkbenchReportsStore()
  const panels  = useWorkbenchPanelsStore()

  /**
   * 发布工具链操作结果到输出面板,使用统一的标题格式。
   * 卡片 ownerKey = `${tool}:${operation}:${inputPath || ''}`,
   * 同一文件的重复操作会覆盖旧卡片,与 publishToolResult 的语义一致。
   */
  function publishToolchainResult({
    tool,
    operation,
    inputPath,
    outputPath,
    success,
    message,
    extra,
    diagnostics
  }: PublishToolchainResultArgs) {
    const action = ACTION_LABELS[tool]?.[operation] || `${tool} ${operation}`
    const suffix = extra ? ` ${extra}` : ''
    const title = `${action} ${success ? '完成' : '失败'}${suffix}`

    reports.publishToolResult({
      ownerKey: `${tool}:${operation}:${inputPath || ''}`,
      source: 'toolchain',
      operation,
      scriptKind: SCRIPT_KIND[tool] || 'text',
      title,
      path: outputPath || inputPath || null,
      success,
      message: message || '',
      diagnostics: diagnostics || []
    })

    panels.showBottomPanel('output')
  }

  return { publishToolchainResult }
}
