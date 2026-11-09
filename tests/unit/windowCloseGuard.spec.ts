import { afterEach, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { useWindowCloseGuard } from '../../src/composables/useWindowCloseGuard'
import { useBeforeUnloadGuard } from '../../src/composables/useBeforeUnloadGuard'
const native = vi.hoisted(() => ({ handler: null as null | ((event: { preventDefault(): void }) => Promise<void>), stop: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ onCloseRequested: async (handler: typeof native.handler) => { native.handler = handler; return native.stop } }) }))
afterEach(() => { vi.clearAllMocks(); native.handler = null })
it('allows cancellation and flushes the session only when closing is approved', async () => {
  const confirmClose = vi.fn().mockResolvedValueOnce(false).mockResolvedValueOnce(true)
  const flushSnapshots = vi.fn(), showError = vi.fn()
  const wrapper = mount(defineComponent({ setup() {
    const { closingApproved } = useWindowCloseGuard({ hasDirtyTabs: () => true, confirmClose, flushSnapshots, showError })
    useBeforeUnloadGuard({ hasDirtyTabs: () => !closingApproved.value, flushSnapshots })
    return () => null
  } }))
  await flushPromises()
  const cancel = { preventDefault: vi.fn() }
  await native.handler!(cancel)
  expect(cancel.preventDefault).toHaveBeenCalled(); expect(flushSnapshots).not.toHaveBeenCalled()
  const accept = { preventDefault: vi.fn() }
  await native.handler!(accept)
  expect(accept.preventDefault).not.toHaveBeenCalled(); expect(flushSnapshots).toHaveBeenCalledOnce()
  const unload = new Event('beforeunload', { cancelable: true })
  window.dispatchEvent(unload)
  expect(unload.defaultPrevented).toBe(false)
  wrapper.unmount(); expect(native.stop).toHaveBeenCalledOnce()
})
it('blocks a repeated native close while confirmation is pending', async () => {
  let resolve!: (accepted: boolean) => void
  const confirmClose = vi.fn(() => new Promise<boolean>(done => { resolve = done })), flushSnapshots = vi.fn()
  const wrapper = mount(defineComponent({ setup() { useWindowCloseGuard({ hasDirtyTabs: () => true, confirmClose, flushSnapshots, showError: vi.fn() }); return () => null } }))
  await flushPromises()
  const first = native.handler!({ preventDefault: vi.fn() })
  const duplicate = { preventDefault: vi.fn() }
  await native.handler!(duplicate)
  expect(duplicate.preventDefault).toHaveBeenCalled(); expect(confirmClose).toHaveBeenCalledOnce()
  resolve(false); await first; expect(flushSnapshots).not.toHaveBeenCalled(); wrapper.unmount()
})
