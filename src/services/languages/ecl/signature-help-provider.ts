import type * as monaco from 'monaco-editor'
import { getCallContext } from '../callContext'
import { normalizeEclSemanticData } from './dynamic-vocabulary'
import { findDisplayInstruction, formatParameterDetails } from './instruction-display'
import type { SemanticDataGetter } from './semantic-state'

export function createEclSignatureHelpProvider(
  getSemanticData: SemanticDataGetter
): monaco.languages.SignatureHelpProvider {
  return {
    signatureHelpTriggerCharacters: ['(', ','],
    signatureHelpRetriggerCharacters: [','],
    provideSignatureHelp(model, position) {
      const prefix = model.getValueInRange({
        startLineNumber: 1, startColumn: 1, endLineNumber: position.lineNumber, endColumn: position.column
      })
      const call = getCallContext(prefix)
      if (!call) return null

      const data = normalizeEclSemanticData(getSemanticData(model))
      const instruction = findDisplayInstruction(data, call.name)
      if (!instruction) return null
      const parameterLabels = instruction.params.map((param, index) => ({
        label: param.label,
        documentation: param.contract ? formatParameterDetails(param.contract) : `参数 ${index + 1}`
      }))

      return {
        value: {
          activeSignature: 0,
          activeParameter: Math.min(call.parameter, Math.max(0, parameterLabels.length - 1)),
          signatures: [
            {
              label: instruction.label,
              documentation: [instruction.fromReference ? '参考签名' : '', instruction.reference?.description].filter(Boolean).join(' · '),
              parameters: parameterLabels
            }
          ]
        },
        dispose: () => {}
      }
    }
  }
}
