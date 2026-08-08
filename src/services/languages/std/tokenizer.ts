import type * as monaco from 'monaco-editor'
import { timelineKeywords } from './timeline'

/**
 * `.dstd` 的 Monarch 语法高亮。
 *
 * ## 形状照抄 thstd 的 dump 函数
 *
 * ```c
 * fprintf(stream, "    Position: %g %g %g\n", ...);   // 文件头：首字母大写的键
 * fprintf(stream, "%i:\n", instr->time);              // 时间标签：顶格、独占一行
 * fprintf(stream, "    ins_%i(", instr->type);        // 指令：4 空格缩进
 * fprintf(stream, ");\n");                            // ★行尾分号
 * ```
 *
 * 所以规则必须认**带缩进的行**与**行尾分号**——这正是翻译器和跳转导航栽过的地方。
 *
 * ## 不需要动态词表
 *
 * ECL 的 tokenizer 要从 eclmap 灌指令名进来，STD 不用：翻译器查得到名字的写成
 * 名字、查不到的写成 `ins_N`，**形状本身就带了"认不认识"这个信息**。于是
 * `ins_\d+` 一条规则就够，既不必把 mapfile 暴露到前端，也不会随版本漂移。
 */

/**
 * 各条规则的正则单独具名，便于直接对**真实 dump 行**做单元测试——
 * 测试环境里 monaco 是桩，跑不了 Monarch 本身，能验的就是这些模式。
 */
export const STD_PATTERNS = {
  /**
   * 方言声明行 `# THTK-Studio …`。
   *
   * STD 的 ARGB 颜色字面量也以 `#` 开头（`#b0d08000`），看着像会冲突，其实不会：
   * Monarch 把**正则源码以 `^` 开头**的规则标为 `matchOnlyAtLineStart`，只在
   * `pos === 0` 时尝试（`monarchCompile.js` 建 Rule 时判定，`monarchLexer.js`
   * 匹配前检查）。所以行中间的 `#` 永远碰不到这条规则。
   *
   * 下面 `headerKey` / `timeLabel` 能安全地用 `^` 也是同一个道理。
   */
  dialectHeader: /^\s*#.*$/,
  /** 文件头的键：`ANM:` `ENTRY:` `QUAD:` `FACE:` `SCRIPT:` `Width:` —— thstd 一律首字母大写 */
  headerKey: /^(\s*)([A-Z]\w*)(\s*:)/,
  /** 时间标签：`720:` `+30:` `@sub:` */
  timeLabel: /^(\s*)([+-]?\d+|@\w+)(\s*:)/,
  /** ARGB 颜色字面量 `#b0d08000`。落 `number.hex`，主题里按 `number` 前缀取色 */
  color: /#[0-9a-fA-F]{6,8}\b/,
  /** 查不到名字的操作号 */
  unresolvedInstruction: /\bins_\d+\b/,
  /** 调用位上的指令名 */
  instruction: /[A-Za-z_]\w*(?=\s*\()/,
  /** thstd 的浮点带 `f` 后缀：`0.06f`、`-200.0f`、`0f` */
  float: /[-+]?\d+(?:\.\d+)?f\b|[-+]?\d+\.\d+\b/,
  integer: /[-+]?\d+\b/
} as const

export function buildStdMonarchLanguage(): monaco.languages.IMonarchLanguage {
  return {
    defaultToken: '',
    tokenPostfix: '.dstd',
    // Monarch 的 `@timelineKeywords` 从这里取值
    timelineKeywords: timelineKeywords('std'),
    tokenizer: {
      root: [
        // 行首规则必须排在消耗空白之前，否则 `^` 就不在行首了
        [STD_PATTERNS.dialectHeader, 'comment'],
        [STD_PATTERNS.headerKey, ['white', 'keyword.directive', 'delimiter']],
        [STD_PATTERNS.timeLabel, ['white', 'entity.name.label', 'delimiter']],
        { include: '@body' }
      ],
      body: [
        [/\/\/.*$/, 'comment'],
        [STD_PATTERNS.color, 'number.hex'],
        [STD_PATTERNS.unresolvedInstruction, 'entity.name.function.unresolved'],
        [
          STD_PATTERNS.instruction,
          {
            cases: {
              // jmp / interruptLabel / stop / *Time / *Bezier —— 与时间线面板同源
              '@timelineKeywords': 'entity.name.function.builtin',
              '@default': 'entity.name.function'
            }
          }
        ],
        [STD_PATTERNS.float, 'number.float'],
        [STD_PATTERNS.integer, 'number'],
        [/[()]/, '@brackets'],
        [/[,;:]/, 'delimiter'],
        [/[A-Za-z_][\w.]*/, 'identifier'],
        [/\s+/, 'white']
      ]
    }
  }
}
