<template>
  <n-modal
    :show="show"
    preset="card"
    class="w-[720px] max-w-[92vw]"
    :mask-closable="false"
    @update:show="handleVisibleChange"
  >
    <template #header>
      <div class="flex items-center justify-between pr-2">
        <div>
          <div class="text-sm font-semibold text-white">{{ title }}</div>
          <div v-if="subtitle" class="text-xs text-gray-400 mt-1">{{ subtitle }}</div>
        </div>
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">
          {{ tool }}
        </div>
      </div>
    </template>

    <slot />

    <template #footer>
      <div class="flex items-center justify-between gap-3">
        <div class="text-xs text-gray-500">
          <slot name="footer-note">{{ footerNote }}</slot>
        </div>
        <div class="flex items-center gap-2">
          <n-button quaternary @click="$emit('cancel')">取消</n-button>
          <n-button
            type="primary"
            :loading="submitting"
            :disabled="confirmDisabled"
            @click="$emit('confirm')"
          >
            {{ confirmText }}
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup>
import { NButton, NModal } from 'naive-ui'

defineProps({
  show: { type: Boolean, required: true },
  title: { type: String, default: '构建配置' },
  subtitle: { type: String, default: '' },
  tool: { type: String, default: '' },
  footerNote: { type: String, default: '执行后会自动把结果写入“输出 / 问题”面板。' },
  confirmText: { type: String, default: '确认执行' },
  confirmDisabled: { type: Boolean, default: false },
  submitting: { type: Boolean, default: false }
})

const emit = defineEmits(['cancel', 'confirm'])

function handleVisibleChange(value) {
  if (!value) {
    emit('cancel')
  }
}
</script>
