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
            <span class="text-gray-100">thecl</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-400">模式</span>
            <span class="text-gray-100">{{ THECL_MODE_LABELS[model.mode] }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-gray-400">版本</span>
            <span class="text-gray-100">{{ model.version || '使用默认值' }}</span>
          </div>
        </div>
      </div>
    </div>

    <n-form
      label-placement="top"
      :model="model"
      class="grid grid-cols-2 gap-x-4 gap-y-2"
    >
      <n-form-item label="操作模式">
        <n-select
          :value="model.mode"
          :options="THECL_MODE_OPTIONS"
          @update:value="updateField('mode', $event)"
        />
      </n-form-item>

      <n-form-item label="目标游戏版本">
        <n-select
          :value="model.version"
          :options="THECL_VERSION_OPTIONS"
          filterable
          tag
          placeholder="选择或输入版本"
          @update:value="updateField('version', $event)"
        />
      </n-form-item>

      <n-form-item label="输出路径" class="col-span-2">
        <n-input
          :value="model.outputPath"
          placeholder="留空则自动推导输出路径"
          @update:value="updateField('outputPath', $event)"
        />
      </n-form-item>

      <n-form-item label="Map 路径" class="col-span-2">
        <n-dynamic-tags
          :value="model.mapPaths"
          :render-tag="renderTag"
          @update:value="updateField('mapPaths', $event)"
        />
      </n-form-item>
    </n-form>

    <div class="rounded border border-white/8 bg-[#171717] p-3">
      <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">可选参数</div>
      <div class="grid grid-cols-2 gap-3">
        <label class="build-option">
          <n-checkbox
            :checked="model.useShiftJis"
            @update:checked="updateField('useShiftJis', $event)"
          />
          <div>
            <div class="text-sm text-gray-100">Shift-JIS / UTF-8 转换</div>
            <div class="text-xs text-gray-500">对应 `-j`，一般建议保持启用。</div>
          </div>
        </label>

        <label class="build-option" :class="{ 'opacity-50': !isCompileMode }">
          <n-checkbox
            :checked="model.simpleCreation"
            :disabled="!isCompileMode"
            @update:checked="updateField('simpleCreation', $event)"
          />
          <div>
            <div class="text-sm text-gray-100">Simple Creation</div>
            <div class="text-xs text-gray-500">对应 `-s`，只在编译模式下有效。</div>
          </div>
        </label>

        <label class="build-option" :class="{ 'opacity-50': !isDecompileMode }">
          <n-checkbox
            :checked="model.rawDump"
            :disabled="!isDecompileMode"
            @update:checked="updateField('rawDump', $event)"
          />
          <div>
            <div class="text-sm text-gray-100">Raw Dump</div>
            <div class="text-xs text-gray-500">对应 `-r`，保留更原始的反编译结果。</div>
          </div>
        </label>

        <label class="build-option" :class="{ 'opacity-50': !isDecompileMode }">
          <n-checkbox
            :checked="model.showOffsets"
            :disabled="!isDecompileMode"
            @update:checked="updateField('showOffsets', $event)"
          />
          <div>
            <div class="text-sm text-gray-100">显示地址偏移</div>
            <div class="text-xs text-gray-500">对应 `-x`，仅在 dump / 反编译时有效。</div>
          </div>
        </label>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, h } from 'vue'
import {
  NCheckbox,
  NDynamicTags,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NTag
} from 'naive-ui'
import {
  THECL_MODE_LABELS,
  THECL_MODE_OPTIONS,
  THECL_VERSION_OPTIONS
} from '../../../services/toolchains/theclMetadata'

const props = defineProps({
  model: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['update:model'])

const isCompileMode = computed(() => props.model.mode === 'compile')
const isDecompileMode = computed(() => props.model.mode === 'decompile')

function updateField(field, value) {
  const nextModel = {
    ...props.model,
    [field]: value
  }

  if (field === 'mode' && value !== 'compile') {
    nextModel.simpleCreation = false
  }

  if (field === 'mode' && value !== 'decompile') {
    nextModel.rawDump = false
    nextModel.showOffsets = false
  }

  emit('update:model', nextModel)
}

function renderTag(tag, index) {
  return h(
    NTag,
    {
      key: `${tag}-${index}`,
      closable: true,
      onClose: () => {
        updateField(
          'mapPaths',
          props.model.mapPaths.filter((_, currentIndex) => currentIndex !== index)
        )
      }
    },
    { default: () => tag }
  )
}
</script>

<style scoped>
.build-option {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px 12px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.02);
}
</style>
