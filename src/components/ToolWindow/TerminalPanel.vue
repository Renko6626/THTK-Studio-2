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
        <span class="truncate max-w-[14rem]" :title="session.title">{{ session.title }}</span>
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

      <div class="flex items-center ml-auto shrink-0">
        <button type="button" class="panel-action" title="查找终端输出（Ctrl/Cmd+F）" aria-label="查找终端输出" @click="openFind(terminalStore.activeSessionId)">
          <n-icon :size="16"><Search16Regular /></n-icon>
        </button>
        <n-dropdown trigger="click" :options="actionOptions" placement="bottom-end" @select="handleMenuAction">
          <button type="button" class="panel-action" title="终端操作" aria-label="终端操作"><n-icon :size="16"><MoreHorizontal16Regular /></n-icon></button>
        </n-dropdown>
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

    <TerminalFindBar
      v-if="findVisible"
      ref="findBar"
      v-model:query="query"
      v-model:case-sensitive="caseSensitive"
      :found="found"
      @search="search"
      @close="closeFind(true)"
    />

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
import { computed, h, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { NDropdown, NIcon, NInput, useDialog, useMessage } from 'naive-ui'
import { Add16Regular, ChevronDown16Regular, Dismiss16Regular, Search16Regular, MoreHorizontal16Regular } from '@vicons/fluent'
import { useTerminalStore } from '../../stores/terminal'
import { useWorkbenchPanelsStore } from '../../stores/workbenchPanels'
import {
  mountAllSessions, showSession, fitSession, focusSession,
  runTerminalAction, findInSession, clearSessionSearch
} from '../../services/terminal/sessionRuntime'
import {
  dispatchTerminalAction, terminalActionEvent, terminalErrorEvent,
  type TerminalAction, type TerminalActionRequest
} from '../../composables/useTerminalActionBridge'
import TerminalFindBar from './TerminalFindBar.vue'

const terminalStore = useTerminalStore()
const workbenchPanelsStore = useWorkbenchPanelsStore()
const hostRef = ref<HTMLElement | null>(null)
const message = useMessage()
const dialog = useDialog()
const findVisible = ref(false)
const query = ref('')
const caseSensitive = ref(false)
const found = ref<boolean | null>(null)
const findBar = ref<InstanceType<typeof TerminalFindBar> | null>(null)
let searchSessionId: number | null = null

const panelActions: { label: string; key: TerminalAction }[] = [
  { label: '复制选区', key: 'copy' },
  { label: '粘贴', key: 'paste' },
  { label: '全选', key: 'selectAll' },
  { label: '清屏', key: 'clear' },
  { label: '滚到底部', key: 'scrollBottom' },
  { label: '重命名', key: 'rename' },
  { label: '关闭终端', key: 'close' }
]
const actionOptions = computed(() => panelActions.map(item => ({
  ...item,
  disabled: !terminalStore.activeSession || (item.key === 'paste' && terminalStore.activeSession.exited)
})))

function handleMenuAction(key: string) {
  const action = panelActions.find(item => item.key === key)?.key
  const id = terminalStore.activeSessionId
  if (action && id != null) dispatchTerminalAction(id, action)
}

function openFind(id: number | null) {
  if (id == null || id !== terminalStore.activeSessionId) return
  searchSessionId = id
  findVisible.value = true
  void nextTick(() => {
    requestAnimationFrame(() => findBar.value?.focus())
  })
}

function closeFind(restoreFocus = false) {
  const id = searchSessionId
  if (id != null && query.value) clearSessionSearch(id)
  findVisible.value = false
  query.value = ''
  found.value = null
  searchSessionId = null
  if (restoreFocus && id != null) focusSession(id)
}

function search(direction: 'next' | 'previous') {
  if (!findVisible.value || searchSessionId == null) return
  try {
    found.value = findInSession(searchSessionId, query.value, direction, caseSensitive.value)
  } catch (error) {
    message.error(`终端查找失败：${String(error)}`)
  }
}
watch([query, caseSensitive], () => search('next'))
watch(() => terminalStore.activeSessionId, () => closeFind())

function renameSession(id: number) {
  const session = terminalStore.sessions.find(item => item.id === id)
  if (!session) return
  const title = ref(session.title)
  dialog.create({
    title: '重命名终端',
    content: () => h(NInput, {
      value: title.value,
      autofocus: true,
      placeholder: '终端名称',
      'onUpdate:value': (value: string) => { title.value = value }
    }),
    positiveText: '保存',
    negativeText: '取消',
    onPositiveClick: () => terminalStore.renameSession(id, title.value)
  })
}

async function handleAction(event: Event) {
  const { sessionId, action } = (event as CustomEvent<TerminalActionRequest>).detail
  if (!terminalStore.sessions.some(session => session.id === sessionId)) return
  try {
    switch (action) {
      case 'find': openFind(sessionId); break
      case 'togglePanel': workbenchPanelsStore.toggleBottomPanel('terminal'); break
      case 'rename': renameSession(sessionId); break
      case 'close': await terminalStore.closeSession(sessionId); break
      default:
        await runTerminalAction(sessionId, action)
        if (sessionId === terminalStore.activeSessionId) focusSession(sessionId)
    }
  } catch (error) {
    message.error(String(error))
  }
}
function handleError(event: Event) {
  message.error((event as CustomEvent<string>).detail)
}
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
  } else {
    closeFind()
  }
})

onMounted(() => {
  window.addEventListener(terminalActionEvent, handleAction)
  window.addEventListener(terminalErrorEvent, handleError)
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
  closeFind()
  window.removeEventListener(terminalActionEvent, handleAction)
  window.removeEventListener(terminalErrorEvent, handleError)
  resizeObserver?.disconnect()
})
</script>
