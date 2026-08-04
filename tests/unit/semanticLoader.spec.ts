import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  loadDefaultEclSemanticData,
  detectEclmapVersionMismatch
} from '../../src/services/languages/ecl/semantic-loader'
import { getEclMapSemantics as rawGetSemantics, getSettings as rawGetSettings } from '../../src/api'

const getEclMapSemantics = vi.mocked(rawGetSemantics)
const getSettings = vi.mocked(rawGetSettings)

import type { EclMapSemanticData } from '../../src/types'
import { makeAppConfig, makeProjectConfig } from '../helpers/fixtures'

vi.mock('../../src/api', () => ({
  getSettings: vi.fn(),
  getEclMapSemantics: vi.fn()
}))

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
    getSettings.mockResolvedValue(
      makeAppConfig({
        default_game_version: '20',
        eclmap_path: '/global/th20.eclm',
        thtk_dir: '/global/thtk'
      })
    )
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

  it('项目 map 全部加载失败时继续回落到全局配置', async () => {
    // 项目里一条过期路径不该把整个 ECL 语言服务打哑
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/global/th20.eclm') return semanticsWith(['global_ins'])
      throw new Error(`missing ${path}`)
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: makeProjectConfig({ mapPaths: ['maps/gone.eclm'] })
    })

    expect(result.instructions.map(i => i.name)).toEqual(['global_ins'])
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

  it('没有任何 map 可用时返回空词表并带上最后一次错误', async () => {
    getEclMapSemantics.mockRejectedValue(new Error('nope'))
    getSettings.mockResolvedValue(makeAppConfig({ default_game_version: '20' }))

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
})
