import type { TheclBuildPayload, TheclMode } from '../../types'

export const THECL_MODE_LABELS: Record<TheclMode, string> = {
  compile: '编译源文件',
  decompile: '反编译二进制',
  header: '生成头文件'
}

/** 与 naive-ui n-select 的 options 兼容 */
export interface SelectOption {
  label: string
  value: string
  [key: string]: unknown
}

export const THECL_MODE_OPTIONS: SelectOption[] = [
  { label: THECL_MODE_LABELS.compile, value: 'compile' },
  { label: THECL_MODE_LABELS.decompile, value: 'decompile' },
  { label: THECL_MODE_LABELS.header, value: 'header' }
]

export const THECL_VERSION_OPTIONS: SelectOption[] = [
  '6', '7', '8', '9', '95', '10', '103', '11', '12',
  '125', '128', '13', '14', '143', '15', '16', '165',
  '17', '18', '185', '19', '20'
].map((value) => ({ label: value, value }))

/**
 * 构建对话框的默认表单值。
 * 注意 outputPath 用空字符串表示"自动推导"，请求构建器再把它转成 null。
 */
export function createDefaultTheclPayload(): TheclBuildPayload {
  return {
    tool: 'thecl',
    mode: 'compile',
    inputPath: '',
    version: '20',
    outputPath: '',
    mapPaths: [],
    useShiftJis: true,
    rawDump: false,
    simpleCreation: false,
    showOffsets: false
  }
}
