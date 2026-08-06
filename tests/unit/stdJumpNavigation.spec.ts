import { describe, expect, it } from 'vitest'
import {
  analyzeStd,
  findJumpTargets,
  jumpSpecFor
} from '../../src/services/languages/std/jumpNavigation'

/**
 * 指令字节大小 = 8（头）+ 4 × 参数个数。
 * 来自 truth 的 `instr_header_size() = 8` 与 thtk 的
 * `std_instr_t = {u32 time; u16 type; u16 size}`（size 含头）。
 */
const SCRIPT = [
  'SCRIPT:',             // 1  段标签
  '0:',                  // 2  纯时间标签
  '    pos(0f, 0f, 0f)', // 3  offset 0，3 参 → 20 字节
  '    stop()',          // 4  offset 20，0 参 → 8 字节
  '    fov(0f)',         // 5  offset 28，1 参 → 12 字节
  '60:',                 // 6
  '    jmp(0, 20)'       // 7  offset 40；目标字节偏移 20 = 第 4 行
].join('\n')

describe('jumpSpecFor —— 按版本条件化，复刻 truth 的分档', () => {
  it('th06 没有跳转指令', () => {
    expect(jumpSpecFor(6)).toBeNull()
  })

  /** truth: `(Th07, 4, Some(("ot_", Some(IKind::Jmp))))`，且 decode_label = bits * 20 */
  it('th07–th09 的跳转是 opcode 4，参数是指令序号', () => {
    for (const v of [7, 8, 9]) {
      const spec = jumpSpecFor(v)
      expect(spec?.opcode).toBe(4)
      expect(spec?.offsetScale).toBe(20)
      expect(spec?.fixedInstrBytes).toBe(20)
    }
  })

  /** truth: `(Th095, 1, Some(("ot", Some(IKind::Jmp))))`，StdHooks10 未 override decode_label */
  it('th095 起是 opcode 1，参数就是字节偏移', () => {
    for (const v of [95, 10, 14, 17, 18, 185, 19, 20]) {
      const spec = jumpSpecFor(v)
      expect(spec?.opcode).toBe(1)
      expect(spec?.offsetScale).toBe(1)
      expect(spec?.fixedInstrBytes).toBeNull()
    }
  })
})

describe('analyzeStd（th17）', () => {
  it('按 8 + 4×参数 累计每条指令的偏移', () => {
    const { instructions } = analyzeStd(SCRIPT, 17)
    expect(instructions.map((i) => [i.line, i.offset])).toEqual([
      [3, 0],
      [4, 20],
      [5, 28],
      [7, 40]
    ])
  })

  it('段标签与纯时间标签不计入指令', () => {
    const { instructions } = analyzeStd(SCRIPT, 17)
    expect(instructions.every((i) => ![1, 2, 6].includes(i.line))).toBe(true)
  })

  /**
   * 核心：偏移直接查表，**没有基准推断**。
   * truth 的 StdHooks10 未 override decode_label，默认就是脚本段起点起算。
   */
  it('跳转偏移直接解析到目标行', () => {
    const { jumps } = analyzeStd(SCRIPT, 17)
    expect(jumps).toHaveLength(1)
    expect(jumps[0].sourceLine).toBe(7)
    expect(jumps[0].byteOffset).toBe(20)
    expect(jumps[0].targetLine).toBe(4)
  })

  /** 宁可不显示，也不要指向错的行——与"不把 label 写进文件"是同一条理由。 */
  it('偏移没落在指令边界上时不给目标', () => {
    const { jumps } = analyzeStd(SCRIPT.replace('jmp(0, 20)', 'jmp(0, 7)'), 17)
    expect(jumps[0].targetLine).toBeNull()
  })
})

describe('v0（th07–th09）：参数是指令序号而非字节偏移', () => {
  /** v0 指令定长 20 字节，跳转是 opcode 4，实参 2 表示"第 2 条指令"= 字节 40 */
  const V0_SCRIPT = [
    '    ins_0(1, 0f, 0f)', // 1  offset 0
    '    ins_1(2, 0f, 0f)', // 2  offset 20
    '    ins_5(3, 0f, 0f)', // 3  offset 40
    '    ins_4(2, 0)'       // 4  offset 60；序号 2 → 字节 40 = 第 3 行
  ].join('\n')

  it('按序号 × 20 换算并解析到目标行', () => {
    const { jumps } = analyzeStd(V0_SCRIPT, 8)
    expect(jumps).toHaveLength(1)
    expect(jumps[0].rawOffset).toBe(2)
    expect(jumps[0].byteOffset).toBe(40)
    expect(jumps[0].targetLine).toBe(3)
  })

  /** v0 的 opcode 1 是普通指令（thtk formats_v0 里是 "Sff"），不该被当跳转 */
  it('v0 的 opcode 1 不是跳转', () => {
    const { jumps } = analyzeStd('    ins_1(2, 0f, 0f)', 8)
    expect(jumps).toEqual([])
  })

  it('th06 没有跳转，任何 opcode 都不算', () => {
    expect(analyzeStd('    ins_4(2, 0)', 6).jumps).toEqual([])
  })
})

describe('findJumpTargets', () => {
  it('忽略注释行里的 jmp', () => {
    expect(findJumpTargets('    // jmp(0, 1)', 17)).toEqual([])
    expect(findJumpTargets('  # jmp(0, 1)', 17)).toEqual([])
  })

  /**
   * 原始 `ins_1(offset, time)` 与翻译后的 `jmp(time, offset)` 顺序相反。
   * truth 的签名是 `ot`（offset 在前）＝ thstd 的原始顺序；我们的 translator
   * 对该 opcode 做了双向交换，所以两种都要认，且结果必须一致。
   */
  it('ins_1 与 jmp 的参数顺序相反，解析结果须一致', () => {
    const translated = findJumpTargets('    jmp(60, 1200)', 17)[0]
    const raw = findJumpTargets('    ins_1(1200, 60)', 17)[0]
    expect(translated.time).toBe(60)
    expect(translated.rawOffset).toBe(1200)
    expect(raw.time).toBe(60)
    expect(raw.rawOffset).toBe(1200)
  })

  it('认时间标签前缀', () => {
    expect(findJumpTargets('120: jmp(0, 5)', 17)).toHaveLength(1)
    expect(findJumpTargets('+30: jmp(0, 5)', 17)).toHaveLength(1)
    expect(findJumpTargets('@sub: jmp(0, 5)', 17)).toHaveLength(1)
  })

  it('不把 jmpFoo 之类的名字误当跳转', () => {
    expect(findJumpTargets('    jmpFoo(0, 1)', 17)).toEqual([])
  })

  it('行尾注释不影响解析', () => {
    const links = findJumpTargets('    jmp(0, 20)  // 回到开头', 17)
    expect(links).toHaveLength(1)
    expect(links[0].rawOffset).toBe(20)
  })
})
