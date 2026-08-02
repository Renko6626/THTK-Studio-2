import { vi } from 'vitest'
import type { DialogApi, DialogOptions, MessageApi } from 'naive-ui'

/**
 * 测试里用的 naive-ui API 替身。
 *
 * 只实现被测代码真正调到的方法。断言成完整接口是刻意的：把 `MessageApi` 的
 * 全部 7 个方法都 mock 出来对测试意图毫无帮助，只会让每次 naive-ui 升级
 * 都要改一遍测试。
 */
export function createMessageStub() {
  const stub = {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn()
  }
  return { stub, api: stub as unknown as MessageApi }
}

/**
 * dialog 替身，同时记录每次调用的 options，便于测试取出按钮回调。
 * `warning` 是唯一被用到的入口。
 */
export function createDialogStub() {
  const calls: DialogOptions[] = []
  const warning = vi.fn((options: DialogOptions) => {
    calls.push(options)
    return { destroy: vi.fn() }
  })
  return { calls, warning, api: { warning } as unknown as DialogApi }
}
