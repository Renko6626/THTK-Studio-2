<template>
  <n-modal
    :show="projectSettingsStore.visible"
    preset="card"
    class="w-[720px] max-w-[94vw]"
    :mask-closable="false"
    @update:show="handleVisibleChange"
  >
    <template #header>
      <div class="pr-2">
        <div class="text-sm font-semibold text-white">项目设置</div>
        <div class="text-xs text-gray-400 mt-1 break-all">
          {{ projectStore.rootPath || '未打开工作区' }}
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- 配置文件损坏：如实展示是哪个文件、坏在哪，覆盖前要二次确认 -->
      <div
        v-if="projectStore.hasInvalidProjectConfig"
        class="rounded border border-[#f48771]/40 bg-[#f48771]/8 px-4 py-3 space-y-1"
      >
        <div class="text-sm font-semibold text-[#f48771]">配置文件无法读取</div>
        <div class="text-xs text-gray-300 break-all">{{ projectStore.projectConfigPath }}</div>
        <div class="text-xs text-gray-400 break-all">{{ projectStore.projectConfigError }}</div>
        <div class="text-xs text-gray-400">
          下面的表单是默认值，不是该文件的内容。保存会整体替换它，届时会再确认一次。
        </div>
      </div>

      <div
        v-else-if="projectStore.projectConfigStatus === 'absent'"
        class="rounded border border-white/8 bg-[#1b1b1b] px-4 py-3 text-xs text-gray-400"
      >
        该项目还没有 <span class="text-gray-200">.thtk-project.json</span>，保存后会创建。
        留空的字段表示沿用全局设置。
      </div>

      <n-form label-placement="top" :model="form" :disabled="loading">
        <div class="grid grid-cols-2 gap-x-4">
          <n-form-item label="目标游戏版本">
            <n-select
              v-model:value="form.gameVersion"
              :options="versionOptions"
              filterable
              tag
              clearable
              placeholder="留空则使用全局默认版本"
            />
          </n-form-item>

          <n-form-item label="文本编码">
            <n-radio-group v-model:value="form.encoding" class="w-full">
              <n-radio-button value="shift-jis" class="w-1/2 text-center">Shift-JIS</n-radio-button>
              <n-radio-button value="utf-8" class="w-1/2 text-center">UTF-8</n-radio-button>
            </n-radio-group>
          </n-form-item>
        </div>

        <n-form-item label="项目级 THTK 目录">
          <div class="flex items-center gap-2 w-full">
            <n-input v-model:value="form.thtkDir" placeholder="留空则沿用全局工具链设置" />
            <n-button :disabled="loading" @click="pickThtkDir">浏览</n-button>
          </div>
        </n-form-item>
      </n-form>

      <div class="rounded border border-white/8 bg-[#151515]">
        <div class="flex items-center justify-between border-b border-white/8 px-4 py-3">
          <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500">ECL Map 路径</div>
          <n-button size="tiny" :disabled="loading" @click="addMapPaths">添加…</n-button>
        </div>

        <div v-if="!form.mapPaths.length" class="px-4 py-4 text-xs text-gray-500">
          未配置。thecl 将回退到全局设置里的默认 eclmap。
        </div>

        <div v-else class="divide-y divide-white/6">
          <div
            v-for="(path, index) in form.mapPaths"
            :key="index"
            class="flex items-center gap-2 px-4 py-2"
          >
            <n-input
              :value="path"
              size="small"
              placeholder="绝对路径，或相对于项目根的路径"
              @update:value="updateMapPath(index, $event)"
            />
            <n-button
              size="tiny"
              quaternary
              :disabled="index === 0"
              title="上移"
              @click="moveMapPath(index, -1)"
            >
              ↑
            </n-button>
            <n-button
              size="tiny"
              quaternary
              :disabled="index === form.mapPaths.length - 1"
              title="下移"
              @click="moveMapPath(index, 1)"
            >
              ↓
            </n-button>
            <n-button size="tiny" quaternary title="移除" @click="removeMapPath(index)">
              移除
            </n-button>
          </div>
        </div>

        <div class="border-t border-white/8 px-4 py-2 text-[11px] text-gray-500">
          第一条同时用作编辑器的 ECL 词表来源，保证补全与编译使用同一份 eclmap。
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <n-button quaternary @click="close">取消</n-button>
        <n-button type="primary" :loading="saving" :disabled="loading" @click="save">
          保存
        </n-button>
      </div>
    </template>
  </n-modal>
</template>

<script setup>
import { h, reactive, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NRadioButton,
  NRadioGroup,
  NSelect,
  useDialog,
  useMessage
} from 'naive-ui'
import { useProjectSettingsStore } from '../../stores/projectSettings'
import { useProjectStore } from '../../stores/project'
import { THECL_VERSION_OPTIONS } from '../../services/toolchains/theclMetadata'

const projectSettingsStore = useProjectSettingsStore()
const projectStore = useProjectStore()
const message = useMessage()
const dialog = useDialog()

const versionOptions = THECL_VERSION_OPTIONS
const loading = ref(false)
const saving = ref(false)

const form = reactive({
  gameVersion: '',
  encoding: 'shift-jis',
  mapPaths: [],
  thtkDir: ''
})

// 表单开始编辑时锁定的项目根。保存时一并发给后端做比对。
const editingRoot = ref('')

watch(
  () => projectSettingsStore.visible,
  async (visible) => {
    if (!visible) return
    editingRoot.value = projectStore.rootPath || ''
    loading.value = true
    try {
      // 每次打开都重读磁盘：文件可能被外部工具或用户手动改过
      await projectStore.reloadProjectConfig()
    } finally {
      syncForm()
      loading.value = false
    }
  }
)

// 对话框拦不住全局快捷键：开着设置框按 Ctrl+O 就会切走项目，而表单里还是上一个
// 项目的值。此时继续保存会写错文件，所以直接关掉并说明原因。
watch(
  () => projectStore.rootPath,
  (nextRoot) => {
    if (!projectSettingsStore.visible) return
    if (nextRoot && editingRoot.value && nextRoot === editingRoot.value) return
    message.warning('项目已切换，已关闭项目设置以免写错文件。')
    close()
  }
)

/** 从 store 填充表单。配置不可用时用默认值，不去猜损坏文件里的内容。 */
function syncForm() {
  const config = projectStore.projectConfig
  // null 而非 ''：n-select 对空字符串会套用 fallbackOption 当成"已选中"，
  // placeholder 就再也不显示了
  form.gameVersion = config?.gameVersion || null
  form.encoding = config?.encoding || 'shift-jis'
  form.mapPaths = [...(config?.mapPaths || [])]
  form.thtkDir = config?.toolchain?.thtkDir || ''
}

/** 表单 → 后端结构。字段名对齐 serde 的 camelCase。 */
function buildConfig() {
  return {
    gameVersion: form.gameVersion?.trim() || '',
    encoding: form.encoding,
    // 空条目会被后端校验拒绝，这里先清掉，免得用户因为一个空行存不了
    mapPaths: form.mapPaths.map(item => String(item).trim()).filter(Boolean),
    toolchain: {
      thtkDir: form.thtkDir?.trim() || ''
    }
  }
}

function updateMapPath(index, value) {
  form.mapPaths = form.mapPaths.map((item, current) => (current === index ? value : item))
}

function moveMapPath(index, delta) {
  const target = index + delta
  if (target < 0 || target >= form.mapPaths.length) return
  const next = [...form.mapPaths]
  ;[next[index], next[target]] = [next[target], next[index]]
  form.mapPaths = next
}

function removeMapPath(index) {
  form.mapPaths = form.mapPaths.filter((_, current) => current !== index)
}

async function addMapPaths() {
  const selected = await open({
    directory: false,
    multiple: true,
    defaultPath: projectStore.rootPath || undefined,
    filters: [{ name: 'ECL Map', extensions: ['eclm', 'eclmap'] }]
  })
  if (!selected) return

  const picked = (Array.isArray(selected) ? selected : [selected]).map(String)
  const added = picked.filter(path => !form.mapPaths.includes(path))
  form.mapPaths = [...form.mapPaths, ...added]
}

async function pickThtkDir() {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: projectStore.rootPath || undefined
  })
  if (selected) {
    form.thtkDir = String(selected)
  }
}

function save() {
  // 损坏的文件不经确认不得覆盖——必须先告诉用户将被替换的是哪个文件
  if (projectStore.hasInvalidProjectConfig) {
    const path = projectStore.projectConfigPath
    const error = projectStore.projectConfigError
    dialog.warning({
      title: '覆盖无法读取的配置文件？',
      content: () =>
        h('div', { class: 'space-y-2 text-sm' }, [
          h('div', { class: 'break-all text-gray-300' }, path),
          h('div', { class: 'text-gray-400' }, `当前无法解析：${error}`),
          h('div', {}, '保存会用表单内容整体替换该文件，其中原有内容将全部丢失。')
        ]),
      positiveText: '覆盖',
      negativeText: '取消',
      onPositiveClick: () => {
        void persist()
      }
    })
    return
  }

  void persist()
}

async function persist() {
  // saveConfig 成功后状态会变成 loaded，先记下这次是不是新建
  // （invalid 是覆盖已有文件，不能说成"已创建"）
  const creating = projectStore.projectConfigStatus === 'absent'
  saving.value = true
  try {
    // 保存后 projectConfig 被替换，useEclSemanticVocabulary 的 watch 会据此
    // 重新加载 ECL 词表；工具链侧每次调用都现读项目配置，无需额外通知。
    await projectStore.saveConfig(buildConfig(), editingRoot.value)
    message.success(creating ? '已创建 .thtk-project.json' : '项目设置已保存')
    close()
  } catch (error) {
    message.error(`保存失败: ${error}`)
  } finally {
    saving.value = false
  }
}

function close() {
  projectSettingsStore.close()
}

function handleVisibleChange(value) {
  if (!value) {
    close()
  }
}
</script>
