<template>
  <n-form label-placement="top" :model="model">
    <n-form-item label="目标文件"><div class="min-w-0 w-full truncate select-text" :title="model.inputPath">{{ model.inputPath }}</div></n-form-item>
    <n-form-item label="操作"><n-select :value="model.mode" :options="modes" @update:value="update('mode', $event)" /></n-form-item>
    <n-form-item label="输出路径"><n-input :value="model.outputPath" placeholder="留空按输入文件自动推导" @update:value="update('outputPath', $event)" /></n-form-item>
    <div class="text-xs text-gray-400">反编译生成 .stdspec，编译生成 .std。游戏版本和 map 使用项目设置。</div>
  </n-form>
</template>
<script setup lang="ts">
import { NForm, NFormItem, NInput, NSelect } from 'naive-ui'
import type { StdBuildPayload } from '../../../types'
const props = defineProps<{ model: StdBuildPayload }>()
const emit = defineEmits<{ 'update:model': [value: StdBuildPayload] }>()
const modes = [{ label: '编译为 .std', value: 'compile' }, { label: '反编译为 .stdspec', value: 'decompile' }]
function update<K extends keyof StdBuildPayload>(key: K, value: StdBuildPayload[K]) { emit('update:model', { ...props.model, [key]: value }) }
</script>
