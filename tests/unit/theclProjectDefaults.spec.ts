import { describe, expect, it } from 'vitest'
import { applyProjectDefaults } from '../../src/composables/useTheclActions'
import type { ProjectConfig, TheclRequest } from '../../src/types'

const BASE: TheclRequest = {
  mode: 'compile',
  version: '',
  inputPath: '/proj/st01.decl',
  outputPath: null,
  mapPaths: [],
  useShiftJis: false,
  rawDump: false,
  simpleCreation: false,
  showOffsets: false
}

const CONFIG: ProjectConfig = {
  gameVersion: '18',
  encoding: 'shift-jis',
  mapPaths: ['maps/th18.eclm'],
  toolchain: { thtkDir: '' }
}

describe('applyProjectDefaults', () => {
  it('配置可用时填入版本、map 与编码', () => {
    const { request, warning } = applyProjectDefaults(BASE, CONFIG, 'loaded')
    expect(request.version).toBe('18')
    expect(request.mapPaths).toEqual(['maps/th18.eclm'])
    expect(request.useShiftJis).toBe(true)
    expect(warning).toBeNull()
  })

  /**
   * 回归：项目配置写 utf-8 时不能再传 -j 给 thecl。
   *
   * 原实现是 `request.useShiftJis ?? (encoding === 'shift-jis')`，而
   * createTheclRequest 把该字段默认成 true 且类型是非可选 boolean——`??`
   * 永远不会回落，于是项目的 encoding 设置对快捷菜单路径完全无效。
   */
  it('项目声明 utf-8 时不使用 Shift-JIS', () => {
    const { request } = applyProjectDefaults(
      { ...BASE, useShiftJis: true },
      { ...CONFIG, encoding: 'utf-8' },
      'loaded'
    )
    expect(request.useShiftJis).toBe(false)
  })

  it('项目声明 shift-jis 时使用 Shift-JIS', () => {
    const { request } = applyProjectDefaults({ ...BASE, useShiftJis: false }, CONFIG, 'loaded')
    expect(request.useShiftJis).toBe(true)
  })

  it('请求里已有的值优先于项目默认值', () => {
    const { request } = applyProjectDefaults(
      { ...BASE, version: '20', mapPaths: ['custom.eclm'] },
      CONFIG,
      'loaded'
    )
    expect(request.version).toBe('20')
    expect(request.mapPaths).toEqual(['custom.eclm'])
  })

  it('没有配置文件时安静地什么都不做', () => {
    const { request, warning } = applyProjectDefaults(BASE, null, 'absent')
    expect(request).toEqual(BASE)
    expect(warning).toBeNull()
  })

  /**
   * 这是本次修复的核心。
   *
   * 配置损坏时前端把 projectConfig 置为 null，于是 ECL 会**静默**丢掉
   * mapPaths——编译在没有 eclmap 的情况下进行，报错完全指不到真正的原因。
   * 而 msg/std/dat 走 Rust 的尽力而为加载器，照常拿到 gameVersion 与 thtkDir，
   * 两边行为并不一致。
   *
   * 不改变"损坏时不采用项目设置"这个决定（拿已知格式错误的数据去编译更糟），
   * 但必须让用户知道发生了什么。
   */
  it('配置损坏时不采用项目设置，并给出可操作的警告', () => {
    const { request, warning } = applyProjectDefaults(BASE, null, 'invalid')
    expect(request).toEqual(BASE)
    expect(warning).not.toBeNull()
    expect(warning).toContain('eclmap')
    expect(warning).toContain('项目设置')
  })

  it('损坏但请求自带 map 时仍然警告——用户需要知道项目配置没生效', () => {
    const { warning } = applyProjectDefaults(
      { ...BASE, mapPaths: ['custom.eclm'] },
      null,
      'invalid'
    )
    expect(warning).not.toBeNull()
  })
})
