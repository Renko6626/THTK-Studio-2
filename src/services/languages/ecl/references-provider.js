import { findEclDocumentReferences } from './document-symbols'

export function createEclReferencesProvider() {
  return {
    provideReferences(model, position) {
      const wordInfo = model.getWordAtPosition(position)
      if (!wordInfo?.word) return []

      const references = findEclDocumentReferences(model, wordInfo.word)
      return references.map((reference) => ({
        uri: model.uri,
        range: {
          startLineNumber: reference.line,
          endLineNumber: reference.line,
          startColumn: reference.column,
          endColumn: reference.column + reference.name.length
        }
      }))
    }
  }
}
