import { beforeEach, describe, expect, it, vi } from 'vitest'

const setZoom = vi.fn()
vi.mock('@tauri-apps/api/webview', () => ({
  getCurrentWebview: () => ({ setZoom })
}))

import {
  applyZoom,
  clampLevel,
  formatScale,
  levelToScale,
  loadZoomLevel,
  MAX_LEVEL,
  MIN_LEVEL,
  saveZoomLevel
} from '../../src/services/workbench/zoom'

describe('缩放刻度', () => {
  it('0 级是 100%', () => {
    expect(levelToScale(0)).toBe(1)
    expect(formatScale(0)).toBe('100%')
  })

  /** 等比而非线性：低倍率步进小、高倍率步进大，同 VS Code 的 1.2 公比 */
  it('每级 1.2 倍，等比递进', () => {
    expect(levelToScale(1)).toBeCloseTo(1.2)
    expect(levelToScale(2)).toBeCloseTo(1.44)
    expect(levelToScale(-1)).toBeCloseTo(1 / 1.2)
    // 等比的意义：相邻两级的比值恒定
    expect(levelToScale(3) / levelToScale(2)).toBeCloseTo(levelToScale(1) / levelToScale(0))
  })

  it('百分比取整显示', () => {
    expect(formatScale(1)).toBe('120%')
    expect(formatScale(-1)).toBe('83%')
  })
})

describe('clampLevel', () => {
  it('限制在上下限内', () => {
    expect(clampLevel(MAX_LEVEL + 10)).toBe(MAX_LEVEL)
    expect(clampLevel(MIN_LEVEL - 10)).toBe(MIN_LEVEL)
  })

  it('取整', () => {
    expect(clampLevel(1.4)).toBe(1)
    expect(clampLevel(1.6)).toBe(2)
  })

  /** 坏值一律归 0——绝不能让 NaN 把界面缩到看不见 */
  it('非有限值归 0', () => {
    expect(clampLevel(NaN)).toBe(0)
    expect(clampLevel(Infinity)).toBe(0)
    expect(clampLevel(-Infinity)).toBe(0)
  })
})

describe('持久化', () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  it('存取往返', () => {
    saveZoomLevel(3)
    expect(loadZoomLevel()).toBe(3)
  })

  it('没存过时是 0', () => {
    expect(loadZoomLevel()).toBe(0)
  })

  /**
   * localStorage 的内容不可信：可能是旧版本写的，也可能被手改过。
   * 解析不出来就当 0，绝不让一个坏值把界面缩到看不见。
   */
  it('坏值归 0 而不是原样返回', () => {
    for (const bad of ['abc', '', 'null', '{}']) {
      window.localStorage.setItem('thtk-studio:workbench-zoom', bad)
      expect(loadZoomLevel()).toBe(0)
    }
  })

  it('超出范围的存量值被钳到界内', () => {
    window.localStorage.setItem('thtk-studio:workbench-zoom', '999')
    expect(loadZoomLevel()).toBe(MAX_LEVEL)
  })
})

describe('applyZoom', () => {
  beforeEach(() => {
    setZoom.mockReset()
  })

  it('把等级换算成倍率传给 webview', async () => {
    await applyZoom(1)
    expect(setZoom).toHaveBeenCalledWith(expect.closeTo(1.2, 5))
  })

  it('传入前先钳位', async () => {
    await applyZoom(999)
    expect(setZoom).toHaveBeenCalledWith(expect.closeTo(levelToScale(MAX_LEVEL), 5))
  })

  /** 无 Tauri 环境或平台不支持时静默降级——不能让缩放失败拖垮其他功能 */
  it('setZoom 抛错时不向上抛', async () => {
    setZoom.mockRejectedValueOnce(new Error('unsupported'))
    await expect(applyZoom(1)).resolves.toBeUndefined()
  })
})
