import { describe, expect, it } from 'vitest'
import { resolveTerminalShortcut } from '../../src/composables/useTerminalActionBridge'

function key(value: string, options: KeyboardEventInit = {}) {
  return new KeyboardEvent('keydown', { key: value, ...options })
}

describe('终端快捷键分工', () => {
  it('Ctrl+C 只在有选区时复制，否则交给 shell', () => {
    const event = key('c', { ctrlKey: true })
    expect(resolveTerminalShortcut(event, 'windows', true)).toBe('copy')
    expect(resolveTerminalShortcut(event, 'windows', false)).toBeNull()
  })

  it('显式复制粘贴、查找及面板切换交给工作台', () => {
    expect(resolveTerminalShortcut(key('C', { ctrlKey: true, shiftKey: true }), 'linux', false)).toBe('copy')
    expect(resolveTerminalShortcut(key('V', { ctrlKey: true, shiftKey: true }), 'windows', false)).toBe('paste')
    expect(resolveTerminalShortcut(key('f', { ctrlKey: true }), 'linux', false)).toBe('find')
    expect(resolveTerminalShortcut(key('`', { ctrlKey: true }), 'windows', false)).toBe('togglePanel')
    expect(resolveTerminalShortcut(key('c', { metaKey: true }), 'mac', true)).toBe('copy')
    expect(resolveTerminalShortcut(key('f', { metaKey: true }), 'mac', false)).toBe('find')
  })

  it('系统粘贴和常用 shell 控制键保持原生处理', () => {
    for (const value of ['r', 'w', 'o', 'k', 'v']) {
      expect(resolveTerminalShortcut(key(value, { ctrlKey: true }), 'windows', false)).toBeNull()
    }
    expect(resolveTerminalShortcut(key('v', { metaKey: true }), 'mac', false)).toBeNull()
    expect(resolveTerminalShortcut(key('c', { ctrlKey: true, altKey: true }), 'windows', true)).toBeNull()
    expect(resolveTerminalShortcut(key('c'), 'windows', true)).toBeNull()
    expect(resolveTerminalShortcut(new KeyboardEvent('keyup', { key: 'c', ctrlKey: true }), 'windows', true)).toBeNull()
  })
})
