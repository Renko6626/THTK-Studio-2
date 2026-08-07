import { describe, expect, it } from 'vitest'
import {
  analyzeTimeline,
  dialectForPath
} from '../../src/services/languages/std/timeline'

const STD = [
  'SCRIPT:',                        // 1
  '0:',                             // 2
  '    pos(0f, 0f, 0f)',            // 3  instant
  '    posTime(60, 0, 1f, 2f, 3f)', // 4  span，持续 60 帧
  '60:',                            // 5
  '    interruptLabel(3)',          // 6  外部入口
  '    fog(#ff000000, 1f, 2f)',     // 7  instant
  '180:',                           // 8
  '    jmp(0, 20)',                 // 9  倒带到 time 0
  '    stop()'                      // 10 无限等待
].join('\n')

describe('analyzeTimeline（STD）', () => {
  it('按时间标签分组', () => {
    const { groups } = analyzeTimeline(STD, 'std')
    expect(groups.map((g) => g.time)).toEqual([0, 60, 180])
    expect(groups[0].events.map((e) => e.line)).toEqual([3, 4])
    expect(groups[1].events.map((e) => e.line)).toEqual([6, 7])
  })

  /** 间隔是文本给不了的信息——看 0/60/180 要心算才知道节奏 */
  it('算出相邻时间点的间隔，最后一组为 null', () => {
    const { groups } = analyzeTimeline(STD, 'std')
    expect(groups.map((g) => g.delta)).toEqual([60, 120, null])
  })

  /**
   * `*Time` 指令占的是**区间**不是点：
   * `.stdm` 说明「在接下来的 duration 帧内…平滑地改变」，duration 是第 0 个实参。
   */
  it('识别持续区间并取出 duration', () => {
    const { groups } = analyzeTimeline(STD, 'std')
    const span = groups[0].events.find((e) => e.name === 'posTime')
    expect(span?.kind).toBe('span')
    expect(span?.duration).toBe(60)
  })

  /** 「可以在脚本等待时被 ECL 指令触发」——光看 ins_16(3) 看不出这是外部入口 */
  it('识别中断标签及其编号', () => {
    const { groups } = analyzeTimeline(STD, 'std')
    const interrupt = groups[1].events.find((e) => e.kind === 'interrupt')
    expect(interrupt?.interruptId).toBe(3)
  })

  /** jmp「并将当前时间设置为 time」——是倒带，不只是控制流 */
  it('识别跳转并取出它设定的时间', () => {
    const { groups } = analyzeTimeline(STD, 'std')
    const loop = groups[2].events.find((e) => e.kind === 'loop')
    expect(loop?.jumpTime).toBe(0)
  })

  /** stop「类似无限大的时间标签…随时可触发中断」——不是结束 */
  it('识别无限等待', () => {
    const { groups, hasHalt } = analyzeTimeline(STD, 'std')
    expect(hasHalt).toBe(true)
    expect(groups[2].events.some((e) => e.kind === 'halt')).toBe(true)
  })
})

describe('插值非阻塞', () => {
  /**
   * thpages 对 stop 的原文：「This is basically like putting an infinitely large
   * time label... **Time-interpolated values will update**」——脚本都停了插值还在跑，
   * 所以插值只是启动一个由帧时钟驱动的过程，不卡住脚本。
   *
   * 后果：插值会跨过后续的时间标签。这一点光看文本发现不了，必须由视图指出。
   */
  it('给出插值的结束时间', () => {
    const { groups } = analyzeTimeline('0:\n  posTime(60, 0, 1f, 2f, 3f)', 'std')
    const span = groups[0].events[0]
    expect(span.endTime).toBe(60)
  })

  it('标出越过后续时间标签的插值', () => {
    const text = [
      '0:',
      '  posTime(100, 0, 1f, 2f, 3f)', // 0 → 100，越过了 30
      '30:',
      '  pos(0f, 0f, 0f)'
    ].join('\n')
    const { groups } = analyzeTimeline(text, 'std')
    expect(groups[0].events[0].crossesNextLabel).toBe(true)
  })

  it('恰好在下一个时间点结束的不算越过', () => {
    const text = ['0:', '  posTime(30, 0, 1f, 2f, 3f)', '30:', '  pos(0f, 0f, 0f)'].join('\n')
    const { groups } = analyzeTimeline(text, 'std')
    expect(groups[0].events[0].crossesNextLabel).toBe(false)
  })

  it('最后一个时间点上的插值不判越过（后面没有标签了）', () => {
    const { groups } = analyzeTimeline('0:\n  posTime(999, 0, 1f, 2f, 3f)', 'std')
    expect(groups[0].events[0].crossesNextLabel).toBe(false)
  })

  /** 非阻塞意味着可以重叠——两条插值在同一时间点启动，各自独立跑 */
  it('同一时间点的多条插值各自独立', () => {
    const text = ['0:', '  posTime(60, 0, 1f, 2f, 3f)', '  fogTime(30, 0, 1f, 2f, 3f)'].join('\n')
    const { groups } = analyzeTimeline(text, 'std')
    expect(groups[0].events.map((e) => e.endTime)).toEqual([60, 30])
  })
})

describe('时间标签', () => {
  it('相对标签累加', () => {
    const { groups } = analyzeTimeline('10:\n  pos(0f)\n+30:\n  pos(1f)', 'std')
    expect(groups.map((g) => g.time)).toEqual([10, 40])
  })

  it('符号标签不推进时间', () => {
    const { groups } = analyzeTimeline('20:\n  pos(0f)\n@sub:\n  pos(1f)', 'std')
    expect(groups).toHaveLength(1)
    expect(groups[0].time).toBe(20)
  })

  it('同一时间点的指令合并成一组', () => {
    const { groups } = analyzeTimeline('0:\n  pos(0f)\n0:\n  fov(1f)', 'std')
    expect(groups).toHaveLength(1)
    expect(groups[0].events).toHaveLength(2)
  })

  it('没有时间标签时默认时间 0', () => {
    const { groups } = analyzeTimeline('  pos(0f)', 'std')
    expect(groups[0].time).toBe(0)
  })

  it('注释与空行不产生事件', () => {
    const { groups } = analyzeTimeline('// 注释\n\n# 方言头\n0:\n  pos(0f)', 'std')
    expect(groups).toHaveLength(1)
    expect(groups[0].events).toHaveLength(1)
  })
})

describe('MSG 方言', () => {
  const MSG = ['0:', '    textboxShow(0)', '30:', '    textAdd("hi")'].join('\n')

  it('同样按时间分组', () => {
    const { groups } = analyzeTimeline(MSG, 'msg')
    expect(groups.map((g) => g.time)).toEqual([0, 30])
    expect(groups[0].delta).toBe(30)
  })

  /** MSG 没有 jmp / 中断 / 插值区间，全是瞬时事件 */
  it('全部是瞬时事件', () => {
    const { groups, hasHalt } = analyzeTimeline(MSG, 'msg')
    expect(groups.every((g) => g.events.every((e) => e.kind === 'instant'))).toBe(true)
    expect(hasHalt).toBe(false)
  })

  /** STD 的指令名不该在 MSG 里被当成特殊语义 */
  it('不把 STD 的特殊指令名套用到 MSG', () => {
    const { groups } = analyzeTimeline('0:\n    jmp(0, 20)', 'msg')
    expect(groups[0].events[0].kind).toBe('instant')
  })
})

describe('dialectForPath', () => {
  it('按扩展名判断方言', () => {
    expect(dialectForPath('/p/st01.dstd')).toBe('std')
    expect(dialectForPath('/p/st01.dmsg')).toBe('msg')
  })

  it('非时间线格式返回 null', () => {
    for (const p of ['/p/a.decl', '/p/a.ecl', '/p/a.txt', '', null, undefined]) {
      expect(dialectForPath(p)).toBeNull()
    }
  })
})
