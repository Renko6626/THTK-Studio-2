import type { MsgBuildPayload } from '../../types'
import type { SelectOption } from 'naive-ui'

export const MSG_MODE_LABELS: Record<MsgBuildPayload['mode'], string> = {
  decompile: '解包 (.msg → .dmsg)',
  compile: '打包 (.dmsg → .msg)'
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
export const MSG_ENCODING_OPTIONS: SelectOption[] = [
  { label: '跟随项目设置', value: '' },
  { label: 'Shift-JIS —— 原作', value: 'shift-jis' },
  { label: 'GBK —— 汉化版（需游戏侧适配）', value: 'gbk' },
  { label: 'UTF-8 —— 原版游戏读不了', value: 'utf-8' }
]

export function createDefaultMsgPayload(): MsgBuildPayload {
  return {
    tool: 'thmsg',
    mode: 'compile',
    inputPath: '',
    // 空字符串表示"自动推导"，请求构建器再把它转成 null
    outputPath: '',
    encoding: '',
    withComments: true
  }
}

export function inferMsgSuccessMessage(mode: MsgBuildPayload['mode']): string {
  return mode === 'compile' ? '打包完成' : '解包完成'
}
