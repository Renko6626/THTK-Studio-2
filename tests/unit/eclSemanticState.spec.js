import { beforeEach, describe, expect, it } from 'vitest'
import {
  clearScopedEclSemanticData,
  getActiveEclSemanticData,
  getEclSemanticDataForModel,
  setActiveEclSemanticScope,
  updateScopedEclSemanticData
} from '../../src/services/languages/ecl/semantic-state'

const dataFor = (version) => ({
  version,
  sourcePath: `/maps/th${version}.eclm`,
  instructions: [{ opcode: 1, name: `ins_${version}` }],
  builtins: []
})

/** 伪造 Monaco model：作用域匹配只看 uri.fsPath / uri.path */
const modelAt = (fsPath) => ({ uri: { fsPath } })

describe('ECL 词表作用域', () => {
  beforeEach(() => {
    // 模块级 Map 跨用例存活，逐个清掉避免互相影响
    for (const key of ['/proj/a', '/proj/b', '/proj/a/nested', '__global__']) {
      clearScopedEclSemanticData(key)
    }
    setActiveEclSemanticScope('__global__')
  })

  it('按作用域隔离，切换活动作用域拿到各自的词表', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    updateScopedEclSemanticData('/proj/b', dataFor('17'))

    setActiveEclSemanticScope('/proj/a')
    expect(getActiveEclSemanticData().version).toBe('18')

    setActiveEclSemanticScope('/proj/b')
    expect(getActiveEclSemanticData().version).toBe('17')
  })

  it('清掉一个作用域后回落到空词表，不影响另一个', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    updateScopedEclSemanticData('/proj/b', dataFor('17'))

    clearScopedEclSemanticData('/proj/a')

    setActiveEclSemanticScope('/proj/a')
    // 取不到时返回空词表而非 null，迁移后要保持这个契约
    expect(getActiveEclSemanticData().instructions).toEqual([])

    setActiveEclSemanticScope('/proj/b')
    expect(getActiveEclSemanticData().version).toBe('17')
  })

  it('按 model 路径匹配作用域，优先取最长前缀', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    updateScopedEclSemanticData('/proj/a/nested', dataFor('17'))

    expect(getEclSemanticDataForModel(modelAt('/proj/a/st01.decl')).version).toBe('18')
    // 嵌套作用域更长，应该赢
    expect(getEclSemanticDataForModel(modelAt('/proj/a/nested/st02.decl')).version).toBe('17')
  })

  it('model 不在任何作用域下时回落到活动作用域', () => {
    updateScopedEclSemanticData('/proj/a', dataFor('18'))
    setActiveEclSemanticScope('/proj/a')

    expect(getEclSemanticDataForModel(modelAt('/elsewhere/x.decl')).version).toBe('18')
  })

  it('归一化：builtins 并入全部指令名并去重', () => {
    updateScopedEclSemanticData('/proj/a', {
      version: '18',
      sourcePath: '',
      instructions: [{ opcode: 1, name: 'ins_a' }, { opcode: 2, name: 'ins_b' }],
      builtins: ['ins_a', 'helper']
    })
    setActiveEclSemanticScope('/proj/a')

    const builtins = getActiveEclSemanticData().builtins
    expect([...builtins].sort()).toEqual(['helper', 'ins_a', 'ins_b'])
  })
})
