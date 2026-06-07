<template>
  <div class="h-full w-full bg-[#1e1e1e] text-gray-200 flex items-center justify-center px-8">
    <div class="w-full max-w-[760px] rounded-lg border border-white/8 bg-[#181818] shadow-[0_18px_60px_rgba(0,0,0,0.28)]">
      <div class="border-b border-white/8 px-6 py-5">
        <div class="text-[11px] uppercase tracking-[0.14em] text-gray-500">Binary Script</div>
        <div class="mt-2 text-2xl font-semibold text-white">{{ activeTab?.name || 'ECL 文件' }}</div>
        <div class="mt-2 text-sm text-gray-400">
          当前打开的是二进制 `.ecl` 文件，不能直接用文本编辑器可靠修改。
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
                <dd class="text-gray-100">Touhou ECL Binary</dd>
              </div>
            </dl>
          </div>

          <div class="rounded border border-white/8 bg-[#202020] p-4">
            <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">建议操作</div>
            <div class="mt-3 text-sm text-gray-300 leading-6">
              先将该文件反编译为 `.decl` 源文件，再进入文本编辑器修改。这样更符合当前 IDE 的脚本工作流，也能把结果接入输出和问题面板。
            </div>
          </div>
        </div>

        <div class="rounded border border-white/8 bg-[#151515] p-4 flex flex-col justify-between">
          <div>
            <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">操作</div>
            <div class="mt-3 text-sm text-gray-300 leading-6">
              通过反编译按钮打开构建配置弹窗，确认版本和参数后生成可编辑的 `.decl` 文件。
            </div>
          </div>

          <div class="mt-6 space-y-3">
            <n-button type="primary" block @click="openDecompileDialog">
              解包 / 反编译为 .decl
            </n-button>
            <div class="text-xs text-gray-500">
              生成的文件会自动出现在资源管理器里，并可直接打开。
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { NButton } from 'naive-ui'
import { useEditorStore } from '../../stores/editor'
import { useBuildDialogStore } from '../../stores/buildDialog'

const editorStore = useEditorStore()
const buildDialogStore = useBuildDialogStore()

const activeTab = computed(() => editorStore.activeTab)

const formattedSize = computed(() => {
  const size = Number(activeTab.value?.size ?? 0)
  if (!size) return '未知'
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(2)} MB`
})

function openDecompileDialog() {
  if (!activeTab.value?.path) return
  buildDialogStore.openDialog({
    tool: 'thecl',
    mode: 'decompile',
    inputPath: activeTab.value.path,
    version: '',
    useShiftJis: true
  })
}
</script>
