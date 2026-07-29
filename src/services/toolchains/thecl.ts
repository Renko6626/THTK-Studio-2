import { runTheclOperation } from '../../api'
import type { EclResult, TheclMode, TheclRequest } from '../../types'
import type { useWorkbenchReportsStore } from '../../stores/workbenchReports'

type ReportsStore = ReturnType<typeof useWorkbenchReportsStore>

/** createTheclRequest 的入参：除 mode / inputPath 外都有默认值 */
export interface TheclRequestInput {
  mode: TheclMode
  inputPath: string
  version?: string
  outputPath?: string | null
  mapPaths?: string[]
  useShiftJis?: boolean
  rawDump?: boolean
  simpleCreation?: boolean
  showOffsets?: boolean
}

function getExtension(path: string | null | undefined): string {
  return path?.split('.').pop()?.toLowerCase() || ''
}

export function inferScriptKind(path: string | null | undefined): string {
  const extension = getExtension(path)
  if (['decl', 'ecl', 'eclmap', 'h'].includes(extension)) {
    return 'ecl'
  }
  return 'text'
}

export function createTheclOwnerKey(mode: string, inputPath: string | null | undefined): string {
  return `thecl:${mode}:${inputPath || 'workspace'}`
}

export function createTheclRequest({
  mode,
  version = '',
  inputPath,
  outputPath = null,
  mapPaths = [],
  useShiftJis = true,
  rawDump = false,
  simpleCreation = false,
  showOffsets = false
}: TheclRequestInput): TheclRequest {
  return {
    mode,
    version,
    inputPath,
    outputPath,
    mapPaths,
    useShiftJis,
    rawDump,
    simpleCreation,
    showOffsets
  }
}

export async function executeThecl(request: TheclRequest): Promise<EclResult> {
  return runTheclOperation(request)
}

export function publishTheclResult(
  reportsStore: ReportsStore,
  request: TheclRequest | null | undefined,
  result: EclResult | null | undefined
): void {
  const mode = request?.mode || result?.mode || 'compile'
  const inputPath = result?.inputPath || request?.inputPath || null

  reportsStore.publishToolResult({
    ownerKey: createTheclOwnerKey(mode, inputPath),
    source: 'thecl',
    operation: mode,
    scriptKind: inferScriptKind(inputPath),
    title: result?.outputPath || inputPath || '',
    path: inputPath,
    success: result?.success,
    message: result?.message || '',
    diagnostics: result?.diagnostics || []
  })
}
