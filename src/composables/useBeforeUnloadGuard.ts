import { onMounted, onBeforeUnmount } from 'vue'

export interface BeforeUnloadGuardDeps {
  hasDirtyTabs: () => boolean
  flushSnapshots: () => void
}

export function useBeforeUnloadGuard({ hasDirtyTabs, flushSnapshots }: BeforeUnloadGuardDeps) {
  function handleBeforeUnload(event: BeforeUnloadEvent) {
    flushSnapshots()

    if (!hasDirtyTabs()) return

    event.preventDefault()
    event.returnValue = ''
  }

  onMounted(() => {
    window.addEventListener('beforeunload', handleBeforeUnload)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
  })
}
