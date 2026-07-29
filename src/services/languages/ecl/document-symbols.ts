/** 文档符号的种类 */
export type EclSymbolKind = 'subroutine' | 'global' | 'label'

export interface EclSymbolEntry {
  kind: EclSymbolKind
  name: string
  /** 1-based，与 Monaco 一致 */
  line: number
  /** 1-based */
  column: number
  detail: string
}

/** 引用条目不带 detail */
export type EclReferenceEntry = Omit<EclSymbolEntry, 'detail'>

export interface EclSymbolEntries {
  subroutines: EclSymbolEntry[]
  globals: EclSymbolEntry[]
  labels: EclSymbolEntry[]
}

export interface EclSymbolNames {
  subroutines: string[]
  globals: string[]
  labels: string[]
}

/** 只用到 getValue 的 Monaco model 子集 */
interface TextModelLike {
  getValue: () => string
}

function uniqueByName(entries: EclSymbolEntry[]): EclSymbolEntry[] {
  const seen = new Set<string>()
  return entries.filter((entry) => {
    const key = `${entry.kind}:${entry.name}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

function createSymbolEntry(
  kind: EclSymbolKind,
  name: string,
  line: number,
  column: number,
  detail = ''
): EclSymbolEntry {
  return {
    kind,
    name,
    line,
    column,
    detail
  }
}

function createReferenceEntry(
  kind: EclSymbolKind,
  name: string,
  line: number,
  column: number
): EclReferenceEntry {
  return {
    kind,
    name,
    line,
    column
  }
}

export function collectEclDocumentSymbolEntriesFromText(text: string): EclSymbolEntries {
  const entries = collectAllEclDocumentSymbolEntriesFromText(text)
  return {
    subroutines: uniqueByName(entries.subroutines),
    globals: uniqueByName(entries.globals),
    labels: uniqueByName(entries.labels)
  }
}

export function collectAllEclDocumentSymbolEntriesFromText(text: string): EclSymbolEntries {
  const subroutines: EclSymbolEntry[] = []
  const globals: EclSymbolEntry[] = []
  const labels: EclSymbolEntry[] = []
  const lines = String(text || '').split(/\r?\n/)

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]
    const lineNumber = index + 1

    const subroutineMatch = line.match(/^\s*(?:void|int|float|var)\s+([A-Za-z_]\w*)\s*\(([^)]*)\)/)
    if (subroutineMatch) {
      subroutines.push(createSymbolEntry(
        'subroutine',
        subroutineMatch[1],
        lineNumber,
        line.indexOf(subroutineMatch[1]) + 1,
        subroutineMatch[2]?.trim() || ''
      ))
    }

    const globalMatch = line.match(/^\s*global\s+([A-Za-z_]\w*)\b/)
    if (globalMatch) {
      globals.push(createSymbolEntry(
        'global',
        globalMatch[1],
        lineNumber,
        line.indexOf(globalMatch[1]) + 1
      ))
    }

    const labelMatch = line.match(/^\s*([A-Za-z_]\w*):/)
    if (labelMatch) {
      labels.push(createSymbolEntry(
        'label',
        labelMatch[1],
        lineNumber,
        line.indexOf(labelMatch[1]) + 1
      ))
    }
  }

  return { subroutines, globals, labels }
}

export function collectEclDocumentSymbolsFromText(text: string): EclSymbolNames {
  const entries = collectEclDocumentSymbolEntriesFromText(text)
  return {
    subroutines: entries.subroutines.map((entry) => entry.name),
    globals: entries.globals.map((entry) => entry.name),
    labels: entries.labels.map((entry) => entry.name)
  }
}

export function collectEclDocumentSymbols(model: TextModelLike): EclSymbolNames {
  return collectEclDocumentSymbolsFromText(model.getValue())
}

export function collectEclDocumentSymbolEntries(model: TextModelLike): EclSymbolEntries {
  return collectEclDocumentSymbolEntriesFromText(model.getValue())
}

export function findEclDocumentDefinitionFromText(
  text: string,
  word: string
): EclSymbolEntry | null {
  const entries = collectEclDocumentSymbolEntriesFromText(text)
  const allEntries = [...entries.subroutines, ...entries.globals, ...entries.labels]
  return allEntries.find((entry) => entry.name === word) || null
}

export function findEclDocumentDefinition(model: TextModelLike, word: string): EclSymbolEntry | null {
  return findEclDocumentDefinitionFromText(model.getValue(), word)
}

function collectSubroutineReferencesFromText(text: string, targetName: string): EclReferenceEntry[] {
  const references: EclReferenceEntry[] = []
  const escapedName = targetName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const lines = String(text || '').split(/\r?\n/)
  const definitionRegex = new RegExp(`^\\s*(?:void|int|float|var)\\s+(${escapedName})\\s*\\(`)
  const callRegex = new RegExp(`@(${escapedName})\\s*\\(`, 'g')

  lines.forEach((line, index) => {
    const lineNumber = index + 1
    const definitionMatch = line.match(definitionRegex)
    if (definitionMatch) {
      references.push(createReferenceEntry(
        'subroutine',
        targetName,
        lineNumber,
        line.indexOf(targetName) + 1
      ))
    }

    let callMatch = callRegex.exec(line)
    while (callMatch) {
      references.push(createReferenceEntry(
        'subroutine',
        targetName,
        lineNumber,
        callMatch.index + 2
      ))
      callMatch = callRegex.exec(line)
    }
  })

  return references
}

function collectLabelReferencesFromText(text: string, targetName: string): EclReferenceEntry[] {
  const references: EclReferenceEntry[] = []
  const escapedName = targetName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const lines = String(text || '').split(/\r?\n/)
  const definitionRegex = new RegExp(`^\\s*(${escapedName}):`)
  const gotoRegex = new RegExp(`\\bgoto\\s+(${escapedName})\\s*@`, 'g')

  lines.forEach((line, index) => {
    const lineNumber = index + 1
    const definitionMatch = line.match(definitionRegex)
    if (definitionMatch) {
      references.push(createReferenceEntry(
        'label',
        targetName,
        lineNumber,
        line.indexOf(targetName) + 1
      ))
    }

    let gotoMatch = gotoRegex.exec(line)
    while (gotoMatch) {
      references.push(createReferenceEntry(
        'label',
        targetName,
        lineNumber,
        gotoMatch.index + gotoMatch[0].indexOf(targetName) + 1
      ))
      gotoMatch = gotoRegex.exec(line)
    }
  })

  return references
}

function collectGlobalReferencesFromText(text: string, targetName: string): EclReferenceEntry[] {
  const references: EclReferenceEntry[] = []
  const escapedName = targetName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const lines = String(text || '').split(/\r?\n/)
  const wordRegex = new RegExp(`\\b${escapedName}\\b`, 'g')

  lines.forEach((line, index) => {
    const lineNumber = index + 1
    let match = wordRegex.exec(line)
    while (match) {
      references.push(createReferenceEntry(
        'global',
        targetName,
        lineNumber,
        match.index + 1
      ))
      match = wordRegex.exec(line)
    }
  })

  return references
}

export function findEclDocumentReferencesFromText(text: string, word: string): EclReferenceEntry[] {
  if (!word) return []

  const definition = findEclDocumentDefinitionFromText(text, word)
  if (!definition) return []

  if (definition.kind === 'subroutine') {
    return collectSubroutineReferencesFromText(text, word)
  }

  if (definition.kind === 'label') {
    return collectLabelReferencesFromText(text, word)
  }

  if (definition.kind === 'global') {
    return collectGlobalReferencesFromText(text, word)
  }

  return []
}

export function findEclDocumentReferences(model: TextModelLike, word: string): EclReferenceEntry[] {
  return findEclDocumentReferencesFromText(model.getValue(), word)
}
