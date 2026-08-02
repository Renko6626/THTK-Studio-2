import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProjectStore } from '../../src/stores/project'
import { getDirChildren as rawGetDirChildren, getFileTree as rawGetFileTree } from '../../src/api'

const getDirChildren = vi.mocked(rawGetDirChildren)
const getFileTree = vi.mocked(rawGetFileTree)

import { makeFileNode } from '../helpers/fixtures'

vi.mock('../../src/api', () => ({
  openProject: vi.fn(),
  getFileTree: vi.fn(),
  getDirChildren: vi.fn(),
  loadProjectConfig: vi.fn(),
  saveProjectConfig: vi.fn()
}))

/** 后端浅层扫描对**所有**目录都发 children: null（Rust 的 Option<Vec> 是 None） */
function unexpandedDir(path: string) {
  return makeFileNode({ path, is_dir: true, isLeaf: false })
}

function file(path: string) {
  return makeFileNode({ path })
}

describe('projectStore.refresh', () => {
  let store: ReturnType<typeof useProjectStore>

  beforeEach(() => {
    // 模块 mock 的调用记录不会自动跨用例清空
    vi.clearAllMocks()
    setActivePinia(createPinia())
    store = useProjectStore()
    store.rootPath = '/proj'
  })

  it('文件树里有未展开目录时也能正常刷新', async () => {
    // 回归防线：曾经 _collectLoadedDirs 只排除 undefined 不排除 null，
    // 于是 null.length 抛 TypeError —— 而后端给每个未展开目录发的都是 null，
    // 等于任何带子目录的项目刷新都是坏的。
    store.files = [unexpandedDir('/proj/stage'), file('/proj/a.decl')]
    getFileTree.mockResolvedValue([file('/proj/b.decl')])

    await store.refresh()

    expect(store.files).toHaveLength(1)
    expect(store.files[0].path).toBe('/proj/b.decl')
  })

  it('已展开的目录在刷新后被重新加载', async () => {
    const expanded = { ...unexpandedDir('/proj/stage'), children: [file('/proj/stage/x.decl')] }
    store.files = [expanded]
    getFileTree.mockResolvedValue([unexpandedDir('/proj/stage')])
    getDirChildren.mockResolvedValue([file('/proj/stage/y.decl')])

    await store.refresh()

    expect(getDirChildren).toHaveBeenCalledWith('/proj/stage')
    expect(store.files[0].children?.[0].path).toBe('/proj/stage/y.decl')
  })

  it('没有项目根时直接返回，不去调后端', async () => {
    store.rootPath = null

    await store.refresh()

    expect(getFileTree).not.toHaveBeenCalled()
  })
})
