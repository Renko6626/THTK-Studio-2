import type * as monaco from 'monaco-editor'

/**
 * `.dmsg` 的 Monarch 语法高亮。
 *
 * ## 形状照抄 thmsg 的 dump 函数
 *
 * ```c
 * fprintf(out, "header(");            // 文件级：header(0, 0)
 * fprintf(out, "entry %u", entry_id); // 文件级：entry 0
 * fprintf(out, "@%u\n", time);        // 时间标签：★是 @120，不是 120:
 * fprintf(out, "\t%d", msg->type);    // 指令：TAB 缩进
 * fprintf(out, ";%s", disp);          // 实参：★分号分隔
 * ```
 *
 * 我们的翻译器把操作号换成名字并套上括号，但**括号里仍是分号分隔**——原因见
 * `modules/msg/translator.rs`：对白是裸字节、不转义，正文里可以有半角逗号。
 *
 * ## 高亮的重点是对白正文
 *
 * MSG 文件里真正被编辑的是台词。所以实参分两类着色：**整个实参都是数字**的落
 * `number`，其余一律落 `string`。这样 `textAdd(こんにちは、世界)` 读起来就是
 * 一段文本，而 `playerFace(0;1)` 读起来是参数。
 *
 * 逗号**不着色为分隔符**——它是正文的一部分。把它标成 delimiter 会让人以为
 * 可以按逗号拆参数，而那正是会静默截断台词的误解。
 */

export const MSG_PATTERNS = {
  /** 方言声明行 `# THTK-Studio …` */
  dialectHeader: /^\s*#.*$/,
  /** 时间标签 `@120`（thmsg 的写法，不是 `120:`） */
  timeLabel: /^(@)(\d+)(\s*)$/,
  /** 文件级：`entry 0` / `entry 0 (1, 2, 3, 4)` */
  entry: /^(entry)(\b.*)$/,
  /** 文件级：`header(0, 0)` —— 顶格、也长得像调用，必须先于指令规则拦下 */
  header: /^(header)(\()([^)]*)(\))/,
  /** 查不到名字的操作号 */
  unresolvedInstruction: /\bins_\d+(?=\s*\()/,
  /** 调用位上的指令名 */
  instruction: /[A-Za-z_]\w*(?=\s*\()/,
  /**
   * 收尾右括号：只有后面除了行尾注释再无别物时才算。
   *
   * 台词里完全可能出现 `)`（`笑 (ぐぬぬ) だ`），若见到第一个 `)` 就收尾，
   * 后半句会掉出字符串着色。加这个前瞻等价于 Rust 侧的贪婪匹配。
   */
  closingParen: /\)(?=\s*(?:\/\/.*)?$)/,
  /** 整个实参都是数字才算数字 */
  numericArg: /[-+]?\d+(?:\.\d+)?f?(?=[;)])/,
  /** 其余实参 = 对白正文，惰性吃到下一个 `;` 或收尾右括号 */
  textArg: /[^;]+?(?=;|\)\s*(?:\/\/.*)?$)/
} as const

export function buildMsgMonarchLanguage(): monaco.languages.IMonarchLanguage {
  return {
    defaultToken: '',
    tokenPostfix: '.dmsg',
    tokenizer: {
      root: [
        [MSG_PATTERNS.dialectHeader, 'comment'],
        [MSG_PATTERNS.timeLabel, ['delimiter', 'entity.name.label', 'white']],
        [MSG_PATTERNS.entry, ['keyword.directive', 'number']],
        [MSG_PATTERNS.header, ['keyword.directive', 'delimiter', 'number', 'delimiter']],
        [/\/\/.*$/, 'comment'],
        [MSG_PATTERNS.unresolvedInstruction, 'entity.name.function.unresolved'],
        [MSG_PATTERNS.instruction, 'entity.name.function'],
        [/\(/, { token: 'delimiter.parenthesis', next: '@args' }],
        [/\s+/, 'white']
      ],
      args: [
        [MSG_PATTERNS.closingParen, { token: 'delimiter.parenthesis', next: '@pop' }],
        [/;/, 'delimiter'],
        [MSG_PATTERNS.numericArg, 'number'],
        [MSG_PATTERNS.textArg, 'string'],
        // 兜底：不该走到这（上面两条已覆盖），但 Monarch 卡死比着色错更糟
        [/[^;)]+/, 'string'],
        [/\)/, { token: 'delimiter.parenthesis', next: '@pop' }]
      ]
    }
  }
}
