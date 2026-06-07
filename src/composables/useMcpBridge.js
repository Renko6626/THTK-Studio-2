// src/composables/useMcpBridge.js
// 监听 Rust MCP server 转发的 agent 报告,落入输出/问题面板。
import { onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useWorkbenchReportsStore } from '../stores/workbenchReports'
import { useWorkbenchPanelsStore } from '../stores/workbenchPanels'

export function useMcpBridge() {
  const reportsStore = useWorkbenchReportsStore()
  const workbenchPanelsStore = useWorkbenchPanelsStore()
  let unlisten = null

  listen('mcp://report', ({ payload }) => {
    if (!payload) return
    reportsStore.publishToolResult({
      ownerKey: `agent:${payload.title || 'report'}:${Date.now()}`,
      source: 'agent',
      operation: 'report',
      scriptKind: 'text',
      title: payload.title || 'Agent 报告',
      path: payload.path || null,
      success: payload.level !== 'error',
      message: payload.body || '',
      diagnostics: []
    })
    // agent 主动汇报时把输出面板带到前台,让人类看到
    workbenchPanelsStore.showBottomPanel('output')
  }).then((fn) => {
    unlisten = fn
  })

  onBeforeUnmount(() => {
    unlisten?.()
  })
}
