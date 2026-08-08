/**
 * monaco-editor 内部 Monarch 模块的类型声明。
 *
 * 这两个模块**没有随包发布类型**，但它们是纯逻辑、不碰 DOM，可以在 vitest 里
 * headless 跑真正的 tokenizer——语法高亮的测试因此能验 token 流本身，而不是
 * 只验几条正则。`vitest.config.js` 的 alias 用带 `$` 锚的正则，正是为了不把
 * 这些深层路径劫持到 monaco 替身上。
 */
declare module 'monaco-editor/esm/vs/editor/standalone/common/monarch/monarchCompile.js' {
  export function compile(languageId: string, json: unknown): unknown
}

declare module 'monaco-editor/esm/vs/editor/standalone/common/monarch/monarchLexer.js' {
  export interface MonarchToken {
    offset: number
    type: string
  }
  export class MonarchTokenizer {
    constructor(
      languageService: unknown,
      themeService: unknown,
      languageId: string,
      lexer: unknown,
      configurationService: unknown
    )
    getInitialState(): unknown
    tokenize(
      line: string,
      hasEOL: boolean,
      state: unknown
    ): { tokens: MonarchToken[]; endState: unknown }
  }
}
