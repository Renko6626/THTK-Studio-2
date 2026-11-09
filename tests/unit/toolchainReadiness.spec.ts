import { describe, expect, it } from 'vitest'
import { summarizeToolchainReadiness } from '../../src/services/toolchains/readiness'
import type { ToolchainStatus } from '../../src/types'

function status(tool: string, available: boolean): ToolchainStatus {
  return {
    tool,
    label: tool,
    exeName: `${tool}.exe`,
    configuredPath: '',
    resolvedPath: available ? `/thtk/${tool}.exe` : '',
    available,
    version: available ? '1.0' : '',
    message: '',
    supportedVersions: []
  }
}

const ALL = ['thecl', 'trumsg', 'thanm', 'trustd', 'thdat']

describe('summarizeToolchainReadiness', () => {
  it('全部可用时不提示', () => {
    const result = summarizeToolchainReadiness(ALL.map((t) => status(t, true)))
    expect(result.state).toBe('ready')
    expect(result.message).toBe('')
    expect(result.missing).toEqual([])
  })

  /** 安装损坏或配置指向无效目录时，说明两类工具的来源。 */
  it('全部不可用时说明 truth 随包提供、thtk 需配置', () => {
    const result = summarizeToolchainReadiness(ALL.map((t) => status(t, false)))
    expect(result.state).toBe('missing')
    expect(result.message).toContain('thtk')
    expect(result.message).toContain('truth 随安装包提供')
    expect(result.missing).toEqual(ALL)
  })

  it('部分缺失时点名缺哪几个', () => {
    const result = summarizeToolchainReadiness([
      status('thecl', true),
      status('trumsg', false),
      status('thanm', false),
      status('trustd', true),
      status('thdat', true)
    ])
    expect(result.state).toBe('partial')
    expect(result.message).toContain('trumsg')
    expect(result.message).toContain('thanm')
    expect(result.missing).toEqual(['trumsg', 'thanm'])
  })

  /**
   * 状态尚未取到时不能报警——否则应用启动瞬间会闪一条"未配置"的假警报，
   * 比不提示更糟。
   */
  it('状态为空时按就绪处理，不闪假警报', () => {
    expect(summarizeToolchainReadiness([]).state).toBe('ready')
  })
})
