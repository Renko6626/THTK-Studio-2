<template>
  <div class="h-full w-full bg-[#1e1e1e] text-gray-200 flex items-center justify-center px-8">
    <div class="w-full max-w-[760px] rounded-lg border border-white/8 bg-[#181818] shadow-[0_18px_60px_rgba(0,0,0,0.28)]">
      <div class="border-b border-white/8 px-6 py-5">
        <div class="text-[11px] uppercase tracking-[0.14em] text-gray-500">Binary Script</div>
        <div class="mt-2 text-2xl font-semibold text-white">{{ activeTab?.name || '二进制文件' }}</div>
        <div class="mt-2 text-sm text-gray-400">
          {{ descriptor.description }}
        </div>
      </div>

      <div v-if="!descriptor.disabled" class="px-6 pt-4">
        <div v-if="fileExtension(activeTab?.path || '') === 'dat'" class="text-xs text-gray-400">
          解包版本：由 thdat 自动识别
        </div>
        <GameVersionSummary v-else />
      </div>
      <div class="grid grid-cols-[1fr_240px] gap-6 px-6 py-6">
        <div class="space-y-4">
          <div class="rounded border border-white/8 bg-[#202020] p-4">
            <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">文件信息</div>
            <dl class="mt-3 space-y-2 text-sm">
              <div class="flex items-start justify-between gap-4">
                <dt class="text-gray-400">路径</dt>
                <dd class="text-gray-100 break-all text-right">{{ activeTab?.path }}</dd>
              </div>
              <div class="flex items-center justify-between gap-4">
                <dt class="text-gray-400">大小</dt>
                <dd class="text-gray-100">{{ formattedSize }}</dd>
              </div>
              <div class="flex items-center justify-between gap-4">
                <dt class="text-gray-400">类型</dt>
                <dd class="text-gray-100">{{ descriptor.typeLabel }}</dd>
              </div>
            </dl>
          </div>

          <div class="rounded border border-white/8 bg-[#202020] p-4">
            <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">建议操作</div>
            <div class="mt-3 text-sm text-gray-300 leading-6">
              {{ descriptor.suggestion }}
            </div>
          </div>
        </div>

        <div class="rounded border border-white/8 bg-[#151515] p-4 flex flex-col justify-between">
          <div>
            <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">操作</div>
            <div class="mt-3 text-sm text-gray-300 leading-6">
              {{ descriptor.actionNote }}
            </div>
          </div>

          <div class="mt-6 space-y-3">
            <div v-if="fileExtension(activeTab?.path || '') === 'msg'">
              <div class="text-gray-400 mb-2">脚本类型</div>
              <n-select v-model:value="selectedKind" :options="MSG_SCRIPT_KIND_OPTIONS" size="small" />
            </div>
            <div v-if="descriptor.needsEncoding">
              <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-2">
                游戏文本编码
              </div>
              <n-select
                v-model:value="selectedEncoding"
                :options="ENCODING_OPTIONS"
                size="small"
                placeholder="跟随项目设置"
              />
              <div class="mt-2 text-xs text-gray-500 leading-5">
                原作通常使用 Shift-JIS，汉化版可能使用 GBK。若解码报错或出现乱码，请按游戏所用编码重新解包。
              </div>
            </div>
            <n-button
              type="primary"
              block
              :disabled="descriptor.disabled || editorStore.compiling"
              :loading="editorStore.compiling"
              @click="handleAction"
            >
              {{ descriptor.actionLabel }}
            </n-button>
            <n-button
              v-if="descriptor.advancedAction"
              text
              size="small"
              class="text-blue-300 hover:text-blue-200"
              @click="handleAdvanced"
            >
              {{ descriptor.advancedLabel || '高级选项…' }}
            </n-button>
            <div class="text-xs text-gray-500">
              操作结果会出现在输出面板。
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import GameVersionSummary from '../Common/GameVersionSummary.vue'
import { fileExtension } from '../../utils/fileExtension'
import { computed, ref, watch } from 'vue'
import { NButton, NSelect } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import type { EditorTab } from '../../stores/editor'
import { useMessage } from 'naive-ui'
import { useToolchainActions } from '../../composables/useToolchainActions'

import { useBuildDialogStore } from '../../stores/buildDialog'
import { inferMsgScriptKind, MSG_SCRIPT_KIND_OPTIONS } from '../../services/toolchains/msgMetadata'
import type { MsgScriptKind } from '../../types'
const buildDialog = useBuildDialogStore()
const selectedKind = ref<MsgScriptKind>('stage')
const editorStore = useEditorStore()
const message = useMessage()
const {
  runDecompileEclQuick, runDecompileEclAdvanced,
  runDecompileMsg, runDecompileStd, runExtractDat,
} = useToolchainActions({ message })

const activeTab = computed(() => editorStore.activeTab)


const formattedSize = computed(() => {
  const size = Number(activeTab.value?.size ?? 0)
  if (!size) return '未知'
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(2)} MB`
})

interface ToolDescriptor {
  typeLabel: string
  description: string
  suggestion: string
  actionLabel: string
  actionNote: string
  /** true 表示该工具链尚未实现（anm），此时 action 为 null */
  disabled: boolean
  action: ((tab: EditorTab, encoding: string | null) => void | Promise<void>) | null
  /** 该类型的操作需要用户选游戏文本编码（目前只有 msg） */
  needsEncoding?: boolean
  advancedAction?: (tab: EditorTab) => void
  advancedLabel?: string
}

const TOOL_DESCRIPTORS: Record<string, ToolDescriptor> = {
  ecl: {
    typeLabel: 'Touhou ECL 二进制',
    description: 'ECL 控制原作敌机行为与弹幕逻辑。',
    suggestion: '反编译为 .decl 文本源码,在编辑器中修改后再编译回 .ecl。',
    actionLabel: '反编译为 .decl',
    actionNote: '使用项目默认 thecl 版本与 eclmap 快速反编译。如需选择版本/参数可点击下方"高级选项"。',
    disabled: false,
    action: async (tab) => { await runDecompileEclQuick(tab.path) },
    advancedAction: (tab) => { runDecompileEclAdvanced(tab.path) },
    advancedLabel: '高级选项…'
  },
  msg: {
    typeLabel: 'Touhou MSG 二进制',
    description: 'MSG 控制原作对话流程与立绘切换。',
    suggestion: '反编译为 .msgspec 文本源码后修改;使用 truth 原生脚本和指令 map。',
    actionLabel: '反编译为 .msgspec',
    actionNote: '调用 trumsg 反编译,产物 .msgspec 会自动在编辑器打开。',
    disabled: false,
    // 游戏文本编码猜不出来：Shift-JIS 解任何字节都不报错，解错了只是满屏乱码
    needsEncoding: true,
    action: async (tab, encoding) => { await runDecompileMsg(tab.path, encoding, selectedKind.value) },
    advancedAction: tab => buildDialog.openDialog({ tool: 'trumsg', mode: 'decompile', inputPath: tab.path, scriptKind: selectedKind.value, encoding: selectedEncoding.value || '' })
  },
  std: {
    typeLabel: 'Touhou STD 二进制',
    description: 'STD 控制原作 3D 背景的摄像机与几何数据。',
    suggestion: '反编译为 .stdspec 文本源码后修改;支持符号标签和结构化控制流。',
    actionLabel: '反编译为 .stdspec',
    actionNote: '调用 trustd 反编译,产物 .stdspec 会自动在编辑器打开。',
    disabled: false,
    action: async (tab) => { await runDecompileStd(tab.path) },
    advancedAction: tab => buildDialog.openDialog({ tool: 'trustd', mode: 'decompile', inputPath: tab.path })
  },
  dat: {
    typeLabel: 'Touhou DAT 容器',
    description: 'DAT 是原作的资源容器,内含数百个 ECL/MSG/STD/ANM 等资源。',
    suggestion: '解包到一个目录,再用对应工具编辑各文件;打包用菜单"打包目录为 .dat"。',
    actionLabel: '解包到目录…',
    actionNote: '弹出目录选择对话框,默认目标是与 .dat 同名的兄弟目录。',
    disabled: false,
    action: async (tab) => { await runExtractDat(tab.path) }
  },
  anm: {
    typeLabel: 'Touhou ANM 二进制',
    description: 'ANM 包含精灵动画与图像资源。',
    suggestion: 'thanm 工具链支持尚未实现,可暂用 thtk 命令行解包。',
    actionLabel: '尚未支持',
    actionNote: 'ANM 文本编辑层与精灵预览将在后续版本中加入。',
    disabled: true,
    action: null
  }
}

/**
 * 本次操作用的游戏文本编码。null = 跟随项目配置。
 *
 * 之所以要能单次覆盖：一次往返里两个方向常常不同——按 shift-jis 解开原版日文，
 * 翻译成中文后按 gbk 打包。项目配置只能记住一个默认值。
 */
const selectedEncoding = ref<string | null>(null)
watch(() => activeTab.value?.path, path => { selectedKind.value = inferMsgScriptKind(path); selectedEncoding.value = null }, { immediate: true })

const ENCODING_OPTIONS = [
  { label: '跟随项目设置', value: '' },
  { label: 'Shift-JIS（原作）', value: 'shift-jis' },
  { label: 'GBK（汉化版，需游戏侧适配）', value: 'gbk' },
  { label: 'UTF-8（原版游戏读不了）', value: 'utf-8' }
]

const descriptor = computed(() => {
  const ext = fileExtension(activeTab.value?.path || '')
  return TOOL_DESCRIPTORS[ext] || { typeLabel: '未识别的文件类型', description: '当前文件不属于二进制脚本。', suggestion: '请重新打开该文件。', actionLabel: '没有可用操作', actionNote: '', disabled: true, action: null }
})

function handleAction() {
  const tab = activeTab.value
  if (!tab || !descriptor.value.action || descriptor.value.disabled) return
  descriptor.value.action(tab, descriptor.value.needsEncoding ? selectedEncoding.value : null)
}

function handleAdvanced() {
  const tab = activeTab.value
  if (!tab || !descriptor.value.advancedAction || descriptor.value.disabled) return
  descriptor.value.advancedAction(tab)
}
</script>
