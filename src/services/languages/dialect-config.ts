import type * as monaco from 'monaco-editor'

/**
 * `.dstd` / `.dmsg` 共用的 Monaco 语言配置。
 *
 * 两种方言的**语法形状不同**（见各自的 tokenizer），但编辑器层面的行为是一样的：
 * `//` 行注释、只有圆括号、没有块注释。抽出来是为了不让两份十几行的配置各自漂移。
 *
 * `.dstd` 此前只注册了语言 id 而没有配置，于是 Ctrl+/ 注释切换、括号匹配、
 * 自动配对全都不工作——注册语言不等于配好语言。
 */
export const dialectLanguageConfiguration: monaco.languages.LanguageConfiguration = {
  comments: {
    // 我们的翻译器把指令说明写成 ` // ...`，反向翻译时整段剥掉，所以行注释是安全的
    lineComment: '//'
  },
  brackets: [['(', ')']],
  autoClosingPairs: [{ open: '(', close: ')' }],
  surroundingPairs: [{ open: '(', close: ')' }]
}
