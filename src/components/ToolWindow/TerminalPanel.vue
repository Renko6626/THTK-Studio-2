<template>
  <div class="h-full flex flex-col bg-[#181818]">
    <!--
      会话条。刻意做得比上方 BottomPanelHost 的面板头**轻**：更矮、无独立底色、
      无边框，读起来是"属于终端的标签"，而不是第二条 chrome。
      这里也不再重复画「隐藏」按钮和 Terminal 标题——正上方那条面板头已有。
    -->
    <div
      v-if="terminalStore.sessionCount"
      class="h-8 pl-2 pr-1 flex items-center gap-px shrink-0 overflow-x-auto"
    >
      <div
        v-for="session in terminalStore.sessions"
        :key="session.id"
        class="group panel-tab gap-1.5 cursor-pointer select-none"
        :class="[
          session.id === terminalStore.activeSessionId ? 'panel-tab-active' : '',
          session.exited ? 'opacity-50' : ''
        ]"
        role="tab"
        :aria-selected="session.id === terminalStore.activeSessionId"
        @click="terminalStore.setActive(session.id)"
      >
        <span class="truncate max-w-[14rem]">{{ session.title }}</span>
        <!-- 关闭只在悬停或当前标签上出现，避免一排 × 抢视线（同 VS Code） -->
        <button
          type="button"
          class="w-4 h-4 shrink-0 flex items-center justify-center rounded-[3px]
                 opacity-0 group-hover:opacity-100 focus-visible:opacity-100
                 hover:bg-white/10 transition-opacity"
          :class="session.id === terminalStore.activeSessionId ? 'opacity-100' : ''"
          :title="`关闭 ${session.title}`"
          @click.stop="terminalStore.closeSession(session.id).catch(() => {})"
        >
          <n-icon :size="12"><Dismiss16Regular /></n-icon>
        </button>
      </div>

      <div class="flex items-center ml-1 shrink-0">
        <button
          type="button"
          class="panel-action"
          title="新建终端（默认 shell）"
          @click="terminalStore.openSession()"
        >
          <n-icon :size="16"><Add16Regular /></n-icon>
        </button>
        <n-dropdown
          trigger="click"
          :options="shellOptions"
          placement="bottom-start"
          @select="openWithShell"
        >
          <button type="button" class="panel-action w-4" title="选择 shell 新建终端">
            <n-icon :size="12"><ChevronDown16Regular /></n-icon>
          </button>
        </n-dropdown>
      </div>
    </div>

    <div ref="hostRef" class="flex-1 min-h-0 relative bg-[#181818]">
      <div
        v-if="!terminalStore.sessionCount"
        class="h-full flex flex-col items-center justify-center gap-3 text-sm text-[#9d9d9d]"
      >
        <div>没有打开的终端</div>
        <button
          type="button"
          class="h-7 px-3 rounded-[5px] text-[12px] flex items-center gap-1.5
                 text-[#9d9d9d] hover:text-[#e7e7e7] hover:bg-white/10 transition-colors"
          @click="terminalStore.openSession()"
        >
          <n-icon :size="14"><Add16Regular /></n-icon>
          新建终端
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { NDropdown, NIcon } from 'naive-ui'
import { Add16Regular, ChevronDown16Regular, Dismiss16Regular } from '@vicons/fluent'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { mountAllSessions, showSession, fitSession } from '../../services/terminal/sessionRuntime'

const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const hostRef = ref(null)
let resizeObserver: ResizeObserver | null = null

// 按平台列出常用 shell;key 即传给后端的可执行名(PATH 解析),
// 启动失败时 store 会回退默认探测并提示。
const isWindows = navigator.userAgent.includes('Windows')
const shellOptions = isWindows
  ? [
      { label: 'PowerShell 7 (pwsh)', key: 'pwsh.exe' },
      { label: 'Windows PowerShell', key: 'powershell.exe' },
      { label: 'CMD', key: 'cmd.exe' },
      { label: 'Git Bash', key: 'bash.exe' }
    ]
  : [
      { label: '默认 ($SHELL)', key: '__default__' },
      { label: 'bash', key: 'bash' },
      { label: 'zsh', key: 'zsh' },
      { label: 'fish', key: 'fish' }
    ]

function openWithShell(key: string) {
  if (key === '__default__') {
    terminalStore.openSession()
    return
  }
  const option = shellOptions.find((item) => item.key === key)
  // tab 标题用短名(去掉 .exe / 括号说明)
  const label = key.replace(/\.exe$/i, '')
  terminalStore.openSession({ shell: key, label: option ? label : key })
}

// BottomPanelHost 里终端用 v-show 常驻，本组件挂载时面板未必真的可见；
// 只有终端面板实际可见时才自动开首个会话 / 重新 show（避免启动即建 PTY）。
const isTerminalVisible = computed(
  () => workbenchPanelsStore.bottomVisible && workbenchPanelsStore.activeBottomPanel === 'terminal'
)

function ensureVisibleSession() {
  if (!isTerminalVisible.value) return
  if (!terminalStore.sessionCount) {
    // pendingOpenCount：菜单"新建终端"等入口可能已在创建中，避免重复建会话
    if (!terminalStore.pendingOpenCount) {
      terminalStore.openSession()
    }
  } else if (terminalStore.activeSessionId != null) {
    showSession(terminalStore.activeSessionId)
  }
}

watch(isTerminalVisible, (visible) => {
  if (visible) {
    ensureVisibleSession()
  }
})

onMounted(() => {
  // 重新挂载模块级容器（面板可能被 v-if 重建）
  if (!hostRef.value) return
  mountAllSessions(hostRef.value)
  ensureVisibleSession()

  resizeObserver = new ResizeObserver(() => {
    if (terminalStore.activeSessionId != null) {
      fitSession(terminalStore.activeSessionId)
    }
  })
  resizeObserver.observe(hostRef.value)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})
</script>
