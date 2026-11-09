import { truthCallContext } from './callContext'
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

export const truthTokens: monaco.languages.IMonarchLanguage = {
      defaultToken: '',
      keywords: ['meta', 'script', 'const', 'int', 'float', 'string', 'loop', 'while', 'times', 'if', 'else', 'goto', 'true', 'false'],
      tokenizer: {
        root: [
          [/\/\*/, 'comment', '@comment'], [/\/\/.*$/, 'comment'],
          [/^\s*#pragma\b.*$/, 'keyword'], [/(script)(\s+)([A-Za-z_]\w*)/, ['keyword', 'white', 'entity.name.function']], [/[+-]?\d+\s*:/, 'number'],
          [/[a-zA-Z_]\w*(?=\s*:)/, 'entity.name.label'],
          [/ins_\d+(?=\s*\()/, 'entity.name.function.unresolved'],
          [/[a-zA-Z_]\w*(?=\s*\()/, { cases: { '@keywords': 'keyword', '@default': 'entity.name.function' } }],
          [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@default': 'identifier' } }],
          [/"/, 'string', '@string'], [/0x[0-9a-fA-F]+|\d*\.\d+(?:f)?|\d+f?/, 'number'],
          [/[{}()[\]]/, '@brackets'], [/[,:;]/, 'delimiter'], [/[+\-*/%=<>!&|]+/, 'operator']
        ],
        comment: [[/[^/*]+/, 'comment'], [/\*\//, 'comment', '@pop'], [/[/*]/, 'comment']],
        string: [[/[^"\\]+/, 'string'], [/\\./, 'string.escape'], [/"/, 'string', '@pop']]
      }
}

export function ensureTruthLanguagesRegistered(): void {
  for (const [id, extensions] of [[msgLanguageId, ['.msgspec']], [endingLanguageId, ['.endspec']], [stdLanguageId, ['.stdspec']]] as const) {
    if (registered.has(id)) continue
    monaco.languages.register({ id, extensions: [...extensions] })
    monaco.languages.setLanguageConfiguration(id, dialectLanguageConfiguration)
    monaco.languages.setMonarchTokensProvider(id, truthTokens)
    monaco.languages.registerCompletionItemProvider(id, {
      async provideCompletionItems(model, position) {
        try {
          const data = await definitions(model)
          const word = model.getWordUntilPosition(position)
          const range = new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn)
          const suggestions: monaco.languages.CompletionItem[] = data.instructions.flatMap(i => (i.aliases.length ? i.aliases : [`ins_${i.opcode}`]).map(name => ({ label: name, insertText: `${name}(${(i.params || []).map((p, n) => '${' + (n + 1) + ':' + (p.enumName || p.type) + '}').join(', ')})`, insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, kind: monaco.languages.CompletionItemKind.Function, detail: docs(i), range })))
          for (const [enumName, values] of Object.entries(data.enums)) for (const [name, value] of Object.entries(values)) suggestions.push({ label: name, insertText: name, kind: monaco.languages.CompletionItemKind.EnumMember, detail: `${enumName} = ${value}`, range })
          suggestions.push(...['true', 'false'].map(name => ({ label: name, insertText: name, kind: monaco.languages.CompletionItemKind.EnumMember, range })))
          const analysis = await analyzeTruthSource(model.getValue())
          const scope = analysis.scripts.find(s => position.lineNumber >= s.line && position.lineNumber <= s.endLine)
          for (const constant of analysis.constants.filter(c => !c.script || c.script === scope?.name)) suggestions.push({ label: constant.name, insertText: constant.name, kind: monaco.languages.CompletionItemKind.Constant, range })
          for (const label of analysis.labels.filter(l => l.script === scope?.name)) suggestions.push({ label: label.name, insertText: label.name, kind: monaco.languages.CompletionItemKind.Reference, range })
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
        const call = truthCallContext(prefix)
        if (!call) return null
        try {
          const found = (await definitions(model)).instructions.find(i => i.aliases.includes(call.name) || `ins_${i.opcode}` === call.name)
          if (!found?.params) return null
          return { value: { activeSignature: 0, activeParameter: Math.min(call.parameter, Math.max(0, found.params.length - 1)), signatures: [{ label: docs(found).split('\n')[0], documentation: found.description || '', parameters: found.params.map((p, n) => ({ label: `${p.enumName || p.type} arg${n + 1}` })) }] }, dispose() {} }
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
    monaco.languages.registerDocumentSymbolProvider(id, {
      async provideDocumentSymbols(model) {
        const data = await analyzeTruthSource(model.getValue())
        return data.scripts.map(s => ({ name: s.name, detail: 'script', tags: [], kind: monaco.languages.SymbolKind.Module, range: new monaco.Range(s.line, 1, s.endLine, model.getLineMaxColumn(s.endLine)), selectionRange: new monaco.Range(s.line, 1, s.line, model.getLineMaxColumn(s.line)), children: data.labels.filter(l => l.script === s.name).map(l => ({ name: l.name, detail: 'label', tags: [], kind: monaco.languages.SymbolKind.Key, range: new monaco.Range(l.line, 1, l.line, model.getLineMaxColumn(l.line)), selectionRange: new monaco.Range(l.line, 1, l.line, model.getLineMaxColumn(l.line)) })) }))
      }
    })
    monaco.languages.registerReferenceProvider(id, {
      async provideReferences(model, position, context) {
        const data = await analyzeTruthSource(model.getValue())
        const scope = data.scripts.find(s => position.lineNumber >= s.line && position.lineNumber <= s.endLine)
        const word = model.getWordAtPosition(position)?.word
        const label = data.labels.find(l => l.script === scope?.name && l.name === word)
        if (!label) return []
        const refs = data.jumps.filter(j => j.targetLine === label.line).map(j => ({ uri: model.uri, range: new monaco.Range(j.line, j.column, j.line, j.endColumn) }))
        if (context.includeDeclaration) refs.push({ uri: model.uri, range: new monaco.Range(label.line, 1, label.line, model.getLineMaxColumn(label.line)) })
        return refs
      }
    })
    registered.add(id)
  }
}
