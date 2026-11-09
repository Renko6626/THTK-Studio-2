import { onMounted, onBeforeUnmount, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'

interface WindowCloseGuardDeps {
  hasDirtyTabs: () => boolean
  confirmClose: () => Promise<boolean>
  flushSnapshots: () => void
  showError: (message: string) => void
}
export function useWindowCloseGuard(deps: WindowCloseGuardDeps) {
  const closingApproved = ref(false)
  let confirming = false, disposed = false, stop: UnlistenFn | undefined
  onMounted(async () => {
    if (!isTauri()) return
    try {
      const unlisten = await getCurrentWindow().onCloseRequested(async event => {
        if (confirming || disposed) { event.preventDefault(); return }
        confirming = true
        try {
          if (deps.hasDirtyTabs() && !await deps.confirmClose()) { event.preventDefault(); return }
          if (disposed) { event.preventDefault(); return }
          deps.flushSnapshots()
          closingApproved.value = true
        } catch (error) { event.preventDefault(); deps.showError(`关闭窗口失败：${String(error)}`) }
        finally { confirming = false }
      })
      if (disposed) unlisten(); else stop = unlisten
    } catch (error) { deps.showError(`注册关闭检查失败：${String(error)}`) }
  })
  onBeforeUnmount(() => { disposed = true; stop?.() })
  return { closingApproved }
}
