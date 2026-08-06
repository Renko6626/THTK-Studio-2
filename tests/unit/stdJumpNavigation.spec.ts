import { describe, expect, it } from 'vitest'
import { findJumpTargets } from '../../src/services/languages/std/jumpNavigation'

/** 时间标签形如 `60:`；跳转在我们翻译后是 `jmp(time, offset)` */
const TRANSLATED = [
  '0:',
  '    pos(0f, 0f, 0f)',
  '60:',
  '    jmp(0, 1)',
  '    stop()'
].join('\n')

describe('findJumpTargets', () => {
  it('解析出源行、时间与偏移', () => {
    const links = findJumpTargets(TRANSLATED)
    expect(links).toHaveLength(1)
    expect(links[0].sourceLine).toBe(4)
    expect(links[0].time).toBe(0)
    expect(links[0].offset).toBe(1)
  })

  it('没有跳转时返回空', () => {
    expect(findJumpTargets('    pos(0f, 0f, 0f)\n    stop()')).toEqual([])
  })

  it('忽略注释行里的 jmp', () => {
    expect(findJumpTargets('    // jmp(0, 1)')).toEqual([])
    expect(findJumpTargets('  # jmp(0, 1)')).toEqual([])
  })

  /**
   * 关键：两种写法的参数顺序**相反**。
   * translator 对 opcode 1 有双向交换实参的特例，这里必须跟它一致，
   * 否则识别出的跳转目标会张冠李戴。
   */
  it('ins_1 与 jmp 的参数顺序相反，解析结果须一致', () => {
    const translated = findJumpTargets('    jmp(60, 1200)')[0]
    const raw = findJumpTargets('    ins_1(1200, 60)')[0]

    expect(translated.time).toBe(60)
    expect(translated.offset).toBe(1200)
    expect(raw.time).toBe(60)
    expect(raw.offset).toBe(1200)
  })

  it('认时间标签前缀', () => {
    expect(findJumpTargets('120: jmp(0, 5)')).toHaveLength(1)
    expect(findJumpTargets('+30: jmp(0, 5)')).toHaveLength(1)
    expect(findJumpTargets('@sub: jmp(0, 5)')).toHaveLength(1)
  })

  it('认负偏移（向前跳）', () => {
    const links = findJumpTargets('    jmp(0, -240)')
    expect(links[0].offset).toBe(-240)
  })

  it('多条跳转各自记录行号', () => {
    const links = findJumpTargets('    jmp(0, 1)\n    stop()\n    jmp(30, 2)')
    expect(links.map((l) => l.sourceLine)).toEqual([1, 3])
  })

  it('不把 jmpFoo 之类的名字误当跳转', () => {
    expect(findJumpTargets('    jmpFoo(0, 1)')).toEqual([])
  })
})
