export const THECL_MODE_LABELS = {
  compile: '编译源文件',
  decompile: '反编译二进制',
  header: '生成头文件'
}

export const THECL_MODE_OPTIONS = [
  { label: THECL_MODE_LABELS.compile, value: 'compile' },
  { label: THECL_MODE_LABELS.decompile, value: 'decompile' },
  { label: THECL_MODE_LABELS.header, value: 'header' }
]

export const THECL_VERSION_OPTIONS = [
  '6', '7', '8', '9', '95', '10', '103', '11', '12',
  '125', '128', '13', '14', '143', '15', '16', '165',
  '17', '18', '185', '19', '20'
].map((value) => ({ label: value, value }))

export function createDefaultTheclPayload() {
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

export function createDefaultBuildPayload(tool = 'thecl') {
  if (tool === 'thecl') {
    return createDefaultTheclPayload()
  }

  return {
    tool,
    inputPath: ''
  }
}
