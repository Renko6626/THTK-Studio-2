import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  loadDefaultEclSemanticData,
  detectEclmapVersionMismatch
} from '../../src/services/languages/ecl/semantic-loader'
import { getEclMapSemantics as rawGetSemantics, getSettings as rawGetSettings, getDependencyStatus as rawGetDependencyStatus } from '../../src/api'

const getEclMapSemantics = vi.mocked(rawGetSemantics)
const getSettings = vi.mocked(rawGetSettings)
const getDependencyStatus = vi.mocked(rawGetDependencyStatus)

import type { DependencyStatus, EclMapSemanticData } from '../../src/types'
import { makeAppConfig, makeProjectConfig } from '../helpers/fixtures'

vi.mock('../../src/api', () => ({
  getSettings: vi.fn(),
  getEclMapSemantics: vi.fn(),
  getDependencyStatus: vi.fn()
}))

function dependencyStatusWithBundledDir(bundledDir: string): DependencyStatus {
  return {
    thtk: {
      source: 'none',
      configured: false,
      available: false,
      thtkDir: '',
      resolvedPath: '',
      message: '',
      license: ''
    },
    maps: {
      source: 'bundled',
      bundledDir,
      coveredVersions: [],
      missingVersions: [],
      sourceCommit: '',
      license: 'Unlicense'
    },
    warnings: []
  }
}

function semanticsWith(names: string[]): EclMapSemanticData {
  return {
    version: '18',
    sourcePath: '',
    instructions: names.map((name: string, index: number) => ({
      name,
      opcode: index,
      section: null,
      signature: null,
      params: []
    })),
    builtins: [],
    globals: []
  }
}

describe('ECL 词表加载', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    getSettings.mockResolvedValue(
      makeAppConfig({
        default_game_version: '20',
        eclmap_path: '/global/th20.eclm',
        thtk_dir: '/global/thtk'
      })
    )
    // 默认不带托管目录，保持既有用例的候选集合不变
    getDependencyStatus.mockResolvedValue(dependencyStatusWithBundledDir(''))
  })

  it('合并项目声明的全部 map，而不是只取第一条', async () => {
    // thecl 和 mcp 侧都是把整个 mapPaths 传下去的；只取 [0] 会让后面的 map
    // 里定义的指令在补全 / 悬停里查无此项。
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/proj/maps/base.eclm') return semanticsWith(['ins_a', 'ins_b'])
      if (path === '/proj/maps/extra.eclm') return semanticsWith(['ins_c'])
      throw new Error(`unexpected ${path}`)
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['maps/base.eclm', 'maps/extra.eclm'] })
    })

    expect(result.instructions.map(i => i.name).sort()).toEqual(['ins_a', 'ins_b', 'ins_c'])
    expect(result.resolvedPath).toContain('base.eclm')
    expect(result.resolvedPath).toContain('extra.eclm')
  })

  it('显式项目 map 失效时报告错误，不回落到全局配置', async () => {
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/global/th20.eclm') return semanticsWith(['global_ins'])
      throw new Error(`missing ${path}`)
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['maps/gone.eclm'] })
    })

    expect(result.instructions).toEqual([])
    expect(result.error).toContain('/proj/maps/gone.eclm')
    expect(getEclMapSemantics).not.toHaveBeenCalledWith('/global/th20.eclm')
  })

  it('项目版本优先于全局默认版本', async () => {
    getEclMapSemantics.mockResolvedValue({ ...semanticsWith(['x']), version: '' })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ gameVersion: '18', mapPaths: ['maps/a.eclm'] })
    })

    expect(result.version).toBe('18')
  })

  /**
   * 契约变更：归一化已从前端移到后端。
   *
   * 此前这里喂 'th18' 期望前端剥掉前缀——那是第三份归一化实现。现在
   * project_config::canonicalize_game_version 与 config::canonicalize_version
   * 在加载/保存时就把值规范成纯数字，前端拿到的必然已规范。
   *
   * 这条测试把该契约钉住：万一后端漏了归一，前端会**原样透传**而不是
   * 悄悄补救——问题会暴露在候选路径 thth18.eclm 上，而不是被掩盖。
   */
  it('不再二次归一：前端原样透传后端给的版本值', async () => {
    getEclMapSemantics.mockResolvedValue({ ...semanticsWith(['x']), version: '' })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ gameVersion: 'th18', mapPaths: ['maps/a.eclm'] })
    })

    expect(result.version).toBe('th18')
  })

  describe('eclmap 版本一致性', () => {
    it('文件名版本与项目版本不符时给出提示', () => {
      const warning = detectEclmapVersionMismatch(['maps/th18.eclm'], '20')
      expect(warning).toContain('th18')
      expect(warning).toContain('th20')
    })

    it('相符时无提示', () => {
      expect(detectEclmapVersionMismatch(['maps/th20.eclm'], '20')).toBeNull()
    })

    it('文件名不含版本时不误报', () => {
      expect(detectEclmapVersionMismatch(['maps/custom.eclm'], '20')).toBeNull()
    })

    /** 多份 map 是并列关系，只要有一个对得上就认为配置是有意的 */
    it('多个 eclmap 只要有一个相符就不提示', () => {
      expect(
        detectEclmapVersionMismatch(['maps/th20.eclm', 'maps/th18.eclm'], '20')
      ).toBeNull()
    })

    it('未选版本或无 map 时不提示', () => {
      expect(detectEclmapVersionMismatch(['maps/th18.eclm'], '')).toBeNull()
      expect(detectEclmapVersionMismatch([], '20')).toBeNull()
    })

    it('认 Windows 反斜杠路径', () => {
      expect(detectEclmapVersionMismatch(['maps\\th18.eclm'], '20')).toContain('th18')
    })
  })

  it('项目相对路径按项目根解析', async () => {
    getEclMapSemantics.mockResolvedValue(semanticsWith(['x']))

    await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['maps/a.eclm'] })
    })

    expect(getEclMapSemantics).toHaveBeenCalledWith('/proj/maps/a.eclm')
  })

  it('项目绝对路径原样使用', async () => {
    getEclMapSemantics.mockResolvedValue(semanticsWith(['x']))

    await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['/abs/a.eclm'] })
    })

    expect(getEclMapSemantics).toHaveBeenCalledWith('/abs/a.eclm')
  })

  it('未显式配置时忽略项目目录的同名 map，使用内置基线', async () => {
    getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '20' }))
    getDependencyStatus.mockResolvedValue(dependencyStatusWithBundledDir('/bundled'))
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/bundled/th20.eclm') return semanticsWith(['bundled_ins'])
      if (path.startsWith('/proj/')) return semanticsWith(['project_ins'])
      throw new Error(`missing ${path}`)
    })
    const result = await loadDefaultEclSemanticData({ projectRoot: '/proj' })
    expect(result.instructions.map(i => i.name)).toEqual(['bundled_ins'])
  })

  it('项目 map 部分失效时不发布残缺词表', async () => {
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/proj/base.eclm') return semanticsWith(['base_ins'])
      throw new Error('file missing')
    })
    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['base.eclm', 'missing.eclm'] })
    })
    expect(result.instructions).toEqual([])
    expect(result.error).toContain('/proj/missing.eclm')
  })

  it('全局显式 map 失效时不改用可用的内置基线', async () => {
    getDependencyStatus.mockResolvedValue(dependencyStatusWithBundledDir('/bundled'))
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/bundled/th20.eclm') return semanticsWith(['bundled_ins'])
      throw new Error('file missing')
    })
    const result = await loadDefaultEclSemanticData({})
    expect(result.instructions).toEqual([])
    expect(result.error).toContain('/global/th20.eclm')
  })

  it('项目 thtkDir 覆盖全局目录，缺失时不混用全局 map', async () => {
    getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '20', thtk_dir: '/global/thtk' }))
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/global/thtk/maps/th20.eclm') return semanticsWith(['global_ins'])
      throw new Error('file missing')
    })
    const result = await loadDefaultEclSemanticData({
      projectConfig: makeProjectConfig({ toolchain: { thtkDir: '/project/thtk', truthDir: '' } })
    })
    expect(result.instructions).toEqual([])
  })

  it('没有任何 map 可用时返回空词表并带上最后一次错误', async () => {
    getEclMapSemantics.mockRejectedValue(new Error('nope'))
    getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '20', thtk_dir: '/thtk' }))

    const result = await loadDefaultEclSemanticData({ projectRoot: '/proj', projectConfig: null })

    expect(result.instructions).toEqual([])
    expect(result.error).toContain('nope')
  })

  it('合并多份 map 时保留 builtins', () => {
    // 回归防线：builtins 是 string[]，曾被喂给按 name 去重的 dedupeByName，
    // 每个字符串取 .name 都是 undefined，结果整个数组被清空——
    // 内置函数（sin/cos 等）在补全与高亮里全部消失。
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/proj/a.eclm') {
        return { ...semanticsWith(['ins_a']), builtins: ['sin', 'cos'] }
      }
      return { ...semanticsWith(['ins_b']), builtins: ['cos', 'tan'] }
    })

    return loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['a.eclm', 'b.eclm'] })
    }).then((result) => {
      expect([...result.builtins].sort()).toEqual(['cos', 'sin', 'tan'])
    })
  })

  it('单份 map 的 builtins 也要保留', async () => {
    getEclMapSemantics.mockResolvedValue({
      ...semanticsWith(['ins_a']),
      builtins: ['sin']
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['a.eclm'] })
    })

    expect(result.builtins).toEqual(['sin'])
  })

  it('同名指令后加载的覆盖先加载的', async () => {
    getEclMapSemantics.mockImplementation(async (path) => {
      const opcode = path === '/proj/a.eclm' ? 1 : 99
      return { ...semanticsWith([]), instructions: [{ name: 'dup', opcode, section: null, signature: null, params: [] }] }
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['a.eclm', 'b.eclm'] })
    })

    expect(result.instructions).toHaveLength(1)
    expect(result.instructions[0].opcode).toBe(99)
  })

  describe('IDE 托管 map', () => {
    beforeEach(() => {
      // 去掉全局 eclmap，才能观察到托管目录这一级
      getSettings.mockResolvedValue(
        makeAppConfig({ default_game_version: '20', thtk_dir: '/global/thtk' })
      )
    })

    it('使用后端返回的 bundledDir，且排在 thtk_dir 推导之前', async () => {
      getDependencyStatus.mockResolvedValue(
        dependencyStatusWithBundledDir('/data/deps/maps/bundled')
      )
      getEclMapSemantics.mockImplementation(async (path) => {
        if (path === '/data/deps/maps/bundled/th20.eclm') return semanticsWith(['bundled_ins'])
        if (path === '/global/thtk/maps/th20.eclm') return semanticsWith(['thtk_ins'])
        throw new Error(`missing ${path}`)
      })

      const result = await loadDefaultEclSemanticData({ projectRoot: null, projectConfig: null })

      expect(result.instructions.map(i => i.name)).toEqual(['bundled_ins'])
      expect(getEclMapSemantics).not.toHaveBeenCalledWith('/global/thtk/maps/th20.eclm')
    })

    it('补零命名的基线（th06 / th095）也能命中', async () => {
      getDependencyStatus.mockResolvedValue(
        dependencyStatusWithBundledDir('/data/deps/maps/bundled')
      )
      getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '6' }))
      getEclMapSemantics.mockImplementation(async (path) => {
        if (path === '/data/deps/maps/bundled/th06.eclm') return semanticsWith(['th06_ins'])
        throw new Error(`missing ${path}`)
      })

      const result = await loadDefaultEclSemanticData({ projectRoot: null, projectConfig: null })

      expect(result.instructions.map(i => i.name)).toEqual(['th06_ins'])
    })

    it('th095 只命中规范名 th095.eclm，不生成 095.eclm / 095.eclmap', async () => {
      getDependencyStatus.mockResolvedValue(
        dependencyStatusWithBundledDir('/data/deps/maps/bundled')
      )
      getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '95' }))
      getEclMapSemantics.mockImplementation(async (path) => {
        if (path === '/data/deps/maps/bundled/th095.eclm') return semanticsWith(['th095_ins'])
        throw new Error(`missing ${path}`)
      })

      const result = await loadDefaultEclSemanticData({ projectRoot: null, projectConfig: null })

      expect(result.instructions.map(i => i.name)).toEqual(['th095_ins'])
      const attempted = getEclMapSemantics.mock.calls.map(([path]) => path)
      // 与 Rust map_file_names 一致：95 的 stem 只有 "95"，补零形式仅来自规范名
      expect(attempted).toContain('/data/deps/maps/bundled/th095.eclm')
      expect(attempted).not.toContain('/data/deps/maps/bundled/095.eclm')
      expect(attempted).not.toContain('/data/deps/maps/bundled/095.eclmap')
    })

    it('thtkDir 根级同名 map 不参与候选，只查 maps/ 子目录', async () => {
      getDependencyStatus.mockResolvedValue(dependencyStatusWithBundledDir(''))
      getEclMapSemantics.mockImplementation(async (path) => {
        if (path === '/global/thtk/th20.eclm') return semanticsWith(['root_level_ins'])
        if (path === '/global/thtk/maps/th20.eclm') return semanticsWith(['maps_ins'])
        throw new Error(`missing ${path}`)
      })

      const result = await loadDefaultEclSemanticData({ projectRoot: null, projectConfig: null })

      expect(result.instructions.map(i => i.name)).toEqual(['maps_ins'])
      expect(getEclMapSemantics).not.toHaveBeenCalledWith('/global/thtk/th20.eclm')
    })

    it('依赖状态不可用时仍回落到 thtk_dir 推导', async () => {
      getDependencyStatus.mockRejectedValue(new Error('command not available'))
      getEclMapSemantics.mockImplementation(async (path) => {
        if (path === '/global/thtk/maps/th20.eclm') return semanticsWith(['thtk_ins'])
        throw new Error(`missing ${path}`)
      })

      const result = await loadDefaultEclSemanticData({ projectRoot: null, projectConfig: null })

      expect(result.instructions.map(i => i.name)).toEqual(['thtk_ins'])
    })
  })
})
