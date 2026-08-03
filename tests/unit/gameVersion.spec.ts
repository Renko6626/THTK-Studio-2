import { describe, it, expect } from 'vitest'
import { versionsForTool, formatVersionLabel } from '../../src/services/toolchains/gameVersions'
import type { GameVersionView } from '../../src/types'

/**
 * 夹具刻意选了三种典型形态：五工具全支持的正作、只有 thdat 的格斗作、
 * 以及 thmsg 不支持的 Uwabami Breakers。
 */
const TABLE: GameVersionView[] = [
  { id: 18, code: 'th18', title: '東方虹龍洞', tools: ['thecl', 'thmsg', 'thdat'] },
  { id: 75, code: 'th75', title: '東方萃夢想', tools: ['thdat'] },
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
    expect(formatVersionLabel(TABLE[0])).toBe('th18 · 東方虹龍洞')
  })
})
