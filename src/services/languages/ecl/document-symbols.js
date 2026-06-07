function uniqueByName(entries) {
  const seen = new Set()
  return entries.filter((entry) => {
    const key = `${entry.kind}:${entry.name}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

function createSymbolEntry(kind, name, line, column, detail = '') {
  return {
    kind,
    name,
    line,
    column,
    detail
  }
}

function createReferenceEntry(kind, name, line, column) {
  return {
    kind,
    name,
    line,
    column
  }
}

export function collectEclDocumentSymbolEntriesFromText(text) {
  const entries = collectAllEclDocumentSymbolEntriesFromText(text)
  return {
    subroutines: uniqueByName(entries.subroutines),
    globals: uniqueByName(entries.globals),
    labels: uniqueByName(entries.labels)
  }
}

export function collectAllEclDocumentSymbolEntriesFromText(text) {
  const subroutines = []
  const globals = []
  const labels = []
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

export function collectEclDocumentSymbolsFromText(text) {
  const entries = collectEclDocumentSymbolEntriesFromText(text)
  return {
    subroutines: entries.subroutines.map((entry) => entry.name),
    globals: entries.globals.map((entry) => entry.name),
    labels: entries.labels.map((entry) => entry.name)
  }
}

export function collectEclDocumentSymbols(model) {
  return collectEclDocumentSymbolsFromText(model.getValue())
}

export function collectEclDocumentSymbolEntries(model) {
  return collectEclDocumentSymbolEntriesFromText(model.getValue())
}

export function findEclDocumentDefinitionFromText(text, word) {
  const entries = collectEclDocumentSymbolEntriesFromText(text)
  const allEntries = [...entries.subroutines, ...entries.globals, ...entries.labels]
  return allEntries.find((entry) => entry.name === word) || null
}

export function findEclDocumentDefinition(model, word) {
  return findEclDocumentDefinitionFromText(model.getValue(), word)
}

function collectSubroutineReferencesFromText(text, targetName) {
  const references = []
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

function collectLabelReferencesFromText(text, targetName) {
  const references = []
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

function collectGlobalReferencesFromText(text, targetName) {
  const references = []
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

export function findEclDocumentReferencesFromText(text, word) {
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

export function findEclDocumentReferences(model, word) {
  return findEclDocumentReferencesFromText(model.getValue(), word)
}
