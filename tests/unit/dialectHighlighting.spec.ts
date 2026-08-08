import { describe, expect, it } from 'vitest'
import { buildStdMonarchLanguage } from '../../src/services/languages/std/tokenizer'
import { buildMsgMonarchLanguage } from '../../src/services/languages/msg/tokenizer'
import { timelineKeywords } from '../../src/services/languages/std/timeline'
import { inferMonacoLanguageId } from '../../src/services/languages/dispatch'
import { stdLanguageId } from '../../src/services/languages/std/register'
import { msgLanguageId } from '../../src/services/languages/msg/register'
import { createTokenizer, typeOf } from '../helpers/monarch'

/**
 * 这些用例跑的是**真正的 Monarch tokenizer**（见 `tests/helpers/monarch.ts`），
 * 输入一律照抄 thstd / thmsg 真实 dump 出来的行。
 *
 * 两条都是有代价换来的：本项目刚发现两个 translator 因为对着想象的格式写正则，
 * 在真实输出上是恒等函数，而 22 个测试全绿。高亮这层不重蹈。
 */

const std = createTokenizer(stdLanguageId, buildStdMonarchLanguage())
const msg = createTokenizer(msgLanguageId, buildMsgMonarchLanguage())

describe('.dstd 高亮（thstd 真实输出）', () => {
  /** thstd 的文件头键一律首字母大写，而且**带缩进** */
  it('文件头的键着色为指令性关键字，含缩进的', () => {
    expect(typeOf(std, 'ANM: st01wl.anm', 'ANM')).toBe('keyword.directive')
    expect(typeOf(std, 'SCRIPT:', 'SCRIPT')).toBe('keyword.directive')
    expect(typeOf(std, '    Position: 0 0 300', 'Position')).toBe('keyword.directive')
    expect(typeOf(std, '        Script_index: 0', 'Script_index')).toBe('keyword.directive')
    expect(typeOf(std, '    FACE: 256 0 280 0', 'FACE')).toBe('keyword.directive')
  })

  it('时间标签单独着色', () => {
    expect(typeOf(std, '720:', '720')).toBe('entity.name.label')
    expect(typeOf(std, '+30:', '+30')).toBe('entity.name.label')
  })

  /** 普通指令（非时间线语义）落 function，不能被首字母大写的文件头规则抢走 */
  it('已命名指令与文件头的键不混淆', () => {
    const line = '    pos(0.0f, -200.0f, -400.0f);'
    expect(typeOf(std, line, 'pos')).toBe('entity.name.function')
    expect(typeOf(std, line, '-200.0f')).toBe('number.float')
    expect(std(line).some((t) => t.type === 'keyword.directive')).toBe(false)
  })

  /** `posTime` 与 `pos` 只差三个字母，但一个是插值区间、一个是瞬时赋值 */
  it('插值指令与同前缀的瞬时指令着色不同', () => {
    expect(typeOf(std, '    pos(0.0f, 0.0f, 0.0f);', 'pos')).toBe('entity.name.function')
    expect(typeOf(std, '    posTime(700, 4, 0.0f, 1.0f, 2.0f);', 'posTime')).toBe(
      'entity.name.function.builtin'
    )
  })

  /** 带时间线语义的指令（jmp/interruptLabel/stop/*Time/*Bezier）要更醒目 */
  it('时间线相关指令着色为 builtin', () => {
    expect(typeOf(std, '    jmp(4220, 728);', 'jmp')).toBe('entity.name.function.builtin')
    expect(typeOf(std, '    interruptLabel(1);', 'interruptLabel')).toBe(
      'entity.name.function.builtin'
    )
    expect(typeOf(std, '    stop();', 'stop')).toBe('entity.name.function.builtin')
    // 与时间线面板同源，不再各存一份名字列表
    expect(buildStdMonarchLanguage().timelineKeywords).toEqual(timelineKeywords('std'))
  })

  /** 翻译器查不到名字才写成 `ins_N`——形状本身就说明"这个我们不认识" */
  it('未命名的操作号压暗，不与已命名指令同色', () => {
    expect(typeOf(std, '    ins_19(1);', 'ins_19')).toBe('entity.name.function.unresolved')
  })

  /**
   * ARGB 颜色字面量与方言声明行都以 `#` 开头，看着会打架。
   *
   * 实际不会：Monarch 把正则源码以 `^` 开头的规则标为 `matchOnlyAtLineStart`，
   * 只在行首尝试。这条用例把这个依赖钉住——哪天有人把方言声明规则的 `^` 去掉，
   * 整行颜色就会被吞成注释。
   */
  it('颜色字面量不会被方言声明规则吞成注释', () => {
    const line = '    fog(#b0d08000, 300.0f, 600.0f);'
    expect(typeOf(std, line, '#b0d08000')).toBe('number.hex')
    expect(typeOf(std, line, '600.0f')).toBe('number.float')
    expect(std(line).some((t) => t.type === 'comment')).toBe(false)
  })

  it('方言声明行本身是注释', () => {
    expect(typeOf(std, '# THTK-Studio dstd 方言：…', '# THTK-Studio dstd 方言：…')).toBe('comment')
  })

  /** 行尾分号是 thstd 输出的一部分，不能把后面的注释挡掉 */
  it('行尾分号之后的注释仍被识别', () => {
    expect(typeOf(std, '    jmp(4220, 728); // 跳转', '// 跳转')).toBe('comment')
  })
})

describe('.dmsg 高亮（thmsg 真实输出）', () => {
  /** thmsg 的时间标签是 `@120`，不是 `120:` */
  it('@ 形式的时间标签', () => {
    expect(typeOf(msg, '@0', '0')).toBe('entity.name.label')
    expect(typeOf(msg, '@1200', '1200')).toBe('entity.name.label')
  })

  /** `header(0, 0)` 顶格、也长得像调用，必须先被文件级规则拦下 */
  it('header / entry 是文件级的行，不是指令', () => {
    expect(typeOf(msg, 'header(0, 0)', 'header')).toBe('keyword.directive')
    expect(typeOf(msg, 'entry 0', 'entry')).toBe('keyword.directive')
  })

  it('已命名指令与未命名操作号分开着色', () => {
    expect(typeOf(msg, '\ttextAdd(hi)', 'textAdd')).toBe('entity.name.function')
    expect(typeOf(msg, '\tins_250(42)', 'ins_250')).toBe('entity.name.function.unresolved')
  })

  /** 数字实参与对白正文要一眼分得开——MSG 文件里真正被编辑的是台词 */
  it('数字实参是数字，对白正文是字符串', () => {
    expect(typeOf(msg, '\tplayerFace(0;1)', '0')).toBe('number')
    expect(typeOf(msg, '\tplayerFace(0;1)', '1')).toBe('number')
    expect(typeOf(msg, '\ttextAdd(こんにちは)', 'こんにちは')).toBe('string')
  })

  /**
   * 台词里的半角逗号必须留在同一个字符串 token 里。
   *
   * 若按逗号断开着色，等于在视觉上暗示可以按逗号拆参数——而那正是会静默截断
   * 台词的误解（见 `modules/msg/translator.rs` 里 `value_to_text` 的裸拷贝）。
   */
  it('台词里的逗号不断开字符串', () => {
    const line = '\ttextAdd(こんにちは、世界, and hello)'
    expect(typeOf(msg, line, 'こんにちは、世界, and hello')).toBe('string')
  })

  /** 台词里可以出现右括号；收尾括号必须取最后那个，与 Rust 侧的贪婪匹配一致 */
  it('台词里的右括号不提前收尾', () => {
    const line = '\ttextAdd(笑 (ぐぬぬ) だ)'
    expect(typeOf(msg, line, '笑 (ぐぬぬ) だ')).toBe('string')
  })

  it('无参指令与行尾注释', () => {
    const line = '\ttextboxShow() // 显示对话框'
    expect(typeOf(msg, line, 'textboxShow')).toBe('entity.name.function')
    expect(typeOf(msg, line, '// 显示对话框')).toBe('comment')
  })

  /** MSG 没有跳转 / 中断 / 插值，不该套用 STD 的特殊指令名 */
  it('不把 STD 的时间线关键字套到 MSG', () => {
    expect(timelineKeywords('msg')).toEqual([])
    expect(typeOf(msg, '\tjmp(0;20)', 'jmp')).toBe('entity.name.function')
  })
})

describe('按扩展名分派语言', () => {
  it('.dstd 与 .dmsg 各走各的语言服务', () => {
    expect(inferMonacoLanguageId({ path: '/p/st01.dstd' })).toBe(stdLanguageId)
    expect(inferMonacoLanguageId({ path: '/p/st01.dmsg' })).toBe(msgLanguageId)
    expect(stdLanguageId).not.toBe(msgLanguageId)
  })

  /** 回归：`.dmsg` 此前一路落到 plaintext——既没高亮，也没有 Ctrl+/ */
  it('.dmsg 不再落到 plaintext', () => {
    expect(inferMonacoLanguageId({ path: '/p/st01.dmsg' })).not.toBe('plaintext')
  })
})
