import type { MsgBuildPayload, MsgScriptKind } from '../../types'
import type { SelectOption } from 'naive-ui'

export const MSG_MODE_LABELS: Record<MsgBuildPayload['mode'], string> = {
  decompile: '反编译为 truth 源码',
  compile: '编译为 .msg'
}

export const MSG_MODE_OPTIONS: SelectOption[] = (
  Object.keys(MSG_MODE_LABELS) as MsgBuildPayload['mode'][]
).map((value) => ({ label: MSG_MODE_LABELS[value], value }))

/**
 * 游戏文本编码选项。
 *
 * 每一项都写明前提——选错的后果不是报错而是产出一个游戏读不了的文件，
 * 只给编码名等于让用户去猜。空串表示跟随项目配置。
 */
/**
 * 两套 opcode 的切换点。结局/staff 不是 stage 的一个选项，而是 trumsg 的 `--ending`
 * 模式；opcode 3 在两边含义不同，选错会得到一张看似合法但完全错误的表。
 */
export const MSG_SCRIPT_KIND_OPTIONS: SelectOption[] = [
  { label: '关卡对话（st01a.msg 等）', value: 'stage' },
  { label: '结局 / staff（e01.msg、staff1.msg）', value: 'ending' }
]

export function inferMsgScriptKind(path: string | null | undefined): MsgScriptKind {
  if (path?.toLowerCase().endsWith('.endspec')) return 'ending'
  if (path?.toLowerCase().endsWith('.msgspec')) return 'stage'
  const stem = String(path || '')
    .split(/[\\/]/)
    .pop()
    ?.replace(/\.[^.]+$/, '')
    .toLowerCase() || ''

  const looksLikeEnding = stem.startsWith('staff')
    || (stem.length === 3 && stem.startsWith('e') && /^\d{2}$/.test(stem.slice(1)))

  return looksLikeEnding ? 'ending' : 'stage'
}

export const MSG_ENCODING_OPTIONS: SelectOption[] = [
  { label: '跟随项目设置', value: '' },
  { label: 'Shift-JIS —— 原作', value: 'shift-jis' },
  { label: 'GBK —— 汉化版（需游戏侧适配）', value: 'gbk' },
  { label: 'UTF-8 —— 原版游戏读不了', value: 'utf-8' }
]

export function createDefaultMsgPayload(): MsgBuildPayload {
  return {
    tool: 'trumsg',
    mode: 'compile',
    inputPath: '',
    // 空字符串表示"自动推导"，请求构建器再把它转成 null
    outputPath: '',
    encoding: '',
    scriptKind: 'stage'
  }
}

export function inferMsgSuccessMessage(mode: MsgBuildPayload['mode']): string {
  return mode === 'compile' ? '打包完成' : '解包完成'
}
