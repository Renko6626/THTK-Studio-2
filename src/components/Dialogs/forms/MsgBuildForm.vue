<template>
  <div class="space-y-4">
    <div class="grid grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)] gap-4">
      <div class="min-w-0 rounded border border-white/8 bg-[#1b1b1b] p-3">
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">目标文件</div>
        <div class="truncate text-sm text-gray-100 select-text" :title="model.inputPath">{{ model.inputPath || '未选择文件' }}</div>
        <div class="text-xs text-gray-500 mt-2">
          输出路径留空时，将按模式自动推导默认产物文件名。
        </div>
      </div>

      <div class="rounded border border-white/8 bg-[#1b1b1b] p-3">
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">任务摘要</div>
        <div class="task-summary space-y-2 text-sm">
          <div class="flex items-center justify-between gap-3">
            <span class="text-gray-400">工具</span>
            <span class="text-gray-100">trumsg</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <span class="text-gray-400">模式</span>
            <span class="text-gray-100">{{ MSG_MODE_LABELS[model.mode] }}</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <span class="text-gray-400">脚本类型</span>
            <span class="text-gray-100" :title="String(scriptKindSummary)">{{ scriptKindSummary }}</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <span class="text-gray-400">编码</span>
            <span class="min-w-0 truncate text-gray-100" :title="String(encodingSummary)">{{ encodingSummary }}</span>
          </div>
        </div>
      </div>
    </div>

    <n-form label-placement="top" :model="model" class="grid grid-cols-2 gap-x-4 gap-y-2">
      <n-form-item label="操作模式">
        <n-select
          :value="model.mode"
          :options="MSG_MODE_OPTIONS"
          @update:value="updateField('mode', $event)"
        />
      </n-form-item>

      <n-form-item label="游戏文本编码">
        <n-select
          :value="model.encoding"
          :options="MSG_ENCODING_OPTIONS"
          @update:value="updateField('encoding', $event)"
        />
      </n-form-item>

      <n-form-item label="脚本类型">
        <n-select
          :value="model.scriptKind"
          :disabled="model.mode === 'compile' && /\.(msgspec|endspec)$/i.test(model.inputPath)"
          :options="MSG_SCRIPT_KIND_OPTIONS"
          @update:value="updateField('scriptKind', $event)"
        />
      </n-form-item>

      <n-form-item label="输出路径" class="col-span-2">
        <n-input
          :value="model.outputPath"
          placeholder="留空则自动推导"
          @update:value="updateField('outputPath', $event)"
        />
      </n-form-item>
    </n-form>

    <div
      v-if="encodingHint"
      class="rounded border px-3 py-2 text-xs leading-5"
      :class="model.encoding === 'utf-8'
        ? 'border-[#f48771]/40 bg-[#f48771]/8 text-[#f48771]'
        : 'border-[#cca700]/40 bg-[#cca700]/8 text-[#cca700]'"
    >
      {{ encodingHint }}
    </div>

    <div
      v-if="model.scriptKind === 'ending'"
      class="rounded border border-[#cca700]/40 bg-[#cca700]/8 px-3 py-2 text-xs leading-5 text-[#cca700]"
    >
      结局 / staff 会使用 trumsg 的 <code>--ending</code>，并使用另一套 opcode 表。文件名是
      <code>e01.msg</code> 或 <code>staff1.msg</code> 时会自动识别。
    </div>


  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NForm, NFormItem, NInput, NSelect } from 'naive-ui'
import {
  MSG_ENCODING_OPTIONS,
  MSG_MODE_LABELS,
  MSG_MODE_OPTIONS,
  MSG_SCRIPT_KIND_OPTIONS
} from '../../../services/toolchains/msgMetadata'
import type { MsgBuildPayload } from '../../../types'

const props = defineProps<{
  model: MsgBuildPayload
}>()

const emit = defineEmits<{
  'update:model': [value: MsgBuildPayload]
}>()


const encodingSummary = computed(
  () =>
    MSG_ENCODING_OPTIONS.find((option) => option.value === props.model.encoding)?.label ||
    '跟随项目设置'
)

const scriptKindSummary = computed(
  () =>
    MSG_SCRIPT_KIND_OPTIONS.find((option) => option.value === props.model.scriptKind)?.label ||
    '关卡对话'
)

/**
 * 编码的前提说明。放在表单里而不是 tooltip 里：选错的后果不是报错，
 * 而是产出一个游戏读不了的文件，这种代价必须在点下按钮之前就看得见。
 */
const encodingHint = computed(() => {
  switch (props.model.encoding) {
    case 'gbk':
      return 'GBK 同时装得下简体汉字与日文，适合简日混排的汉化版。前提是游戏侧已做适配（字体 charset 补丁、字节边界判断、转区），未打补丁的原版读 GBK 会乱码。'
    case 'utf-8':
      return '⚠ 原版东方游戏不支持任何形式的 Unicode，UTF-8 的 .msg 在原版里读不出来。仅在你确知目标引擎能读 UTF-8 时选它。'
    case 'shift-jis':
      return '原作唯一的原生编码。它装不下简体汉字——写中文请改用 GBK，否则打包会失败并指出是哪几个字。'
    default:
      return ''
  }
})

function updateField<K extends keyof MsgBuildPayload>(field: K, value: MsgBuildPayload[K]) {
  emit('update:model', { ...props.model, [field]: value })
}
</script>

<style scoped>
.task-summary > div > span:first-child { flex-shrink: 0; white-space: nowrap; }
.task-summary > div > span:last-child { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
