import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProjectStore } from '../../src/stores/project'
import { useRecentProjectsStore } from '../../src/stores/recentProjects'
import { useWorkbenchReportsStore } from '../../src/stores/workbenchReports'
import {
  listRecentProjects,
  loadProjectConfig,
  openProject,
  removeRecentProject,
  saveProjectConfig
} from '../../src/api'

vi.mock('../../src/api', () => ({
  openProject: vi.fn(),
  getFileTree: vi.fn().mockResolvedValue([]),
  getDirChildren: vi.fn().mockResolvedValue([]),
  loadProjectConfig: vi.fn(),
  saveProjectConfig: vi.fn().mockResolvedValue(undefined),
  listRecentProjects: vi.fn(),
  removeRecentProject: vi.fn(),
  clearRecentProjects: vi.fn().mockResolvedValue(undefined)
}))

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('project store 的配置三态', () => {
  it('loadProject 一次性提交 rootPath / files / 配置状态', async () => {
    openProject.mockResolvedValue({
      rootPath: '/proj',
      files: [{ name: 'a.decl', path: '/proj/a.decl' }],
      projectConfig: {
        status: 'loaded',
        value: { gameVersion: '18', encoding: 'utf-8', mapPaths: [], toolchain: { thtkDir: '' } },
        error: null,
        path: '/proj/.thtk-project.json'
      }
    })
    const store = useProjectStore()

    await store.loadProject('/proj')

    expect(store.rootPath).toBe('/proj')
    expect(store.files).toHaveLength(1)
    expect(store.hasProjectConfig).toBe(true)
    expect(store.gameVersion).toBe('18')
  })

  it('打开失败时不改动任何状态，并把错误抛给调用方', async () => {
    openProject.mockResolvedValueOnce({
      rootPath: '/proj',
      files: [],
      projectConfig: { status: 'absent', value: null, error: null, path: '' }
    })
    const store = useProjectStore()
    await store.loadProject('/proj')

    openProject.mockRejectedValueOnce('目录不存在')
    await expect(store.loadProject('/gone')).rejects.toBe('目录不存在')

    expect(store.rootPath).toBe('/proj')
    expect(store.isLoading).toBe(false)
  })

  it('配置损坏时不伪装成"还没有配置"', async () => {
    // 这两个状态必须可区分：把损坏当成 absent，保存动作就会静默覆盖用户手写的内容
    openProject.mockResolvedValue({
      rootPath: '/proj',
      files: [],
      projectConfig: {
        status: 'invalid',
        value: null,
        error: '无法识别的字段 "mapPath"',
        path: '/proj/.thtk-project.json'
      }
    })
    const store = useProjectStore()

    await store.loadProject('/proj')

    expect(store.hasInvalidProjectConfig).toBe(true)
    expect(store.hasProjectConfig).toBe(false)
    expect(store.projectConfigPath).toBe('/proj/.thtk-project.json')
    expect(store.projectConfigError).toContain('mapPath')
  })

  it('saveConfig 把编辑时的项目根一起发给后端', async () => {
    const store = useProjectStore()
    store.rootPath = '/proj/current'

    await store.saveConfig({ gameVersion: '18' }, '/proj/editing')

    expect(saveProjectConfig).toHaveBeenCalledWith({ gameVersion: '18' }, '/proj/editing')
    expect(store.projectConfigStatus).toBe('loaded')
  })

  it('命令本身失败时状态记为 invalid 而不是 absent', async () => {
    loadProjectConfig.mockRejectedValue('No project root set')
    const store = useProjectStore()

    await store.reloadProjectConfig()

    expect(store.projectConfigStatus).toBe('invalid')
    expect(store.hasProjectConfig).toBe(false)
  })
})

describe('recentProjects store', () => {
  it('读取失败时保留上一次的列表，只记录错误', async () => {
    listRecentProjects.mockResolvedValueOnce([
      { path: '/a', name: 'a', lastOpenedAt: 1, available: true }
    ])
    const store = useRecentProjectsStore()
    await store.refresh()

    listRecentProjects.mockRejectedValueOnce('读不出来')
    await store.refresh()

    // 清空会让用户以为记录丢了
    expect(store.items).toHaveLength(1)
    expect(store.error).toContain('读不出来')
  })

  it('移除后用后端返回的列表替换本地缓存', async () => {
    listRecentProjects.mockResolvedValue([
      { path: '/a', name: 'a', lastOpenedAt: 2, available: true },
      { path: '/b', name: 'b', lastOpenedAt: 1, available: false }
    ])
    const store = useRecentProjectsStore()
    await store.refresh()

    removeRecentProject.mockResolvedValue([
      { path: '/a', name: 'a', lastOpenedAt: 2, available: true }
    ])
    await store.remove('/b')

    expect(store.items).toHaveLength(1)
    expect(store.items[0].path).toBe('/a')
    expect(store.error).toBeNull()
  })

  it('失效路径仍然留在列表里，只是标记不可用', async () => {
    listRecentProjects.mockResolvedValue([
      { path: '/gone', name: 'gone', lastOpenedAt: 1, available: false }
    ])
    const store = useRecentProjectsStore()

    await store.refresh()

    expect(store.items).toHaveLength(1)
    expect(store.items[0].available).toBe(false)
  })
})

describe('workbenchReports 容量上限', () => {
  it('输出条目超限时淘汰最旧的', async () => {
    const store = useWorkbenchReportsStore()

    for (let index = 0; index < 5200; index += 1) {
      store.pushOutputEntry({ ownerKey: 'k', text: `line-${index}` })
    }

    expect(store.outputEntries.length).toBeLessThanOrEqual(5000)
    // 保留的是最新的那一批
    expect(store.outputEntries.at(-1).text).toBe('line-5199')
    expect(store.outputEntries.some(entry => entry.text === 'line-0')).toBe(false)
  })

  it('单次替换超过上限时也不会无限增长', async () => {
    const store = useWorkbenchReportsStore()
    const entries = Array.from({ length: 6000 }, (_, index) => ({ text: `x-${index}` }))

    store.replaceOutput('owner', entries)

    expect(store.outputEntries.length).toBeLessThanOrEqual(5000)
  })

  it('问题条目同样有上限', async () => {
    const store = useWorkbenchReportsStore()
    const problems = Array.from({ length: 2500 }, (_, index) => ({
      path: `/f${index}.decl`,
      line: 1,
      column: 1,
      severity: 'error',
      message: `e-${index}`
    }))

    store.replaceProblems('owner', problems)

    expect(store.problemEntries.length).toBeLessThanOrEqual(2000)
  })

  it('淘汰不会让输出分组变成空组', async () => {
    const store = useWorkbenchReportsStore()
    for (let index = 0; index < 5200; index += 1) {
      store.pushOutputEntry({ ownerKey: `owner-${index % 3}`, text: `line-${index}` })
    }

    for (const group of store.outputGroups) {
      expect(group.lines.length).toBeGreaterThan(0)
    }
  })
})
