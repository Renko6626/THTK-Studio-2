import { expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import WorkbenchEditorHost from '../../src/components/Editor/WorkbenchEditorHost.vue'
import { useEditorStore } from '../../src/stores/editor'

vi.mock('../../src/api', () => ({ readFile: vi.fn().mockResolvedValue('# Notes') }))
vi.mock('../../src/components/Editor/MonacoEditor.vue', () => ({
  default: { name: 'MonacoEditor', template: '<div data-editor="text">Text editor</div>' }
}))
vi.mock('../../src/components/Editor/BinaryScriptView.vue', () => ({
  default: { name: 'BinaryScriptView', template: '<div data-editor="binary">Binary script</div>' }
}))

it('switches between an MSG binary page and an existing Markdown tab', async () => {
  const pinia = createPinia()
  setActivePinia(pinia)
  const editor = useEditorStore()
  await editor.openFile({ path: '/p/notes.md' })
  await editor.openFile({ path: '/p/st01.msg' })
  const wrapper = mount(WorkbenchEditorHost, { global: { plugins: [pinia] } })
  expect(wrapper.find('[data-editor="binary"]').exists()).toBe(true)
  await editor.openFile({ path: '/p/notes.md' })
  await nextTick()
  expect(editor.activeTab?.viewType).toBe('text')
  expect(wrapper.find('[data-editor="text"]').exists()).toBe(true)
  expect(wrapper.find('[data-editor="binary"]').exists()).toBe(false)
  await editor.openFile({ path: '/p/st01.msg' })
  await nextTick()
  expect(wrapper.find('[data-editor="binary"]').exists()).toBe(true)
  await editor.openFile({ path: '/p/notes.md' })
  await nextTick()
  expect(wrapper.find('[data-editor="text"]').exists()).toBe(true)
  wrapper.unmount()
})
