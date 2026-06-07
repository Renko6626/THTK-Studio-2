<template>
  <n-modal
    :show="toolchainSettingsStore.visible"
    preset="card"
    class="w-[860px] max-w-[94vw]"
    :mask-closable="false"
    @update:show="handleVisibleChange"
  >
    <template #header>
      <div class="flex items-center justify-between pr-2">
        <div>
          <div class="text-sm font-semibold text-white">工具链设置</div>
          <div class="text-xs text-gray-400 mt-1">统一管理 THTK 根目录、各工具覆盖路径、默认 ECL map 和版本检测</div>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <n-form label-placement="top" :model="form">
        <n-form-item label="THTK 根目录">
          <div class="flex items-center gap-2 w-full">
            <n-input v-model:value="form.thtk_dir" placeholder="例如：D:\\tools\\thtk" />
            <n-button @click="pickThtkDir">浏览</n-button>
          </div>
        </n-form-item>

        <n-form-item label="默认游戏版本">
          <n-select
            v-model:value="form.default_game_version"
            :options="versionOptions"
            filterable
            tag
          />
        </n-form-item>
      </n-form>

      <div class="rounded border border-white/8 bg-[#151515]">
        <div class="border-b border-white/8 px-4 py-3 text-[11px] uppercase tracking-[0.12em] text-gray-500">
          工具链总览
        </div>

        <div class="divide-y divide-white/6">
          <div v-for="tool in registeredToolchains" :key="tool.id" class="px-4 py-4">
            <div class="flex items-start justify-between gap-4">
              <div>
                <div class="text-sm font-semibold text-white">{{ tool.id }}</div>
                <div class="text-xs text-gray-500 mt-1">{{ tool.label }} · {{ tool.exeName }}</div>
              </div>
              <div class="text-xs" :class="statusBadgeClass(toolStatuses[tool.id]?.available)">
                {{ toolStatuses[tool.id]?.available ? 'Available' : 'Unavailable' }}
              </div>
            </div>

            <div class="mt-4 grid grid-cols-[1fr_280px] gap-4">
              <div class="space-y-3">
                <div>
                  <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-2">覆盖路径</div>
                  <div class="flex items-center gap-2">
                    <n-input
                      :value="form.tool_overrides[tool.id] || ''"
                      :placeholder="`留空则默认使用 ${tool.exeName}`"
                      @update:value="updateToolOverride(tool.id, $event)"
                    />
                    <n-button @click="pickToolExe(tool)">浏览</n-button>
                  </div>
                </div>

                <div v-if="tool.id === 'thecl'">
                  <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-2">默认 ECL Map</div>
                  <div class="flex items-center gap-2">
                    <n-input
                      v-model:value="form.eclmap_path"
                      placeholder="留空则按默认版本自动推导 thXX.eclm"
                    />
                    <n-button @click="pickEclMapFile">浏览</n-button>
                  </div>
                </div>

                <div class="grid grid-cols-[100px_1fr] gap-x-3 gap-y-2 text-sm">
                  <div class="text-gray-400">解析路径</div>
                  <div class="break-all text-gray-100">{{ toolStatuses[tool.id]?.resolvedPath || '未配置' }}</div>
                  <div class="text-gray-400">版本</div>
                  <div class="text-gray-100">{{ toolStatuses[tool.id]?.version || '未检测到' }}</div>
                  <div class="text-gray-400">说明</div>
                  <div class="text-gray-300">{{ toolStatuses[tool.id]?.message || '尚未检测' }}</div>
                </div>
              </div>

              <div class="rounded border border-white/8 bg-[#1b1b1b] p-3">
                <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 mb-3">工具状态</div>
                <div class="space-y-2 text-sm">
                  <div class="flex items-center justify-between">
                    <span class="text-gray-400">可构建表单</span>
                    <span class="text-gray-100">{{ tool.supportsBuildDialog ? 'Yes' : 'No' }}</span>
                  </div>
                  <div class="flex items-center justify-between">
                    <span class="text-gray-400">版本检测</span>
                    <span class="text-gray-100">Yes</span>
                  </div>
                  <div v-if="tool.id === 'thecl'" class="flex items-start justify-between gap-4">
                    <span class="text-gray-400">默认 ECL Map</span>
                    <span class="text-right break-all text-gray-100">{{ form.eclmap_path || '自动推导' }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between gap-3">
        <n-button quaternary :loading="checking" @click="refreshStatuses">
          刷新全部工具链状态
        </n-button>
        <div class="flex items-center gap-2">
          <n-button quaternary @click="closeDialog">取消</n-button>
          <n-button type="primary" :loading="saving" @click="save">
            保存设置
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup>
import { reactive, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NSelect,
  useMessage
} from 'naive-ui'
import { getSettings, getToolchainStatuses, saveSettings } from '../../api'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { getRegisteredToolchains } from '../../services/toolchains/registry'
import { THECL_VERSION_OPTIONS } from '../../services/toolchains/theclMetadata'

const toolchainSettingsStore = useToolchainSettingsStore()
const message = useMessage()
const registeredToolchains = getRegisteredToolchains()

const form = reactive({
  thtk_dir: '',
  thecl_path: '',
  eclmap_path: '',
  tool_overrides: {},
  default_game_version: '20',
  theme: 'dark'
})

const toolStatuses = reactive({})
const checking = ref(false)
const saving = ref(false)
const versionOptions = THECL_VERSION_OPTIONS

watch(
  () => toolchainSettingsStore.visible,
  async (visible) => {
    if (!visible) return
    await loadSettings()
    await refreshStatuses()
  }
)

async function loadSettings() {
  const settings = await getSettings()
  form.thtk_dir = settings?.thtk_dir || ''
  form.thecl_path = settings?.thecl_path || ''
  form.eclmap_path = settings?.eclmap_path || ''
  form.tool_overrides = { ...(settings?.tool_overrides || {}) }
  if (!form.tool_overrides.thecl && form.thecl_path) {
    form.tool_overrides.thecl = form.thecl_path
  }
  form.default_game_version = settings?.default_game_version || '20'
  form.theme = settings?.theme || 'dark'
}

async function refreshStatuses() {
  checking.value = true
  try {
    const statuses = await getToolchainStatuses()
    registeredToolchains.forEach((tool) => {
      toolStatuses[tool.id] = statuses.find(item => item.tool === tool.id) || null
    })
  } catch (error) {
    message.error(`刷新工具链状态失败: ${error}`)
  } finally {
    checking.value = false
  }
}

function updateToolOverride(toolId, value) {
  form.tool_overrides = {
    ...form.tool_overrides,
    [toolId]: value
  }
}

async function pickThtkDir() {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    form.thtk_dir = String(selected)
  }
}

async function pickToolExe(tool) {
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'Executable', extensions: ['exe'] }]
  })
  if (selected) {
    updateToolOverride(tool.id, String(selected))
  }
}

async function pickEclMapFile() {
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'ECL Map', extensions: ['eclm', 'eclmap'] }]
  })
  if (selected) {
    form.eclmap_path = String(selected)
  }
}

async function save() {
  saving.value = true
  try {
    const normalizedOverrides = Object.fromEntries(
      Object.entries(form.tool_overrides)
        .map(([key, value]) => [key, String(value || '').trim()])
        .filter(([, value]) => Boolean(value))
    )

    await saveSettings({
      thtk_dir: form.thtk_dir.trim(),
      thecl_path: normalizedOverrides.thecl || '',
      eclmap_path: form.eclmap_path.trim(),
      tool_overrides: normalizedOverrides,
      default_game_version: form.default_game_version,
      theme: form.theme
    })
    window.dispatchEvent(new CustomEvent('thtk:toolchain-settings-saved'))
    await refreshStatuses()
    message.success('工具链设置已保存')
    closeDialog()
  } catch (error) {
    message.error(`保存失败: ${error}`)
  } finally {
    saving.value = false
  }
}

function statusBadgeClass(available) {
  return available
    ? 'text-[#89d185]'
    : 'text-[#f48771]'
}

function closeDialog() {
  toolchainSettingsStore.close()
}

function handleVisibleChange(value) {
  if (!value) {
    closeDialog()
  }
}
</script>
