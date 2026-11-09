import * as monaco from 'monaco-editor'
import { getTruthSourceDefinitions, analyzeTruthSource } from '../../../api'
import { dialectLanguageConfiguration } from '../dialect-config'
import type { TruthScriptKind, TruthInstruction } from '../../../types'

export const msgLanguageId = 'thtk-truth-msg'
export const endingLanguageId = 'thtk-truth-ending'
export const stdLanguageId = 'thtk-truth-std'
const languageKinds: Record<string, TruthScriptKind> = {
  [msgLanguageId]: 'msg', [endingLanguageId]: 'ending', [stdLanguageId]: 'std'
}
const registered = new Set<string>()
const docs = (i: TruthInstruction) => `${i.name || `ins_${i.opcode}`}(${i.params?.map((p, n) => `${p.enumName || p.type} arg${n + 1}`).join(', ') ?? '?'})${i.description ? `\n\n${i.description}` : ''}`
async function definitions(model: monaco.editor.ITextModel) {
  return getTruthSourceDefinitions(model.uri.fsPath, languageKinds[model.getLanguageId()], model.getValue())
}

export function ensureTruthLanguagesRegistered(): void {
  for (const [id, extensions] of [[msgLanguageId, ['.msgspec']], [endingLanguageId, ['.endspec']], [stdLanguageId, ['.stdspec']]] as const) {
    if (registered.has(id)) continue
    monaco.languages.register({ id, extensions: [...extensions] })
    monaco.languages.setLanguageConfiguration(id, dialectLanguageConfiguration)
    monaco.languages.setMonarchTokensProvider(id, {
      defaultToken: '',
      keywords: ['meta', 'script', 'const', 'int', 'float', 'string', 'loop', 'while', 'times', 'if', 'else', 'goto', 'true', 'false'],
      tokenizer: {
        root: [
          [/\/\*/, 'comment', '@comment'], [/\/\/.*$/, 'comment'],
          [/^\s*#pragma\b.*$/, 'keyword'], [/[+-]?\d+\s*:/, 'number'],
          [/[a-zA-Z_]\w*(?=\s*:)/, 'type.identifier'],
          [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
          [/"/, 'string', '@string'], [/0x[0-9a-fA-F]+|\d*\.\d+(?:f)?|\d+f?/, 'number'],
          [/[{}()[\]]/, '@brackets'], [/[,:;]/, 'delimiter'], [/[+\-*/%=<>!&|]+/, 'operator']
        ],
        comment: [[/[^/*]+/, 'comment'], [/\*\//, 'comment', '@pop'], [/[/*]/, 'comment']],
        string: [[/[^"\\]+/, 'string'], [/\\./, 'string.escape'], [/"/, 'string', '@pop']]
      }
    })
    monaco.languages.registerCompletionItemProvider(id, {
      async provideCompletionItems(model, position) {
        try {
          const data = await definitions(model)
          const word = model.getWordUntilPosition(position)
          const range = new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn)
          const suggestions: monaco.languages.CompletionItem[] = data.instructions.flatMap(i => i.aliases.map(name => ({ label: name, insertText: name, kind: monaco.languages.CompletionItemKind.Function, detail: docs(i), range })))
          for (const [enumName, values] of Object.entries(data.enums)) for (const [name, value] of Object.entries(values)) suggestions.push({ label: name, insertText: name, kind: monaco.languages.CompletionItemKind.EnumMember, detail: `${enumName} = ${value}`, range })
          return { suggestions }
        } catch { return { suggestions: [] } }
      }
    })
    monaco.languages.registerHoverProvider(id, {
      async provideHover(model, position) {
        const word = model.getWordAtPosition(position)?.word
        if (!word) return null
        try {
          const data = await definitions(model)
          const found = data.instructions.find(i => i.aliases.includes(word) || `ins_${i.opcode}` === word)
          if (found) return { contents: [{ value: docs(found) }] }
          for (const [name, values] of Object.entries(data.enums)) if (word in values) return { contents: [{ value: `${name}: ${word} = ${values[word]}` }] }
        } catch { /* map load errors are reported by the build/explicit query */ }
        return null
      }
    })
    monaco.languages.registerSignatureHelpProvider(id, {
      signatureHelpTriggerCharacters: ['(', ','],
      async provideSignatureHelp(model, position) {
        const prefix = model.getValue().slice(0, model.getOffsetAt(position))
        const call = /([A-Za-z_]\w*)\s*\(([^()]*)$/.exec(prefix)
        if (!call) return null
        try {
          const found = (await definitions(model)).instructions.find(i => i.aliases.includes(call[1]) || `ins_${i.opcode}` === call[1])
          if (!found?.params) return null
          return { value: { activeSignature: 0, activeParameter: Math.min(call[2].split(',').length - 1, Math.max(0, found.params.length - 1)), signatures: [{ label: docs(found).split('\n')[0], documentation: found.description || '', parameters: found.params.map((p, n) => ({ label: `${p.enumName || p.type} arg${n + 1}` })) }] }, dispose() {} }
        } catch { return null }
      }
    })
    monaco.languages.registerDefinitionProvider(id, {
      async provideDefinition(model, position) {
        try {
          const analysis = await analyzeTruthSource(model.getValue())
          const jump = analysis.jumps.find(j => j.line === position.lineNumber && position.column >= j.column && position.column <= j.endColumn)
          return jump?.targetLine ? { uri: model.uri, range: new monaco.Range(jump.targetLine, 1, jump.targetLine, 1) } : null
        } catch { return null }
      }
    })
    registered.add(id)
  }
}
