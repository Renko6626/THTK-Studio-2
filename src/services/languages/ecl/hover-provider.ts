import type * as monaco from 'monaco-editor'
import { normalizeEclSemanticData } from './dynamic-vocabulary'
import { buildInstructionDocumentation } from './instruction-display'
import type { SemanticDataGetter } from './semantic-state'
import type { EclInstructionSpec, NormalizedEclSemanticData } from '../../../types'

function findInstruction(
  semanticData: NormalizedEclSemanticData,
  word: string | null | undefined
): EclInstructionSpec | null {
  if (!word) return null
  return semanticData.instructions.find((item) => item.name === word) || null
}

export function createEclHoverProvider(
  getSemanticData: SemanticDataGetter
): monaco.languages.HoverProvider {
  return {
    provideHover(model, position) {
      const wordInfo = model.getWordAtPosition(position)
      if (!wordInfo?.word) return null

      const semanticData = normalizeEclSemanticData(getSemanticData(model))
      const instruction = findInstruction(semanticData, wordInfo.word)
      if (!instruction) return null

      return {
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: wordInfo.startColumn,
          endColumn: wordInfo.endColumn
        },
        contents: [buildInstructionDocumentation(instruction)]
      }
    }
  }
}
