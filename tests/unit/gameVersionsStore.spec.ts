import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { listGameVersions as rawListGameVersions } from '../../src/api'
import { useGameVersionsStore } from '../../src/stores/gameVersions'
import type { GameVersionView } from '../../src/types'

const listGameVersions = vi.mocked(rawListGameVersions)

vi.mock('../../src/api', () => ({
  listGameVersions: vi.fn()
}))

const TABLE: GameVersionView[] = [
  { id: 18, code: 'th18', title: '東方虹龍洞', tools: ['thecl', 'thdat'] },
  { id: 75, code: 'th75', title: '東方萃夢想', tools: ['thdat'] }
]

describe('gameVersions store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    listGameVersions.mockReset()
    listGameVersions.mockResolvedValue(TABLE)
  })

  it('只拉取一次', async () => {
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    await store.ensureLoaded()
    expect(listGameVersions).toHaveBeenCalledTimes(1)
  })

  it('并发调用也只拉取一次', async () => {
    const store = useGameVersionsStore()
    await Promise.all([store.ensureLoaded(), store.ensureLoaded()])
    expect(listGameVersions).toHaveBeenCalledTimes(1)
    expect(store.table).toHaveLength(2)
  })

  it('按工具给出下拉选项', async () => {
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    expect(store.optionsForTool('thecl')).toEqual([
      { label: 'th18 · 東方虹龍洞', value: '18' }
    ])
    expect(store.optionsForTool('thdat')).toHaveLength(2)
  })

  it('加载失败时不抛出，选项为空并留下 error', async () => {
    listGameVersions.mockRejectedValue(new Error('boom'))
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    expect(store.optionsForTool('thecl')).toEqual([])
    expect(store.error).toContain('boom')
  })

  it('加载失败后可重试', async () => {
    listGameVersions.mockRejectedValueOnce(new Error('boom'))
    const store = useGameVersionsStore()
    await store.ensureLoaded()
    expect(store.table).toEqual([])

    await store.ensureLoaded()
    expect(store.table).toHaveLength(2)
    expect(store.error).toBe('')
  })
})
