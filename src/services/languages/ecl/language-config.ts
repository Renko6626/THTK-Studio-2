import type * as monaco from 'monaco-editor'

export const eclLanguageId = 'ecl'

export const eclLanguageConfiguration: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/']
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')']
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' }
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' }
  ],
  folding: {
    markers: {
      start: new RegExp('^\\s*#(?:ifset)\\b'),
      end: new RegExp('^\\s*#(?:endif)\\b')
    }
  }
}
