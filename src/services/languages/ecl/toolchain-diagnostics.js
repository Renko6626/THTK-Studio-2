import * as monaco from 'monaco-editor'

const markerOwner = 'thtk-ecl-toolchain'

function toMarkerSeverity(severity) {
  if (severity === 'error') return monaco.MarkerSeverity.Error
  if (severity === 'warning') return monaco.MarkerSeverity.Warning
  if (severity === 'info') return monaco.MarkerSeverity.Info
  return monaco.MarkerSeverity.Hint
}

function createMarker(problem, model) {
  const line = Math.max(1, Number(problem.line || 1))
  const column = Math.max(1, Number(problem.column || 1))

  let endColumn = column + 1

  if (model && line <= model.getLineCount()) {
    const word = model.getWordAtPosition({ lineNumber: line, column })
    if (word) {
      endColumn = word.endColumn
    } else {
      // 没有找到单词，延伸到行尾有内容处
      const lineContent = model.getLineContent(line)
      const trimmedEnd = lineContent.trimEnd().length + 1
      endColumn = Math.max(column + 1, trimmedEnd)
    }
  }

  return {
    startLineNumber: line,
    endLineNumber: line,
    startColumn: column,
    endColumn,
    message: problem.message || 'Unknown issue',
    severity: toMarkerSeverity(problem.severity)
  }
}

function normalizePath(path) {
  return String(path || '').replace(/\//g, '\\').toLowerCase()
}

export function syncToolchainDiagnosticsToModels(models, problemEntries) {
  const markersByPath = new Map()

  for (const problem of problemEntries || []) {
    if (problem?.source !== 'thecl' || !problem?.path) continue
    const key = normalizePath(problem.path)
    const group = markersByPath.get(key) || []
    group.push(problem)
    markersByPath.set(key, group)
  }

  models.forEach((model) => {
    const modelKey = normalizePath(model?.uri?.fsPath)
    const problems = markersByPath.get(modelKey)
    const markers = problems
      ? problems.map((problem) => createMarker(problem, model))
      : []
    monaco.editor.setModelMarkers(model, markerOwner, markers)
  })
}

export function clearToolchainDiagnostics(model) {
  if (!model) return
  monaco.editor.setModelMarkers(model, markerOwner, [])
}
