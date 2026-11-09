import { expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { useFileWatcher } from '../../src/composables/useFileWatcher'
import { useProjectStore } from '../../src/stores/project'
import { useEditorStore } from '../../src/stores/editor'
import { makeProjectConfig } from '../helpers/fixtures'
import { loadProjectConfig } from '../../src/api'
import type { FileChangeEvent } from '../../src/types'

const { callbacks } = vi.hoisted(() => ({ callbacks: [] as Array<(event: { payload: FileChangeEvent[] }) => Promise<void>> }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn((_name, callback) => { callbacks.push(callback); return Promise.resolve(vi.fn()) }) }))
vi.mock('../../src/api', () => ({
  loadProjectConfig: vi.fn(),
  getFileTree: vi.fn().mockResolvedValue([])
}))

it('reloads a changed project version even when its config file has no editor tab', async () => {
  setActivePinia(createPinia())
  vi.mocked(loadProjectConfig).mockResolvedValue({ status: 'loaded', value: makeProjectConfig({ gameVersion: '16' }), error: null, path: '/p/.thtk-project.json' })
  const project = useProjectStore()
  project.rootPath = '/p'
  project.projectConfig = makeProjectConfig({ gameVersion: '20' })
  const editor = useEditorStore()
  const wrapper = mount(defineComponent({
    setup() {
      useFileWatcher({ editorStore: editor, projectStore: project, showReloadNotice: vi.fn() })
      return () => null
    }
  }))
  await flushPromises()
  await callbacks[0]({ payload: [{ path: '/p/.thtk-project.json', kind: 'modify' }] })
  expect(project.gameVersion).toBe('16')
  wrapper.unmount()
})
