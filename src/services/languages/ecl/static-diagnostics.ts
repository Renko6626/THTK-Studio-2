import * as monaco from 'monaco-editor'
import {
  collectAllEclDocumentSymbolEntriesFromText,
  collectEclDocumentSymbolEntriesFromText
} from './document-symbols'
import { eclLanguageId } from './language-config'
import type { EclSymbolEntries, EclSymbolEntry } from './document-symbols'

/**
 * 本文件内部的引用条目：与 document-symbols 的 EclReferenceEntry 不同，
 * 这里直接存 Monaco marker 需要的列区间而不是单个 column。
 */
interface CallSiteRef {
  name: string
  line: number
  startColumn: number
  endColumn: number
}
import type { ProblemInput } from '../../../stores/workbenchReports'

type MarkerData = monaco.editor.IMarkerData

const markerOwner = 'thtk-ecl-static'
const staticProblemOwnerPrefix = 'ecl-static:analysis:'

function createIndex(entries: EclSymbolEntry[]): Map<string, EclSymbolEntry[]> {
  const map = new Map<string, EclSymbolEntry[]>()
  entries.forEach((entry) => {
    const group = map.get(entry.name) || []
    group.push(entry)
    map.set(entry.name, group)
  })
  return map
}

function createMarker(
  lineNumber: number,
  startColumn: number,
  endColumn: number,
  message: string,
  severity: monaco.MarkerSeverity
): MarkerData {
  return {
    startLineNumber: lineNumber,
    endLineNumber: lineNumber,
    startColumn,
    endColumn,
    message,
    severity
  }
}

function collectSubroutineCalls(text: string): CallSiteRef[] {
  const references: CallSiteRef[] = []
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

function collectGotoTargets(text: string): CallSiteRef[] {
  const references: CallSiteRef[] = []
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

function collectDuplicateMarkers(entries: EclSymbolEntry[], kindLabel: string): MarkerData[] {
  const markers: MarkerData[] = []
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

function collectMissingSubroutineMarkers(
  text: string,
  symbolEntries: EclSymbolEntries
): MarkerData[] {
  const markers: MarkerData[] = []
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

function collectMissingLabelMarkers(
  text: string,
  symbolEntries: EclSymbolEntries
): MarkerData[] {
  const markers: MarkerData[] = []
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

export function collectEclStaticMarkersFromText(text: string): MarkerData[] {
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

export function createEclStaticProblemEntries(
  path: string | null,
  text: string
): ProblemInput[] {
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

export function getEclStaticProblemOwnerKey(path: string | null | undefined): string {
  return `${staticProblemOwnerPrefix}${path || 'workspace'}`
}

export function updateEclStaticDiagnostics(model: monaco.editor.ITextModel | null): void {
  if (!model) return
  if (model.getLanguageId() !== eclLanguageId) {
    monaco.editor.setModelMarkers(model, markerOwner, [])
    return
  }
  const markers = collectEclStaticMarkersFromText(model.getValue())
  monaco.editor.setModelMarkers(model, markerOwner, markers)
}

export function clearEclStaticDiagnostics(model: monaco.editor.ITextModel | null): void {
  if (!model) return
  monaco.editor.setModelMarkers(model, markerOwner, [])
}

function normalizeMarkerSeverity(severity: monaco.MarkerSeverity): string {
  if (severity === monaco.MarkerSeverity.Error) return 'error'
  if (severity === monaco.MarkerSeverity.Warning) return 'warning'
  if (severity === monaco.MarkerSeverity.Info) return 'info'
  return 'hint'
}
