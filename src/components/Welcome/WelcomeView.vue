<template>
  <div class="w-full h-full overflow-y-auto flex items-start justify-center px-6 py-12">
    <div class="w-full max-w-[560px]">
      <div class="mb-8 select-none">
        <div class="text-3xl font-mono text-gray-300/30">THTK Studio</div>
        <div class="text-xs text-gray-500 mt-2">东方 Project 脚本与资源魔改工作台</div>
      </div>

      <div class="flex items-center gap-3 mb-10">
        <n-button
          type="primary"
          :loading="projectStore.isLoading"
          :disabled="projectStore.isLoading"
          @click="openFromPicker"
        >
          打开文件夹
        </n-button>
        <span class="text-xs text-gray-600 select-none">Ctrl+O</span>
      </div>

      <div
        v-if="readiness.state !== 'ready'"
        class="rounded border px-3 py-2.5 mb-8 text-xs leading-relaxed"
        :class="readiness.state === 'missing'
          ? 'border-[#cca700]/40 bg-[#cca700]/8 text-[#cca700]'
          : 'border-[#f48771]/40 bg-[#f48771]/8 text-[#f48771]'"
      >
        <div>{{ readiness.message }}</div>
        <n-button size="tiny" quaternary class="mt-2" @click="toolchainSettingsStore.open()">
          打开工具链设置
        </n-button>
      </div>

      <div class="flex items-center justify-between mb-3">
        <div class="text-[11px] uppercase tracking-[0.12em] text-gray-500 select-none">最近项目</div>
        <n-button
          v-if="!recentProjectsStore.isEmpty"
          size="tiny"
          quaternary
          :disabled="projectStore.isLoading"
          @click="confirmClear"
        >
          清空
        </n-button>
      </div>

      <div
        v-if="recentProjectsStore.error"
        class="rounded border border-[#f48771]/40 bg-[#f48771]/8 px-3 py-2 mb-3 text-xs text-[#f48771] break-all"
      >
        最近项目读取失败：{{ recentProjectsStore.error }}
      </div>

      <!-- 读取失败时列表是空的，但那不代表"没打开过项目"，两句话不能同时出现 -->
      <div
        v-if="recentProjectsStore.isEmpty && !recentProjectsStore.error"
        class="text-xs text-gray-600 py-2 select-none"
      >
        还没有打开过项目。
      </div>

      <div v-else class="-mx-3">
        <div
          v-for="item in recentProjectsStore.items"
          :key="item.path"
          class="flex items-center gap-3 px-3 py-2 rounded hover:bg-white/5 focus-within:bg-white/5"
        >
          <button
            type="button"
            class="recent-entry flex-1 min-w-0 text-left disabled:cursor-not-allowed"
            :disabled="projectStore.isLoading"
            @click="openRecent(item)"
          >
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-sm text-gray-100 truncate">{{ item.name }}</span>
              <!-- 失效项仍可点击：路径可能只是临时不可达（U 盘、网络盘），
                   真打不开时后端会给出明确错误 -->
              <span
                v-if="!item.available"
                class="shrink-0 text-[10px] leading-none px-1.5 py-1 rounded border border-[#f48771]/40 text-[#f48771]"
              >
                不可用
              </span>
            </div>
            <div class="text-xs text-gray-500 truncate" :title="item.path">{{ item.path }}</div>
          </button>

          <span class="shrink-0 text-[11px] text-gray-600 select-none">
            {{ formatOpenedAt(item.lastOpenedAt) }}
          </span>

          <n-button
            size="tiny"
            quaternary
            :disabled="projectStore.isLoading"
            :title="`从最近项目移除 ${item.name}`"
            @click="removeRecent(item)"
          >
            移除
          </n-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NButton, useDialog, useMessage } from 'naive-ui'
import { useProjectStore } from '../../stores/project'
import type { RecentProjectView, ToolchainStatus } from '../../types'
import { useRecentProjectsStore } from '../../stores/recentProjects'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
import { useProjectActions } from '../../composables/useProjectActions'
import { getToolchainStatuses } from '../../api'
import { summarizeToolchainReadiness } from '../../services/toolchains/readiness'

const projectStore = useProjectStore()
const recentProjectsStore = useRecentProjectsStore()
const toolchainSettingsStore = useToolchainSettingsStore()
const message = useMessage()
const dialog = useDialog()
const projectActions = useProjectActions({ message, dialog })

// 全新安装时 thtk 工具链一定是未配置的（安装包不含 thtk）。欢迎页是新用户
// 见到的第一个界面，把这件事说在他点任何功能之前。
const toolchainStatuses = ref<ToolchainStatus[]>([])
const readiness = computed(() => summarizeToolchainReadiness(toolchainStatuses.value))

onMounted(async () => {
  void recentProjectsStore.refresh()
  try {
    toolchainStatuses.value = await getToolchainStatuses()
  } catch {
    // 拿不到状态就不提示，不值得为此打断欢迎页
  }
})

async function openFromPicker() {
  await projectActions.openProjectFromPicker()
}

async function openRecent(item: RecentProjectView) {
  await projectActions.openProjectPath(item.path)
}

async function removeRecent(item: RecentProjectView) {
  await projectActions.removeRecentProject(item.path)
}

function confirmClear() {
  dialog.warning({
    title: '清空最近项目？',
    content: '只清空这份列表，不会动到磁盘上的任何项目。',
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await recentProjectsStore.clear()
      } catch (error) {
        message.error(`清空失败: ${error}`)
      }
    }
  })
}

/** 相对时间；超过一周就直接给日期，避免"53 天前"这种没法换算的说法 */
function formatOpenedAt(timestamp: number) {
  if (!timestamp) return ''

  const elapsed = Date.now() - timestamp
  // 时钟回拨或手改过 settings.json 时可能出现未来时间，别显示"-3 分钟前"
  if (elapsed < 0) return ''

  const minutes = Math.floor(elapsed / 60000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} 小时前`

  const days = Math.floor(hours / 24)
  if (days < 7) return `${days} 天前`

  const date = new Date(timestamp)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}
</script>

<style scoped>
/*
 * 本项目没有 CSS reset（uno.config..js 文件名有笔误，UnoCSS 的 preflight 从未生效），
 * 裸 <button> 会拿到 UA 默认的浅灰按钮外观和按钮字体，里面的浅色文字会变成
 * 浅灰底上的浅灰字。同仓库其他裸 button 也都各自做了这件事。
 */
.recent-entry {
  appearance: none;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  padding: 0;
  color: inherit;
  font: inherit;
  cursor: pointer;
}
</style>
