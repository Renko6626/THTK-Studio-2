<template>
  <div class="h-full flex flex-col bg-[#181818] border-t border-black">
    <div class="h-9 px-3 flex items-center justify-between border-b border-white/8 bg-[#202020]">
      <div class="flex items-center gap-1 min-w-0 overflow-x-auto">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-400 mr-2">Terminal</span>
        <button
          v-for="session in terminalStore.sessions"
          :key="session.id"
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border flex items-center gap-1 shrink-0"
          :class="[
            session.id === terminalStore.activeSessionId
              ? 'text-gray-100 border-[#3b82f6] bg-transparent'
              : 'text-gray-400 border-transparent hover:text-gray-200 hover:border-[#3b82f6]/55',
            session.exited ? 'opacity-50' : ''
          ]"
          @click="terminalStore.setActive(session.id)"
        >
          {{ session.title }}
          <span
            class="text-gray-500 hover:text-red-400"
            @click.stop="terminalStore.closeSession(session.id).catch(() => {})"
          >×</span>
        </button>
        <button
          type="button"
          class="h-6 pl-2 pr-1 text-[12px] rounded-l-sm text-gray-400 hover:text-gray-200 hover:bg-white/8 shrink-0"
          title="新建终端(默认 shell)"
          @click="terminalStore.openSession()"
        >
          ＋
        </button>
        <n-dropdown
          trigger="click"
          :options="shellOptions"
          placement="bottom-start"
          @select="openWithShell"
        >
          <button
            type="button"
            class="h-6 px-1 text-[10px] rounded-r-sm text-gray-400 hover:text-gray-200 hover:bg-white/8 shrink-0"
            title="选择 shell 新建终端"
          >
            ▾
          </button>
        </n-dropdown>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="h-6 px-2 text-[11px] rounded-sm border border-transparent bg-transparent text-gray-400 hover:text-gray-200 hover:border-[#3b82f6]/60"
          @click="workbenchPanelsStore.hideBottomPanel()"
        >
          隐藏
        </button>
      </div>
    </div>

    <div ref="hostRef" class="flex-1 min-h-0 relative bg-[#111111]">
      <div
        v-if="!terminalStore.sessionCount"
        class="h-full flex items-center justify-center text-sm text-gray-500"
      >
        没有打开的终端 — 点击 ＋ 新建
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { NDropdown } from 'naive-ui'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import { mountAllSessions, showSession, fitSession } from '../../services/terminal/sessionRuntime'

const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const hostRef = ref(null)
let resizeObserver = null

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

function openWithShell(key) {
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
