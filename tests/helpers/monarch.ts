import { compile } from 'monaco-editor/esm/vs/editor/standalone/common/monarch/monarchCompile.js'
import { MonarchTokenizer } from 'monaco-editor/esm/vs/editor/standalone/common/monarch/monarchLexer.js'

/**
 * 跑**真正的 Monarch tokenizer**。
 *
 * 只验正则是不够的：规则顺序、状态迁移（`@push` / `@pop`）、`cases` 的 `@关键字表`
 * 查表、以及"Monarch 是拿行内**剩余部分**去匹配"这个容易踩的语义，都只有让它真跑
 * 一遍才看得出来。本项目已经吃过一次"对着想象的格式写规则、测试全绿而功能全废"
 * 的亏（见 msg/std 两个 translator），高亮这层不重蹈。
 *
 * `monarchCompile` / `monarchLexer` 是纯逻辑模块，不需要 DOM、worker 或主题服务，
 * 因此可以在 happy-dom 里直接构造。
 */

/** 一个 token：原文 + 类型（已去掉 `.dstd` / `.dmsg` 这类 tokenPostfix） */
export interface TokenSlice {
  text: string
  type: string
}

export type Tokenize = (line: string) => TokenSlice[]

/** 编译一份 Monarch 语言定义，得到逐行分词的函数。 */
export function createTokenizer(languageId: string, languageDef: unknown): Tokenize {
  const lexer = compile(languageId, languageDef)
  const configurationService = {
    getValue: () => 20000,
    onDidChangeConfiguration: () => ({ dispose: () => {} })
  }
  const languageService = {
    languageIdCodec: { encodeLanguageId: () => 1, decodeLanguageId: () => languageId }
  }
  const tokenizer = new MonarchTokenizer(
    languageService,
    null,
    languageId,
    lexer,
    configurationService
  )
  const postfix = new RegExp(`\\.${languageId.replace(/^thtk-/, 'd')}$`)

  return (line: string): TokenSlice[] => {
    const { tokens } = tokenizer.tokenize(line, true, tokenizer.getInitialState())
    return tokens
      .map((token, index) => ({
        text: line.slice(token.offset, tokens[index + 1]?.offset ?? line.length),
        type: token.type.replace(postfix, '')
      }))
      // 空白 token 是噪声，断言里不关心
      .filter((slice) => slice.type !== 'white' && slice.text.trim() !== '')
  }
}

/**
 * 取覆盖 `text` 那个 token 的类型。
 *
 * 断言写成「这段原文应该是什么颜色」最贴近人肉验收时看的东西。
 * 找不到（说明该片段被拆散或被别的规则吞了）就返回 null，让断言明确失败。
 */
export function typeOf(tokenize: Tokenize, line: string, text: string): string | null {
  return tokenize(line).find((slice) => slice.text.trim() === text)?.type ?? null
}
