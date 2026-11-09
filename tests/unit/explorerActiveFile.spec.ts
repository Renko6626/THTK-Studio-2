import { beforeEach, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, nextTick, ref } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { useExplorerActiveFile } from '../../src/composables/useExplorerActiveFile'
import { useEditorStore } from '../../src/stores/editor'
import { useExplorerViewStore } from '../../src/stores/explorerView'
import { useProjectStore } from '../../src/stores/project'
import { getDirChildren } from '../../src/api'
import { makeFileNode } from '../helpers/fixtures'
vi.mock('../../src/api', () => ({ getDirChildren: vi.fn() }))
beforeEach(() => { setActivePinia(createPinia()); vi.resetAllMocks() })
it('reveals an active tab through lazy folders and follows tab changes', async () => {
  const editor = useEditorStore(), explorer = useExplorerViewStore(), project = useProjectStore()
  project.rootPath = '/p'; project.files = [makeFileNode({ path: '/p/scripts', is_dir: true, isLeaf: false })]
  vi.mocked(getDirChildren).mockResolvedValue([makeFileNode({ path: '/p/scripts/a.msgspec' })])
  const expanded = ref<string[]>([]), reveal = vi.fn()
  const wrapper = mount(defineComponent({ setup() { useExplorerActiveFile(editor, project, explorer, expanded, vi.fn(), reveal); return () => null } }))
  editor.activePath = '/p/scripts/a.msgspec'; await flushPromises()
  expect(explorer.selectedPaths).toEqual(['/p/scripts/a.msgspec']); expect(expanded.value).toContain('/p/scripts')
  expect(getDirChildren).toHaveBeenCalledWith('/p/scripts')
  expect(reveal).toHaveBeenCalledWith('/p/scripts/a.msgspec')
  explorer.setSelectedPaths(['/p/scripts/a.msgspec', '/p/b.std']); await nextTick()
  expect(explorer.selectedPaths).toHaveLength(2)
  editor.activePath = '/p/b.std'; await nextTick()
  expect(explorer.selectedPaths).toEqual(['/p/b.std'])
  editor.activePath = null; await nextTick(); expect(explorer.selectedPaths).toEqual([])
  wrapper.unmount()
})
it('does not overwrite a newer tab selection when an old lazy load completes', async () => {
  const editor = useEditorStore(), explorer = useExplorerViewStore(), project = useProjectStore()
  project.rootPath = '/p'; project.files = [makeFileNode({ path: '/p/scripts', is_dir: true, isLeaf: false })]
  let finish!: (files: ReturnType<typeof makeFileNode>[]) => void
  vi.mocked(getDirChildren).mockReturnValue(new Promise(resolve => { finish = resolve }))
  const wrapper = mount(defineComponent({ setup() { useExplorerActiveFile(editor, project, explorer, ref([]), vi.fn()); return () => null } }))
  editor.activePath = '/p/scripts/a.msgspec'; await nextTick()
  editor.activePath = '/p/b.std'; await nextTick()
  finish([]); await flushPromises()
  expect(explorer.selectedPaths).toEqual(['/p/b.std']); wrapper.unmount()
})
