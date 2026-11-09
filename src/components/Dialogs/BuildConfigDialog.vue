<template>
  <BuildDialogShell
    :show="buildDialogStore.visible"
    :title="dialogTitle"
    :subtitle="dialogSubtitle"
    :tool="dialogToolLabel"
    :confirm-disabled="!formModel.inputPath || (formModel.tool === 'thecl' && !formModel.version.trim())"
    :submitting="editorStore.compiling"
    @cancel="closeDialog"
    @confirm="handleSubmit"
  >
    <GameVersionSummary class="mb-4" :version="formModel.tool === 'thecl' ? formModel.version : undefined" />
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
import GameVersionSummary from '../Common/GameVersionSummary.vue'
import { getSettings } from '../../api'
import BuildDialogShell from './BuildDialogShell.vue'
import { useEditorStore } from '../../stores/editor'
import { useProjectStore } from '../../stores/project'
import { useBuildDialogStore } from '../../stores/buildDialog'
import { useTheclActions } from '../../composables/useTheclActions'
import { useToolchainActions } from '../../composables/useToolchainActions'
import type { BuildDialogPayload, TheclBuildPayload } from '../../types'
import {
  createDefaultBuildPayload,
  getToolchainDescriptor
} from '../../services/toolchains/registry'
import { inferMsgScriptKind } from '../../services/toolchains/msgMetadata'

const buildDialogStore = useBuildDialogStore()
const editorStore = useEditorStore()
const projectStore = useProjectStore()
const message = useMessage()
const { runTheclRequest } = useTheclActions()
const { runDecompileMsg, runCompileMsg, runDecompileStd, runCompileStd } = useToolchainActions({ message })

const formModel = reactive(createDefaultBuildPayload())

const activeToolchain = computed(() => getToolchainDescriptor(formModel.tool))
const activeBuildFormComponent = computed(() => activeToolchain.value?.buildFormComponent || null)
const dialogToolLabel = computed(() => formModel.tool || 'toolchain')
const dialogTitle = computed(() => activeToolchain.value?.buildDialogTitle || '工具链配置')
const dialogSubtitle = computed(() => (
  activeToolchain.value?.buildDialogSubtitle || '为当前工具链确认执行参数'
))

watch(
  () => [buildDialogStore.visible, buildDialogStore.payload] as const,
  async ([visible], _, onCleanup) => {
    if (!visible) return

    let current = true
    onCleanup(() => { current = false })
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
          if (!current) return
          if (!formModel.version) {
            formModel.version = settings?.default_game_version || ''
          }
          const configuredMapPath = String(settings?.eclmap_path || '').trim()
          if (!formModel.mapPaths?.length && configuredMapPath) {
            formModel.mapPaths = [configuredMapPath]
          }
        } catch {
          if (!current) return
          // 保留空值，由表单提示选择版本，避免悄悄使用 TH20。
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

  // 菜单打开对话框时只带 inputPath；结局/staff 的类型按文件名补上。
  // 用户在表单里手动改过（payload 显式带 scriptKind）则不覆盖。
  if (formModel.tool === 'trumsg' && !(payload && 'scriptKind' in payload)) {
    const msgModel = formModel as unknown as { scriptKind: string }
    msgModel.scriptKind = inferMsgScriptKind(formModel.inputPath)
  }
}

function updateFormModel(nextModel: BuildDialogPayload) {
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
  if (formModel.tool === 'thecl' && !formModel.version.trim()) {
    message.warning('请先选择目标游戏版本')
    return
  }

  const submittedPayload = buildDialogStore.payload
  const descriptor = activeToolchain.value
  const context = { runTheclRequest, runDecompileMsg, runCompileMsg, runDecompileStd, runCompileStd }

  // 自带执行入口的工具（thmsg）走这条：它的请求形状与 thecl 完全不同，
  // 强行套 TheclRequest 会让两边都别扭。
  if (descriptor?.runFromPayload) {
    const result = await descriptor.runFromPayload(
      context,
      formModel as unknown as BuildDialogPayload
    )
    if (result?.success && buildDialogStore.payload === submittedPayload) closeDialog()
    return
  }

  if (!descriptor?.createRequest || !descriptor?.execute) {
    message.warning('当前工具链尚未接入执行器')
    return
  }

  // 到这一步 descriptor.createRequest 存在就说明它是 thecl 的载荷。
  const payload = formModel as TheclBuildPayload
  const request = descriptor.createRequest(payload)
  const result = await descriptor.execute(context, request, payload)

  if (result?.success && buildDialogStore.payload === submittedPayload) {
    closeDialog()
  }
}
</script>
