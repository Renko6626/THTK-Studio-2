import { runTheclOperation } from '../../api'

function getExtension(path) {
  return path?.split('.').pop()?.toLowerCase() || ''
}

export function inferScriptKind(path) {
  const extension = getExtension(path)
  if (['decl', 'ecl', 'eclmap', 'h'].includes(extension)) {
    return 'ecl'
  }
  return 'text'
}

export function createTheclOwnerKey(mode, inputPath) {
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
}) {
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

export async function executeThecl(request) {
  return runTheclOperation(request)
}

export function publishTheclResult(reportsStore, request, result) {
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
