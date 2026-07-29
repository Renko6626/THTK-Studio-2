import { beforeEach, describe, expect, it, vi } from 'vitest'
import { loadDefaultEclSemanticData } from '../../src/services/languages/ecl/semantic-loader'
import { getEclMapSemantics, getSettings } from '../../src/api'

vi.mock('../../src/api', () => ({
  getSettings: vi.fn(),
  getEclMapSemantics: vi.fn()
}))

function semanticsWith(names) {
  return {
    version: '18',
    sourcePath: '',
    instructions: names.map((name, index) => ({ name, opcode: index })),
    builtins: []
  }
}

describe('ECL 词表加载', () => {
  beforeEach(() => {
    getSettings.mockResolvedValue({
      default_game_version: '20',
      eclmap_path: '/global/th20.eclm',
      thtk_dir: '/global/thtk'
    })
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
      projectConfig: { mapPaths: ['maps/base.eclm', 'maps/extra.eclm'] }
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
      projectConfig: { mapPaths: ['maps/gone.eclm'] }
    })

    expect(result.instructions.map(i => i.name)).toEqual(['global_ins'])
  })

  it('项目版本优先于全局默认版本', async () => {
    getEclMapSemantics.mockResolvedValue({ ...semanticsWith(['x']), version: '' })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: { gameVersion: 'th18', mapPaths: ['maps/a.eclm'] }
    })

    expect(result.version).toBe('18')
  })

  it('项目相对路径按项目根解析', async () => {
    getEclMapSemantics.mockResolvedValue(semanticsWith(['x']))

    await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: { mapPaths: ['maps/a.eclm'] }
    })

    expect(getEclMapSemantics).toHaveBeenCalledWith('/proj/maps/a.eclm')
  })

  it('项目绝对路径原样使用', async () => {
    getEclMapSemantics.mockResolvedValue(semanticsWith(['x']))

    await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: { mapPaths: ['/abs/a.eclm'] }
    })

    expect(getEclMapSemantics).toHaveBeenCalledWith('/abs/a.eclm')
  })

  it('没有任何 map 可用时返回空词表并带上最后一次错误', async () => {
    getEclMapSemantics.mockRejectedValue(new Error('nope'))
    getSettings.mockResolvedValue({ default_game_version: '20', eclmap_path: '', thtk_dir: '' })

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
      projectConfig: { mapPaths: ['a.eclm', 'b.eclm'] }
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
      projectConfig: { mapPaths: ['a.eclm'] }
    })

    expect(result.builtins).toEqual(['sin'])
  })

  it('同名指令后加载的覆盖先加载的', async () => {
    getEclMapSemantics.mockImplementation(async (path) => {
      if (path === '/proj/a.eclm') return { instructions: [{ name: 'dup', opcode: 1 }], builtins: [] }
      return { instructions: [{ name: 'dup', opcode: 99 }], builtins: [] }
    })

    const result = await loadDefaultEclSemanticData({
      projectRoot: '/proj',
      projectConfig: { mapPaths: ['a.eclm', 'b.eclm'] }
    })

    expect(result.instructions).toHaveLength(1)
    expect(result.instructions[0].opcode).toBe(99)
  })
})
