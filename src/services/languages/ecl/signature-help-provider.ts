import { normalizeEclSemanticData } from './dynamic-vocabulary'
import {
  buildInstructionDocumentation,
  buildInstructionParameterLabel,
  buildInstructionSignature
} from './instruction-display'
import type * as monaco from 'monaco-editor'
import type { SemanticDataGetter } from './semantic-state'
import type { EclInstructionSpec, NormalizedEclSemanticData } from '../../../types'

/** 光标所在的调用上下文：函数名 + 当前是第几个参数 */
interface CallContext {
  functionName: string
  activeParameter: number
}


function findInstruction(
  semanticData: NormalizedEclSemanticData,
  name: string | null | undefined
): EclInstructionSpec | null {
  if (!name) return null
  return semanticData.instructions.find((item) => item.name === name) || null
}

function getCallContext(linePrefix: string): CallContext | null {
  let depth = 0

  for (let index = linePrefix.length - 1; index >= 0; index -= 1) {
    const char = linePrefix[index]

    if (char === ')') {
      depth += 1
      continue
    }

    if (char === '(') {
      if (depth > 0) {
        depth -= 1
        continue
      }

      const beforeParen = linePrefix.slice(0, index)
      const match = beforeParen.match(/([A-Za-z_][A-Za-z0-9_]*)\s*$/)
      if (!match) return null

      const argsSlice = linePrefix.slice(index + 1)
      let activeParameter = 0
      let nestedDepth = 0

      for (const currentChar of argsSlice) {
        if (currentChar === '(') {
          nestedDepth += 1
        } else if (currentChar === ')' && nestedDepth > 0) {
          nestedDepth -= 1
        } else if (currentChar === ',' && nestedDepth === 0) {
          activeParameter += 1
        }
      }

      return {
        functionName: match[1],
        activeParameter
      }
    }
  }

  return null
}

export function createEclSignatureHelpProvider(
  getSemanticData: SemanticDataGetter
): monaco.languages.SignatureHelpProvider {
  return {
    signatureHelpTriggerCharacters: ['(', ','],
    signatureHelpRetriggerCharacters: [','],
    provideSignatureHelp(model, position) {
      const linePrefix = model.getLineContent(position.lineNumber).slice(0, position.column - 1)
      const callContext = getCallContext(linePrefix)
      if (!callContext) return null

      const semanticData = normalizeEclSemanticData(getSemanticData(model))
      const instruction = findInstruction(semanticData, callContext.functionName)
      if (!instruction) return null

      const params = Array.isArray(instruction.params) ? instruction.params : []
      const parameterLabels = params.map((param, index) => ({
        label: buildInstructionParameterLabel(param, index),
        documentation: {
          value: `参数 ${index + 1}${param?.type ? ` · ${param.type}` : ''}`
        }
      }))

      return {
        value: {
          activeSignature: 0,
          activeParameter: Math.min(callContext.activeParameter, Math.max(0, parameterLabels.length - 1)),
          signatures: [
            {
              label: buildInstructionSignature(instruction),
              documentation: buildInstructionDocumentation(instruction),
              parameters: parameterLabels
            }
          ]
        },
        dispose: () => {}
      }
    }
  }
}
