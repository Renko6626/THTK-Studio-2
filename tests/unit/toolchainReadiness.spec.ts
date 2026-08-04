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

const ALL = ['thecl', 'thmsg', 'thanm', 'thstd', 'thdat']

describe('summarizeToolchainReadiness', () => {
  it('全部可用时不提示', () => {
    const result = summarizeToolchainReadiness(ALL.map((t) => status(t, true)))
    expect(result.state).toBe('ready')
    expect(result.message).toBe('')
    expect(result.missing).toEqual([])
  })

  /** 全新安装的默认状态：thtk_dir 为空，五个工具全不可用 */
  it('全部不可用时给出"尚未配置"并说明 thtk 需另行下载', () => {
    const result = summarizeToolchainReadiness(ALL.map((t) => status(t, false)))
    expect(result.state).toBe('missing')
    expect(result.message).toContain('thtk')
    expect(result.message).toContain('第三方')
    expect(result.missing).toEqual(ALL)
  })

  it('部分缺失时点名缺哪几个', () => {
    const result = summarizeToolchainReadiness([
      status('thecl', true),
      status('thmsg', false),
      status('thanm', false),
      status('thstd', true),
      status('thdat', true)
    ])
    expect(result.state).toBe('partial')
    expect(result.message).toContain('thmsg')
    expect(result.message).toContain('thanm')
    expect(result.missing).toEqual(['thmsg', 'thanm'])
  })

  /**
   * 状态尚未取到时不能报警——否则应用启动瞬间会闪一条"未配置"的假警报，
   * 比不提示更糟。
   */
  it('状态为空时按就绪处理，不闪假警报', () => {
    expect(summarizeToolchainReadiness([]).state).toBe('ready')
  })
})
