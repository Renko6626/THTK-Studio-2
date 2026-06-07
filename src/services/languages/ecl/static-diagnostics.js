import * as monaco from 'monaco-editor'
import {
  collectAllEclDocumentSymbolEntriesFromText,
  collectEclDocumentSymbolEntriesFromText
} from './document-symbols'
import { eclLanguageId } from './language-config'

const markerOwner = 'thtk-ecl-static'
const staticProblemOwnerPrefix = 'ecl-static:analysis:'

function createIndex(entries) {
  const map = new Map()
  entries.forEach((entry) => {
    const group = map.get(entry.name) || []
    group.push(entry)
    map.set(entry.name, group)
  })
  return map
}

function createMarker(lineNumber, startColumn, endColumn, message, severity) {
  return {
    startLineNumber: lineNumber,
    endLineNumber: lineNumber,
    startColumn,
    endColumn,
    message,
    severity
  }
}

function collectSubroutineCalls(text) {
  const references = []
  const lines = String(text || '').split(/\r?\n/)

  lines.forEach((line, index) => {
    const lineNumber = index + 1
    const regex = /@([A-Za-z_]\w*)\s*\(/g
    let match = regex.exec(line)
    while (match) {
      const name = match[1]
      const startColumn = match.index + 2
      references.push({
        name,
        line: lineNumber,
        startColumn,
        endColumn: startColumn + name.length
      })
      match = regex.exec(line)
    }
  })

  return references
}

function collectGotoTargets(text) {
  const references = []
  const lines = String(text || '').split(/\r?\n/)

  lines.forEach((line, index) => {
    const lineNumber = index + 1
    const regex = /\bgoto\s+([A-Za-z_]\w*)\s*@/g
    let match = regex.exec(line)
    while (match) {
      const name = match[1]
      const startColumn = match.index + match[0].indexOf(name) + 1
      references.push({
        name,
        line: lineNumber,
        startColumn,
        endColumn: startColumn + name.length
      })
      match = regex.exec(line)
    }
  })

  return references
}

function collectDuplicateMarkers(entries, kindLabel) {
  const markers = []
  const entryIndex = createIndex(entries)

  entryIndex.forEach((group, name) => {
    if (group.length < 2) return
    group.forEach((entry) => {
      markers.push(createMarker(
        entry.line,
        entry.column,
        entry.column + entry.name.length,
        `重复的${kindLabel}定义：${name}`,
        monaco.MarkerSeverity.Error
      ))
    })
  })

  return markers
}

function collectMissingSubroutineMarkers(text, symbolEntries) {
  const markers = []
  const subroutineIndex = createIndex(symbolEntries.subroutines)

  collectSubroutineCalls(text).forEach((reference) => {
    if (subroutineIndex.has(reference.name)) return
    markers.push(createMarker(
      reference.line,
      reference.startColumn,
      reference.endColumn,
      `未在当前文件中找到子程序定义：${reference.name}`,
      monaco.MarkerSeverity.Warning
    ))
  })

  return markers
}

function collectMissingLabelMarkers(text, symbolEntries) {
  const markers = []
  const labelIndex = createIndex(symbolEntries.labels)

  collectGotoTargets(text).forEach((reference) => {
    if (labelIndex.has(reference.name)) return
    markers.push(createMarker(
      reference.line,
      reference.startColumn,
      reference.endColumn,
      `未定义的 goto 标签：${reference.name}`,
      monaco.MarkerSeverity.Error
    ))
  })

  return markers
}

export function collectEclStaticMarkersFromText(text) {
  const allSymbolEntries = collectAllEclDocumentSymbolEntriesFromText(text)
  const symbolEntries = collectEclDocumentSymbolEntriesFromText(text)

  return [
    ...collectDuplicateMarkers(allSymbolEntries.subroutines, '子程序'),
    ...collectDuplicateMarkers(allSymbolEntries.globals, '全局定义'),
    ...collectDuplicateMarkers(allSymbolEntries.labels, '标签'),
    ...collectMissingSubroutineMarkers(text, symbolEntries),
    ...collectMissingLabelMarkers(text, symbolEntries)
  ]
}

export function createEclStaticProblemEntries(path, text) {
  return collectEclStaticMarkersFromText(text).map((marker) => ({
    source: 'ecl-static',
    operation: 'analysis',
    scriptKind: 'ecl',
    path,
    line: marker.startLineNumber,
    column: marker.startColumn,
    severity: normalizeMarkerSeverity(marker.severity),
    message: marker.message
  }))
}

export function getEclStaticProblemOwnerKey(path) {
  return `${staticProblemOwnerPrefix}${path || 'workspace'}`
}

export function updateEclStaticDiagnostics(model) {
  if (!model) return
  if (model.getLanguageId() !== eclLanguageId) {
    monaco.editor.setModelMarkers(model, markerOwner, [])
    return
  }
  const markers = collectEclStaticMarkersFromText(model.getValue())
  monaco.editor.setModelMarkers(model, markerOwner, markers)
}

export function clearEclStaticDiagnostics(model) {
  if (!model) return
  monaco.editor.setModelMarkers(model, markerOwner, [])
}

function normalizeMarkerSeverity(severity) {
  if (severity === monaco.MarkerSeverity.Error) return 'error'
  if (severity === monaco.MarkerSeverity.Warning) return 'warning'
  if (severity === monaco.MarkerSeverity.Info) return 'info'
  return 'hint'
}
