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
            <n-button
              type="primary"
              block
              :disabled="descriptor.disabled"
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
import { computed } from 'vue'
import { NButton } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import type { EditorTab } from '../../stores/editor'
import { useMessage } from 'naive-ui'
import { useToolchainActions } from '../../composables/useToolchainActions'

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
  action: ((tab: EditorTab) => void | Promise<void>) | null
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
    suggestion: '反编译为 .dmsg 文本源码后修改;指令 ins_N 自动翻译为名字。',
    actionLabel: '反编译为 .dmsg',
    actionNote: '调用 thmsg 反编译,产物 .dmsg 会自动在编辑器打开。',
    disabled: false,
    action: async (tab) => { await runDecompileMsg(tab.path) }
  },
  std: {
    typeLabel: 'Touhou STD 二进制',
    description: 'STD 控制原作 3D 背景的摄像机与几何数据。',
    suggestion: '反编译为 .dstd 文本源码后修改;jmp 参数顺序自动适配生态约定。',
    actionLabel: '反编译为 .dstd',
    actionNote: '调用 thstd 反编译,产物 .dstd 会自动在编辑器打开。',
    disabled: false,
    action: async (tab) => { await runDecompileStd(tab.path) }
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

const descriptor = computed(() => {
  const ext = activeTab.value?.extension?.toLowerCase()
  return (ext ? TOOL_DESCRIPTORS[ext] : null) || TOOL_DESCRIPTORS.ecl
})

function handleAction() {
  const tab = activeTab.value
  if (!tab || !descriptor.value.action || descriptor.value.disabled) return
  descriptor.value.action(tab)
}

function handleAdvanced() {
  const tab = activeTab.value
  if (!tab || !descriptor.value.advancedAction || descriptor.value.disabled) return
  descriptor.value.advancedAction(tab)
}
</script>
