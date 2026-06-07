import {
  eclHeaderKeywords,
  eclKeywords,
  eclPreprocessorKeywords,
  eclTypes
} from './vocabulary'
import { createEmptyEclSemanticData, normalizeEclSemanticData } from './dynamic-vocabulary'

const preprocessorPattern = new RegExp(
  `^\\s*#(?:${eclPreprocessorKeywords.join('|')})\\b`
)

export function buildEclTokenizer(semanticData = createEmptyEclSemanticData()) {
  const normalized = normalizeEclSemanticData(semanticData)

  return {
    root: [
      { include: '@whitespace' },
      [preprocessorPattern, 'preprocessor'],
      [/^\s*([A-Za-z_]\w*)(:)/, ['entity.name.label', 'delimiter']],
      [/\b(goto)(\s+)([A-Za-z_]\w*)(\s*)(@)(\s*)(\d+)/, ['keyword', '', 'entity.name.label', '', 'operator', '', 'number']],
      [/!\*|![ENHL]+[0-9]*:?/, 'keyword.difficulty'],
      [/\[\s*-?\d+(?:\.\d+)?f?\s*\]/, 'meta.stack'],
      [/@[A-Za-z_]\w*/, 'entity.name.function'],
      [/\$[A-Za-z_]\w*/, 'variable.special'],
      [/%[A-Za-z_]\w*/, 'variable.predefined'],
      [/[{}()[\]]/, '@brackets'],
      [/[;,]/, 'delimiter'],
      [/[+\-*/=<>!]+/, 'operator'],
      [/\b\d+\.\d+f?\b/, 'number.float'],
      [/\b\d+\b/, 'number'],
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, 'string', '@string'],
      [/[A-Za-z_]\w*(?=\s*\()/, {
        cases: {
          '@types': 'type',
          '@keywords': 'keyword',
          '@headerKeywords': 'keyword.directive',
          '@instructionKeywords': 'entity.name.function.builtin',
          '@builtinIdentifiers': 'variable.predefined',
          '@default': 'entity.name.function'
        }
      }],
      [/[A-Za-z_]\w*/, {
        cases: {
          '@types': 'type',
          '@keywords': 'keyword',
          '@headerKeywords': 'keyword.directive',
          '@instructionKeywords': 'entity.name.function.builtin',
          '@builtinIdentifiers': 'variable.predefined',
          '@default': 'identifier'
        }
      }]
    ],
    whitespace: [
      [/[ \t\r\n]+/, ''],
      [/\/\*/, 'comment', '@comment'],
      [/\/\/.*$/, 'comment']
    ],
    comment: [
      [/[^/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/./, 'comment']
    ],
    string: [
      [/[^\\"]+/, 'string'],
      [/\\./, 'string.escape'],
      [/"/, 'string', '@pop']
    ]
  }
}

export function buildEclMonarchLanguage(semanticData = createEmptyEclSemanticData()) {
  const normalized = normalizeEclSemanticData(semanticData)

  return {
    defaultToken: '',
    tokenPostfix: '.ecl',
    keywords: eclKeywords,
    types: eclTypes,
    headerKeywords: eclHeaderKeywords,
    preprocessorKeywords: eclPreprocessorKeywords,
    instructionKeywords: normalized.instructions.map((item) => item.name),
    builtinIdentifiers: normalized.builtins,
    tokenizer: buildEclTokenizer(normalized)
  }
}
