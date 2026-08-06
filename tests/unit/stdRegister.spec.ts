import { beforeEach, describe, expect, it } from 'vitest'

// monaco-editor 由 vitest.config.js 别名到 tests/stubs/ 的替身

import {
  __resetStdRegistryForTests,
  getStdGameVersion,
  setStdGameVersion
} from '../../src/services/languages/std/register'

describe('setStdGameVersion', () => {
  beforeEach(() => {
    __resetStdRegistryForTests()
  })

  it('接受数字与字符串两种写法', () => {
    setStdGameVersion(17)
    expect(getStdGameVersion()).toBe(17)
    setStdGameVersion('18')
    expect(getStdGameVersion()).toBe(18)
  })

  /** 项目配置在后端已归一成纯数字，但旧配置可能残留 th 前缀 */
  it('容忍 th 前缀', () => {
    setStdGameVersion('th20')
    expect(getStdGameVersion()).toBe(20)
    setStdGameVersion('TH14')
    expect(getStdGameVersion()).toBe(14)
  })

  /**
   * 关键：拿不到版本就是 null，**不给默认值**。
   * 不同版本的跳转 opcode 与偏移含义都不同（v0 存指令序号、v1+ 存字节偏移），
   * 随便填一个会让导航指向完全不相干的行——那比不导航糟糕得多。
   */
  it('拿不到版本时为 null，不猜默认值', () => {
    for (const bad of ['', '   ', null, undefined, 'abc', 0, -1]) {
      setStdGameVersion(bad as never)
      expect(getStdGameVersion()).toBeNull()
    }
  })

  it('项目切换后能被覆盖', () => {
    setStdGameVersion(17)
    setStdGameVersion(8)
    expect(getStdGameVersion()).toBe(8)
    setStdGameVersion(null)
    expect(getStdGameVersion()).toBeNull()
  })
})
