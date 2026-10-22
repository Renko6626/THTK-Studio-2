import { describe, expect, it } from 'vitest'
import {
  createDefaultMsgPayload,
  inferMsgScriptKind,
  MSG_SCRIPT_KIND_OPTIONS
} from '../../src/services/toolchains/msgMetadata'

describe('MSG 脚本类型识别', () => {
  it('把 e01–e12 与 staff1–4 识别为结局/staff', () => {
    for (const path of ['/p/e01.msg', '/p/e12.dmsg', '/p/staff1.msg', '/p/staff4.dmsg']) {
      expect(inferMsgScriptKind(path)).toBe('ending')
    }
  })

  it('其它 .msg / .dmsg 默认是关卡对话', () => {
    for (const path of ['/p/st01a.msg', '/p/st07d.dmsg', '/p/ending.msg', '/p/e1.msg']) {
      expect(inferMsgScriptKind(path)).toBe('stage')
    }
  })

  it('Windows 路径也能识别', () => {
    expect(inferMsgScriptKind('C:\\proj\\e03.dmsg')).toBe('ending')
    expect(inferMsgScriptKind('C:\\proj\\st03a.dmsg')).toBe('stage')
  })

  it('默认载荷包含显式的脚本类型', () => {
    expect(createDefaultMsgPayload().scriptKind).toBe('stage')
    expect(MSG_SCRIPT_KIND_OPTIONS.map((option) => option.value)).toEqual(['stage', 'ending'])
  })
})
