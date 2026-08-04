import { describe, it, expect } from 'vitest'
import {
  versionsForTool,
  formatVersionLabel,
  toolAvailability
} from '../../src/services/toolchains/gameVersions'
import type { GameVersionView } from '../../src/types'

/**
 * 夹具刻意选了三种典型形态：五工具全支持的正作、只有 thdat 的格斗作、
 * 以及 thmsg 不支持的 Uwabami Breakers。
 */
const TABLE: GameVersionView[] = [
  { id: 18, code: 'th18', title: '东方虹龙洞', tools: ['thecl', 'thmsg', 'thdat'] },
  { id: 75, code: 'th75', title: '东方萃梦想', tools: ['thdat'] },
  { id: 103, code: 'th103', title: 'Uwabami Breakers（非东方）', tools: ['thecl', 'thdat'] }
]

describe('gameVersions', () => {
  it('按工具过滤版本', () => {
    expect(versionsForTool(TABLE, 'thecl').map((v) => v.id)).toEqual([18, 103])
    expect(versionsForTool(TABLE, 'thdat').map((v) => v.id)).toEqual([18, 75, 103])
    expect(versionsForTool(TABLE, 'thmsg').map((v) => v.id)).toEqual([18])
  })

  it('未知工具返回空列表而不是全量', () => {
    expect(versionsForTool(TABLE, 'thbogus')).toEqual([])
  })

  it('保持表内顺序，不重排', () => {
    expect(versionsForTool(TABLE, 'thdat').map((v) => v.code)).toEqual([
      'th18',
      'th75',
      'th103'
    ])
  })

  it('标签同时给出版本号与标题', () => {
    expect(formatVersionLabel(TABLE[0])).toBe('th18 · 东方虹龙洞')
  })
})

describe('toolAvailability', () => {
  it('版本受支持时启用', () => {
    expect(toolAvailability('thdat', '75', TABLE).enabled).toBe(true)
    expect(toolAvailability('thecl', '18', TABLE).enabled).toBe(true)
  })

  it('版本不被该工具支持时禁用并说明原因', () => {
    const availability = toolAvailability('thecl', '75', TABLE)
    expect(availability.enabled).toBe(false)
    expect(availability.reason).toContain('东方萃梦想')
    expect(availability.reason).toContain('thdat')
  })

  it('接受 th 前缀写法（旧配置可能还没被归一）', () => {
    expect(toolAvailability('thecl', 'th75', TABLE).enabled).toBe(false)
  })

  /**
   * 以下三种"信息不足"的情形一律放行。
   * 宁可让用户点了拿后端的明确报错，也不要因为前端状态没就绪就把功能灰掉——
   * 灰掉的按钮不会告诉用户为什么，而后端的报错会。
   */
  it('版本表未加载时不误禁用', () => {
    expect(toolAvailability('thecl', '18', []).enabled).toBe(true)
  })

  it('未选版本时不误禁用', () => {
    expect(toolAvailability('thecl', '', TABLE).enabled).toBe(true)
    expect(toolAvailability('thecl', null, TABLE).enabled).toBe(true)
  })

  it('版本不在表内时不误禁用', () => {
    expect(toolAvailability('thecl', '999', TABLE).enabled).toBe(true)
  })
})
