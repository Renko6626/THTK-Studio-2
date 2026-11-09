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
          <div class="text-xs text-gray-400 mt-1">配置 thtk、随包 truth 的覆盖路径与默认游戏版本</div>
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

        <n-form-item label="truth 目录（留空使用随包版本）">
          <n-input v-model:value="form.truth_dir" placeholder="可选：覆盖 trumsg / trustd 所在目录" />
        </n-form-item>

        <n-form-item label="默认游戏版本">
          <n-select
            v-model:value="form.default_game_version"
            :options="versionOptions"
            filterable
          />
        </n-form-item>
      </n-form>

      <div class="rounded border border-white/8 bg-[#151515]">
        <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
          <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">依赖</div>
          <div class="text-[11px] text-gray-600">truth 随应用提供，thtk 使用手动路径</div>
        </div>

        <div v-if="dependencyLoading" class="px-4 py-4 text-sm text-gray-400">
          正在读取依赖状态…
        </div>
        <div v-else-if="dependencyError" class="px-4 py-4 text-sm text-[#f48771] break-all">
          读取依赖状态失败：{{ dependencyError }}
        </div>
        <div v-else-if="dependencyStatus" class="divide-y divide-white/6">
          <div class="grid grid-cols-[110px_1fr] gap-x-4 gap-y-3 px-4 py-4 text-sm">
            <div class="text-gray-400">thtk</div>
            <div class="space-y-1">
              <div class="flex items-center gap-2">
                <span :class="thtkStateClass">
                  {{ thtkStateLabel }}
                </span>
                <span class="text-xs text-gray-600">{{ dependencyStatus.thtk.license }}</span>
              </div>
              <div class="break-all text-gray-300">
                {{ dependencyStatus.thtk.resolvedPath || '手动路径为空' }}
              </div>
              <div class="text-xs text-gray-500">{{ dependencyStatus.thtk.message }}</div>
            </div>

            <div class="text-gray-400">ECL map</div>
            <div class="space-y-1">
              <div class="flex items-center gap-2">
                <span :class="dependencyStatus.maps.source === 'bundled' ? 'text-[#89d185]' : 'text-[#f48771]'">
                  {{ dependencyStatus.maps.source === 'bundled' ? '内置基线已就绪' : '内置基线未释放' }}
                </span>
                <span class="text-xs text-gray-600">
                  {{ dependencyStatus.maps.license }} · {{ dependencyStatus.maps.sourceCommit.slice(0, 7) }}
                </span>
              </div>
              <div class="break-all text-xs text-gray-500">{{ dependencyStatus.maps.bundledDir }}</div>
              <div class="text-xs text-gray-400">
                覆盖：
                <span class="text-gray-200">{{ formatVersions(dependencyStatus.maps.coveredVersions) || '无' }}</span>
              </div>
              <div class="text-xs text-gray-400">
                缺失：
                <span class="text-[#f48771]">{{ formatVersions(dependencyStatus.maps.missingVersions) || '无' }}</span>
                <span v-if="dependencyStatus.maps.missingVersions.length" class="text-gray-500">
                  （请在项目设置的 mapPaths 里手动提供）
                </span>
              </div>
            </div>
          </div>

          <div v-if="dependencyStatus.warnings.length" class="space-y-1 px-4 py-3 text-xs text-[#e2c08d]">
            <div v-for="warning in dependencyStatus.warnings" :key="warning">- {{ warning }}</div>
          </div>
        </div>
      </div>

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

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
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
import { getDependencyStatus, getSettings, getToolchainStatuses, saveSettings } from '../../api'
import type { DependencyStatus, ToolchainStatus, UserSettingsPayload } from '../../types'
import type { ToolchainDescriptor } from '../../services/toolchains/registry'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { getRegisteredToolchains } from '../../services/toolchains/registry'
import { useGameVersionsStore } from '../../stores/gameVersions'

const toolchainSettingsStore = useToolchainSettingsStore()
const message = useMessage()
const registeredToolchains = getRegisteredToolchains()

const form = reactive<UserSettingsPayload>({
  thtk_dir: '',
  truth_dir: '',
  thecl_path: '',
  eclmap_path: '',
  tool_overrides: {},
  default_game_version: '20',
  theme: 'dark'
})

const toolStatuses = reactive<Record<string, ToolchainStatus | null>>({})
const checking = ref(false)
const saving = ref(false)
const gameVersionsStore = useGameVersionsStore()
const versionOptions = computed(() => gameVersionsStore.allOptions)

// 依赖卡片只读：数据全部来自后端 get_dependency_status，表单不回写这些字段。
const dependencyStatus = ref<DependencyStatus | null>(null)
const dependencyError = ref('')
const dependencyLoading = ref(false)

const thtkStateLabel = computed(() => {
  const thtk = dependencyStatus.value?.thtk
  if (!thtk) return ''
  // available 只表示解析出的 thecl.exe 存在，版本探测仍由工具链总览负责。
  if (thtk.available) return '已找到 thecl.exe'
  return thtk.configured ? '已配置但未找到 thecl.exe' : '未配置'
})

const thtkStateClass = computed(() =>
  dependencyStatus.value?.thtk.available ? 'text-[#89d185]' : 'text-[#f48771]'
)

watch(
  () => toolchainSettingsStore.visible,
  async (visible) => {
    if (!visible) return
    await loadSettings()
    await Promise.all([refreshStatuses(), loadDependencies()])
  }
)

async function loadSettings() {
  const settings = await getSettings()
  form.thtk_dir = settings?.thtk_dir || ''
  form.truth_dir = settings?.truth_dir || ''
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

async function loadDependencies() {
  dependencyLoading.value = true
  dependencyError.value = ''
  try {
    dependencyStatus.value = await getDependencyStatus()
  } catch (error) {
    dependencyStatus.value = null
    dependencyError.value = String(error)
  } finally {
    dependencyLoading.value = false
  }
}

function formatVersions(versions: number[]) {
  return versions.map(version => `th${version}`).join(' / ')
}

function updateToolOverride(toolId: string, value: string) {
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

async function pickToolExe(tool: ToolchainDescriptor) {
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
    const normalizedOverrides: Record<string, string> = Object.fromEntries(
      Object.entries(form.tool_overrides)
        .map(([key, value]) => [key, String(value || '').trim()])
        .filter(([, value]) => Boolean(value))
    )

    await saveSettings({
      thtk_dir: form.thtk_dir.trim(),
      truth_dir: form.truth_dir.trim(),
      thecl_path: normalizedOverrides.thecl || '',
      eclmap_path: form.eclmap_path.trim(),
      tool_overrides: normalizedOverrides,
      default_game_version: form.default_game_version,
      theme: form.theme
    })
    window.dispatchEvent(new CustomEvent('thtk:toolchain-settings-saved'))
    await Promise.all([refreshStatuses(), loadDependencies()])
    message.success('工具链设置已保存')
    closeDialog()
  } catch (error) {
    message.error(`保存失败: ${error}`)
  } finally {
    saving.value = false
  }
}

function statusBadgeClass(available: boolean | undefined) {
  return available
    ? 'text-[#89d185]'
    : 'text-[#f48771]'
}

function closeDialog() {
  toolchainSettingsStore.close()
}

function handleVisibleChange(value: boolean) {
  if (!value) {
    closeDialog()
  }
}
</script>
