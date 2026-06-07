import * as monaco from 'monaco-editor'
import {
  createEmptyEclSemanticData,
  normalizeEclSemanticData
} from './dynamic-vocabulary'
import { collectEclDocumentSymbols } from './document-symbols'
import {
  buildInstructionDocumentation,
  buildInstructionInsertText
} from './instruction-display'
import {
  eclHeaderKeywords,
  eclKeywords,
  eclPreprocessorKeywords,
  eclTypes
} from './vocabulary'

const difficultyLabels = ['!E', '!N', '!H', '!L', '!*']

const builtinVariables = [
  '$RAND_INT',
  '%RAND_ANGLE',
  '%RAND_FLOAT_SIGNED'
]

const keywordSnippets = [
  {
    label: 'if',
    kind: monaco.languages.CompletionItemKind.Keyword,
    insertText: 'if (${1:condition}) {\n\t$0\n}',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '条件判断'
  },
  {
    label: 'while',
    kind: monaco.languages.CompletionItemKind.Keyword,
    insertText: 'while (${1:condition}) {\n\t$0\n}',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '循环'
  },
  {
    label: 'times',
    kind: monaco.languages.CompletionItemKind.Keyword,
    insertText: 'times (${1:count}) {\n\t$0\n}',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '定次数循环'
  },
  {
    label: 'switch',
    kind: monaco.languages.CompletionItemKind.Keyword,
    insertText: 'switch (${1:value}) {\n\tcase ${2:0}:\n\t\t$0\n\t\tbreak;\n}',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '分支语句'
  },
  {
    label: 'global',
    kind: monaco.languages.CompletionItemKind.Snippet,
    insertText: 'global ${1:NAME} = ${2:value};',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '全局常量定义'
  },
  {
    label: '#include',
    kind: monaco.languages.CompletionItemKind.Snippet,
    insertText: '#include "${1:path}"',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '包含其他脚本'
  },
  {
    label: 'subroutine',
    kind: monaco.languages.CompletionItemKind.Snippet,
    insertText: 'void ${1:SubName}(${2:var A})\n{\n\t$0\n}',
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    detail: '子程序模板'
  }
]

function createCompletionRange(model, position) {
  const word = model.getWordUntilPosition(position)
  return {
    insert: {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn
    },
    replace: {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn
    }
  }
}

function createSimpleCompletion(label, kind, range, overrides = {}) {
  return {
    label,
    kind,
    insertText: label,
    range,
    ...overrides
  }
}

function buildStaticCompletions(range) {
  const items = []

  for (const keyword of eclKeywords) {
    items.push(createSimpleCompletion(
      keyword,
      monaco.languages.CompletionItemKind.Keyword,
      range,
      { sortText: `1_${keyword}` }
    ))
  }

  for (const typeName of eclTypes) {
    items.push(createSimpleCompletion(
      typeName,
      monaco.languages.CompletionItemKind.TypeParameter,
      range,
      { sortText: `1_${typeName}` }
    ))
  }

  for (const directive of eclHeaderKeywords) {
    items.push(createSimpleCompletion(
      directive,
      monaco.languages.CompletionItemKind.Keyword,
      range,
      { detail: 'ECL 头部声明', sortText: `1_${directive}` }
    ))
  }

  for (const directive of eclPreprocessorKeywords) {
    items.push(createSimpleCompletion(
      `#${directive}`,
      monaco.languages.CompletionItemKind.Keyword,
      range,
      {
        insertText: `#${directive}`,
        detail: '预处理指令',
        sortText: `1_#${directive}`
      }
    ))
  }

  for (const label of difficultyLabels) {
    items.push(createSimpleCompletion(
      label,
      monaco.languages.CompletionItemKind.EnumMember,
      range,
      { detail: '难度标签', sortText: `1_${label}` }
    ))
  }

  for (const variableName of builtinVariables) {
    items.push(createSimpleCompletion(
      variableName,
      monaco.languages.CompletionItemKind.Variable,
      range,
      { detail: '内建变量', sortText: `2_${variableName}` }
    ))
  }

  for (const snippet of keywordSnippets) {
    items.push({
      ...snippet,
      range,
      sortText: `0_${snippet.label}`
    })
  }

  return items
}

function buildSemanticCompletions(semanticData, range) {
  return semanticData.instructions.map((instruction) => createSimpleCompletion(
    instruction.name,
    monaco.languages.CompletionItemKind.Function,
    range,
    {
      insertText: buildInstructionInsertText(instruction),
      insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      detail: instruction.section
        ? `ECL 指令 · ${instruction.section} · opcode ${instruction.opcode}`
        : `ECL 指令 · opcode ${instruction.opcode}`,
      documentation: buildInstructionDocumentation(instruction),
      sortText: `2_${instruction.name}`
    }
  ))
}

function buildModelSymbolCompletions(model, range) {
  const { subroutines, globals, labels } = collectEclDocumentSymbols(model)
  const items = []

  for (const name of subroutines) {
    items.push(createSimpleCompletion(
      name,
      monaco.languages.CompletionItemKind.Function,
      range,
      { detail: '当前文件子程序', sortText: `3_${name}` }
    ))
  }

  for (const name of globals) {
    items.push(createSimpleCompletion(
      name,
      monaco.languages.CompletionItemKind.Constant,
      range,
      { detail: '当前文件全局定义', sortText: `3_${name}` }
    ))
  }

  for (const name of labels) {
    items.push(createSimpleCompletion(
      name,
      monaco.languages.CompletionItemKind.Reference,
      range,
      { detail: '当前文件标签', sortText: `3_${name}` }
    ))
  }

  return items
}

export function createEclCompletionProvider(getSemanticData = () => createEmptyEclSemanticData()) {
  return {
    triggerCharacters: ['@', '#', '$', '%', '!', '.'],
    provideCompletionItems(model, position) {
      const range = createCompletionRange(model, position)
      const semanticData = normalizeEclSemanticData(getSemanticData(model))

      const suggestions = [
        ...buildStaticCompletions(range),
        ...buildSemanticCompletions(semanticData, range),
        ...buildModelSymbolCompletions(model, range)
      ]

      return { suggestions }
    }
  }
}
