import TheclBuildForm from '../../components/Dialogs/forms/TheclBuildForm.vue'
import { createTheclRequest, executeThecl, publishTheclResult } from './thecl'
import {
  createDefaultTheclPayload,
  THECL_MODE_LABELS,
  THECL_MODE_OPTIONS,
  THECL_VERSION_OPTIONS
} from './theclMetadata'

function inferTheclSuccessMessage(mode) {
  if (mode === 'compile') return '编译完成'
  if (mode === 'decompile') return '反编译完成'
  if (mode === 'header') return '头文件已生成'
  return '执行完成'
}

export const TOOLCHAIN_REGISTRY = {
  thecl: {
    id: 'thecl',
    label: 'Enemy Script Compiler',
    exeName: 'thecl.exe',
    supportsBuildDialog: true,
    buildDialogTitle: '构建配置',
    buildDialogSubtitle: '为当前脚本选择模式、版本和 thecl 选项',
    defaultPayload: createDefaultTheclPayload,
    buildFormComponent: TheclBuildForm,
    versionOptions: THECL_VERSION_OPTIONS,
    modeLabels: THECL_MODE_LABELS,
    modeOptions: THECL_MODE_OPTIONS,
    createRequest(payload) {
      return createTheclRequest({
        mode: payload.mode,
        version: payload.version,
        inputPath: payload.inputPath,
        outputPath: payload.outputPath || null,
        mapPaths: payload.mapPaths || [],
        useShiftJis: payload.useShiftJis,
        rawDump: payload.rawDump,
        simpleCreation: payload.simpleCreation,
        showOffsets: payload.showOffsets
      })
    },
    async execute(context, request, payload) {
      return context.runTheclRequest(request, {
        requireSave: payload.mode !== 'decompile',
        openOutput: payload.mode !== 'compile',
        successMessage: inferTheclSuccessMessage(payload.mode)
      })
    },
    executeDirect: executeThecl,
    publishResult: publishTheclResult
  },
  thmsg: {
    id: 'thmsg',
    label: 'Message Script Tool',
    exeName: 'thmsg.exe',
    supportsBuildDialog: false,
    defaultPayload: () => ({ tool: 'thmsg', inputPath: '' })
  },
  thanm: {
    id: 'thanm',
    label: 'Animation Tool',
    exeName: 'thanm.exe',
    supportsBuildDialog: false,
    defaultPayload: () => ({ tool: 'thanm', inputPath: '' })
  },
  thstd: {
    id: 'thstd',
    label: 'Stage Data Tool',
    exeName: 'thstd.exe',
    supportsBuildDialog: false,
    defaultPayload: () => ({ tool: 'thstd', inputPath: '' })
  },
  thdat: {
    id: 'thdat',
    label: 'Archive Tool',
    exeName: 'thdat.exe',
    supportsBuildDialog: false,
    defaultPayload: () => ({ tool: 'thdat', inputPath: '' })
  }
}

export function getToolchainDescriptor(tool) {
  return TOOLCHAIN_REGISTRY[tool] || null
}

export function getRegisteredToolchains() {
  return Object.values(TOOLCHAIN_REGISTRY)
}

export function createDefaultBuildPayload(tool = 'thecl') {
  const descriptor = getToolchainDescriptor(tool)
  return descriptor?.defaultPayload?.() || { tool, inputPath: '' }
}
