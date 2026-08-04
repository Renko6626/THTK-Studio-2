<template>
  <div class="space-y-4">
    <div class="grid grid-cols-[1.15fr_0.85fr] gap-4">
      <div class="rounded border border-white/8 bg-[#1b1b1b] p-3">
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">目标文件</div>
        <div class="text-sm text-gray-100 break-all">{{ model.inputPath || '未选择文件' }}</div>
        <div class="text-xs text-gray-500 mt-2">
          输出路径留空时，将按模式自动推导默认产物文件名。
        </div>
      </div>

      <div class="rounded border border-white/8 bg-[#1b1b1b] p-3">
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">任务摘要</div>
        <div class="space-y-2 text-sm">
          <div class="flex items-center justify-between">
            <span class="text-gray-400">工具</span>
            <span class="text-gray-100">thmsg</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-400">模式</span>
            <span class="text-gray-100">{{ MSG_MODE_LABELS[model.mode] }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-400">编码</span>
            <span class="text-gray-100">{{ encodingSummary }}</span>
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

    <div v-if="isDecompileMode" class="build-option">
      <n-checkbox
        :checked="model.withComments"
        @update:checked="updateField('withComments', $event)"
      >
        在每行末尾追加指令说明
      </n-checkbox>
      <div class="text-xs text-gray-500 mt-1">
        便于阅读，但打包时会被忽略——注释不会写进 .msg。
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NCheckbox, NForm, NFormItem, NInput, NSelect } from 'naive-ui'
import {
  MSG_ENCODING_OPTIONS,
  MSG_MODE_LABELS,
  MSG_MODE_OPTIONS
} from '../../../services/toolchains/msgMetadata'
import type { MsgBuildPayload } from '../../../types'

const props = defineProps<{
  model: MsgBuildPayload
}>()

const emit = defineEmits<{
  'update:model': [value: MsgBuildPayload]
}>()

const isDecompileMode = computed(() => props.model.mode === 'decompile')

const encodingSummary = computed(
  () =>
    MSG_ENCODING_OPTIONS.find((option) => option.value === props.model.encoding)?.label ||
    '跟随项目设置'
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
.build-option {
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  padding: 0.75rem;
  background: #1b1b1b;
}
</style>
