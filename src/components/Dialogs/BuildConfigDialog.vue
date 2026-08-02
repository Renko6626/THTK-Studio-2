<template>
  <BuildDialogShell
    :show="buildDialogStore.visible"
    :title="dialogTitle"
    :subtitle="dialogSubtitle"
    :tool="dialogToolLabel"
    :confirm-disabled="!formModel.inputPath"
    :submitting="editorStore.compiling"
    @cancel="closeDialog"
    @confirm="handleSubmit"
  >
    <component
      :is="activeBuildFormComponent"
      v-if="activeBuildFormComponent"
      :model="formModel"
      @update:model="updateFormModel"
    />

    <div
      v-else
      class="rounded border border-white/8 bg-[#181818] p-4 text-sm text-gray-400"
    >
      当前工具链暂未提供图形化配置表单。
    </div>
  </BuildDialogShell>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { getSettings } from '../../api'
import BuildDialogShell from './BuildDialogShell.vue'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { useBuildDialogStore } from '../../stores/buildDialog'
import { useTheclActions } from '../../composables/useTheclActions'
import type { BuildDialogPayload, TheclBuildPayload } from '../../types'
import {
  createDefaultBuildPayload,
  getToolchainDescriptor
} from '../../services/toolchains/registry'

const buildDialogStore = useBuildDialogStore()
const editorStore = useEditorStore()
const projectStore = useProjectStore()
const { runTheclRequest } = useTheclActions()
const message = useMessage()

const formModel = reactive(createDefaultBuildPayload())

const activeToolchain = computed(() => getToolchainDescriptor(formModel.tool))
const activeBuildFormComponent = computed(() => activeToolchain.value?.buildFormComponent || null)
const dialogToolLabel = computed(() => formModel.tool || 'toolchain')
const dialogTitle = computed(() => activeToolchain.value?.buildDialogTitle || '工具链配置')
const dialogSubtitle = computed(() => (
  activeToolchain.value?.buildDialogSubtitle || '为当前工具链确认执行参数'
))

watch(
  () => buildDialogStore.visible,
  async (visible) => {
    if (!visible) return

    syncForm(buildDialogStore.payload)

    if (formModel.tool === 'thecl') {
      // 优先从项目配置读取，回退到全局设置
      const pc = projectStore.projectConfig
      if (pc) {
        if (!formModel.version && pc.gameVersion) {
          formModel.version = pc.gameVersion
        }
        if (!formModel.mapPaths?.length && pc.mapPaths?.length) {
          formModel.mapPaths = [...pc.mapPaths]
        }
        if (pc.encoding) {
          formModel.useShiftJis = pc.encoding === 'shift-jis'
        }
      }

      // 仍缺少的字段从全局设置补齐
      if (!formModel.version || !formModel.mapPaths?.length) {
        try {
          const settings = await getSettings()
          if (!formModel.version) {
            formModel.version = settings?.default_game_version || '20'
          }
          const configuredMapPath = String(settings?.eclmap_path || '').trim()
          if (!formModel.mapPaths?.length && configuredMapPath) {
            formModel.mapPaths = [configuredMapPath]
          }
        } catch {
          if (!formModel.version) {
            formModel.version = '20'
          }
        }
      }
    }
  },
  { immediate: true }
)

function syncForm(payload: Partial<BuildDialogPayload> | null | undefined) {
  const base = createDefaultBuildPayload(payload?.tool)

  Object.assign(formModel, base, payload, {
    inputPath: payload?.inputPath || editorStore.activeTab?.path || '',
    // 只有 thecl 的载荷有 mapPaths；其余工具没有构建对话框
    mapPaths: [...((payload as Partial<TheclBuildPayload>)?.mapPaths || [])]
  })
}

function updateFormModel(nextModel: TheclBuildPayload) {
  Object.assign(formModel, nextModel)
}

function closeDialog() {
  buildDialogStore.close()
}

async function handleSubmit() {
  if (!formModel.inputPath) {
    message.warning('当前没有可执行的输入文件')
    return
  }

  const descriptor = activeToolchain.value
  if (!descriptor?.createRequest || !descriptor?.execute) {
    message.warning('当前工具链尚未接入执行器')
    return
  }

  // formModel 是 reactive 包装的联合类型；只有 thecl 支持构建对话框，
  // 到这一步 descriptor.createRequest 存在就说明它是 thecl 的载荷。
  const payload = formModel as TheclBuildPayload
  const request = descriptor.createRequest(payload)
  const result = await descriptor.execute({ runTheclRequest }, request, payload)

  if (result?.success) {
    closeDialog()
  }
}
</script>
