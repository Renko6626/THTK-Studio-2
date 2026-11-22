import type * as monaco from 'monaco-editor'
import { normalizeEclSemanticData } from './dynamic-vocabulary'
import { buildInstructionDocumentation, findDisplayInstruction } from './instruction-display'
import type { SemanticDataGetter } from './semantic-state'

export function createEclHoverProvider(
  getSemanticData: SemanticDataGetter
): monaco.languages.HoverProvider {
  return {
    provideHover(model, position) {
      const wordInfo = model.getWordAtPosition(position)
      if (!wordInfo?.word) return null

      const data = normalizeEclSemanticData(getSemanticData(model))
      const display = findDisplayInstruction(data, wordInfo.word)
      if (!display) return null

      const contents = [buildInstructionDocumentation(display)]
      const row = display.reference
      if (row && data.reference) {
        const signature = row.signature === null ? '未声明' : row.signature || '无参数'
        const names = [...new Set([row.zero318Name, row.priw8Name].filter(Boolean))]
        contents.push({
          value: [
            `**TH${data.reference.version} 指令参考 · opcode ${row.opcode} · ${row.reliability}**`,
            names.map(name => `\`${name}\``).join(' / '),
            row.zero318SourceName && !names.includes(row.zero318SourceName)
              ? `zero318 的 TH${row.zero318SourceGame} 原表名称 \`${row.zero318SourceName}\`` : '',
            row.assessment !== row.description ? row.assessment : '',
            display.fromReference ? '' : `参考签名 \`${signature}\``,
            row.stockCount === null ? '' : `原版 ECL 出现 ${row.stockCount} 次`,
            row.stockExample ? `原版样例 \`${row.stockExample}\`` : ''
          ].filter(Boolean).join('\n\n')
        })
      }

      return {
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: wordInfo.startColumn,
          endColumn: wordInfo.endColumn
        },
        contents
      }
    }
  }
}
