import { expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import BuildConfigDialog from '../../src/components/Dialogs/BuildConfigDialog.vue'
import { useBuildDialogStore } from '../../src/stores/buildDialog'
import { useProjectStore } from '../../src/stores/project'
import { makeProjectConfig } from '../helpers/fixtures'
vi.mock('../../src/api', () => ({ getSettings: vi.fn().mockResolvedValue({ default_game_version: '18', eclmap_path: '' }) }))
vi.mock('../../src/composables/useTheclActions', () => ({ useTheclActions: () => ({ runTheclRequest: vi.fn() }) }))
vi.mock('../../src/composables/useToolchainActions', () => ({ useToolchainActions: () => ({}) }))
vi.mock('naive-ui', async original => ({ ...await original<typeof import('naive-ui')>(), useMessage: () => ({ warning: vi.fn() }) }))
it('replaces an already visible ECL form with the new MSG and STD targets', async () => {
  const pinia = createPinia(); setActivePinia(pinia)
  const store = useBuildDialogStore()
  useProjectStore().projectConfig = makeProjectConfig({ gameVersion: '16' })
  const wrapper = mount(BuildConfigDialog, { global: { plugins: [pinia], stubs: {
    BuildDialogShell: { props: ['show', 'title'], template: '<div v-if="show"><span>{{ title }}</span><slot /></div>' },
    TheclBuildForm: true, MsgBuildForm: true, StdBuildForm: true
  } } })
  store.openDialog({ tool: 'thecl', inputPath: '/p/a.decl' }); await flushPromises()
  expect(wrapper.findComponent({ name: 'TheclBuildForm' }).exists()).toBe(true)
  expect(wrapper.findComponent({ name: 'TheclBuildForm' }).props('model').version).toBe('16')
  store.openDialog({ inputPath: '/p/e01.msgspec' }); await nextTick()
  expect(wrapper.findComponent({ name: 'MsgBuildForm' }).props('model')).toMatchObject({ tool: 'trumsg', inputPath: '/p/e01.msgspec', scriptKind: 'stage' })
  expect(wrapper.findComponent({ name: 'TheclBuildForm' }).exists()).toBe(false)
  store.openDialog({ inputPath: '/p/a.std' }); await nextTick()
  expect(wrapper.findComponent({ name: 'StdBuildForm' }).props('model')).toMatchObject({ tool: 'trustd', inputPath: '/p/a.std', mode: 'decompile' })
  wrapper.unmount()
})
