import { findEclDocumentDefinition } from './document-symbols'

export function createEclDefinitionProvider() {
  return {
    provideDefinition(model, position) {
      const wordInfo = model.getWordAtPosition(position)
      if (!wordInfo?.word) return null

      const definition = findEclDocumentDefinition(model, wordInfo.word)
      if (!definition) return null

      return {
        uri: model.uri,
        range: {
          startLineNumber: definition.line,
          endLineNumber: definition.line,
          startColumn: definition.column,
          endColumn: definition.column + definition.name.length
        }
      }
    }
  }
}
