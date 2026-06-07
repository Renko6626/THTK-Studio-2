import { onMounted, onBeforeUnmount } from 'vue'

export function useBeforeUnloadGuard({ hasDirtyTabs, flushSnapshots }) {
  function handleBeforeUnload(event) {
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
