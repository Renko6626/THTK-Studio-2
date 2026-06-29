import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import { useWorkbenchPanelsStore } from '../stores/workbenchPanels'

// 单一来源:(tool, operation) → 卡片标题动词部分
const ACTION_LABELS = {
  thecl: { compile: '编译 .ecl', decompile: '反编译 .ecl', header: '生成 ECL 头文件' },
  thmsg: { compile: '编译 .msg', decompile: '反编译 .msg' },
  thstd: { compile: '编译 .std', decompile: '反编译 .std' },
  thdat: { extract: '解包 .dat', pack: '打包 .dat' },
  ai:    { generate: '生成 AI 辅助包' }
}

const SCRIPT_KIND = {
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
   *
   * @param {object} args
   * @param {'thecl'|'thmsg'|'thstd'|'thdat'|'ai'} args.tool
   * @param {'compile'|'decompile'|'header'|'extract'|'pack'|'generate'} args.operation
   * @param {string} [args.inputPath] 操作的源路径(决定 ownerKey)
   * @param {string} [args.outputPath] 产物路径(可选)
   * @param {boolean} args.success
   * @param {string} [args.message] stderr/info 文本
   * @param {string} [args.extra] 标题附加内容(如 "(N 个文件)")
   * @param {Array}  [args.diagnostics] 结构化诊断(默认空)
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
  }) {
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
