import { beforeEach, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import GameVersionSummary from '../../src/components/Common/GameVersionSummary.vue'
import { useProjectStore } from '../../src/stores/project'
import { getSettings } from '../../src/api'
import { makeAppConfig, makeProjectConfig } from '../helpers/fixtures'
vi.mock('../../src/api', () => ({ getSettings: vi.fn(), listGameVersions: vi.fn().mockResolvedValue([]) }))
beforeEach(() => { setActivePinia(createPinia()); vi.mocked(getSettings).mockReset() })
it('shows the actual project version instead of the global default', async () => {
  vi.mocked(getSettings).mockResolvedValue(makeAppConfig({ default_game_version: '20' }))
  useProjectStore().projectConfig = makeProjectConfig({ gameVersion: '16' })
  const wrapper = mount(GameVersionSummary); await flushPromises()
  expect(wrapper.text()).toContain('TH16'); expect(wrapper.text()).toContain('项目设置'); expect(wrapper.text()).not.toContain('TH20')
  wrapper.unmount()
})
it('shows a concrete global fallback and refreshes it after settings change', async () => {
  vi.mocked(getSettings).mockResolvedValue(makeAppConfig({ default_game_version: '20' }))
  const wrapper = mount(GameVersionSummary); await flushPromises()
  expect(wrapper.text()).toContain('TH20'); expect(wrapper.text()).toContain('全局工具链设置')
  vi.mocked(getSettings).mockResolvedValue(makeAppConfig({ default_game_version: '18' }))
  window.dispatchEvent(new Event('thtk:toolchain-settings-saved')); await flushPromises()
  expect(wrapper.text()).toContain('TH18'); wrapper.unmount()
})
it('shows an ECL operation override ahead of the project version', async () => {
  vi.mocked(getSettings).mockResolvedValue(makeAppConfig())
  useProjectStore().projectConfig = makeProjectConfig({ gameVersion: '20' })
  const wrapper = mount(GameVersionSummary, { props: { version: '16' } }); await flushPromises()
  expect(wrapper.text()).toContain('TH16'); expect(wrapper.text()).toContain('本次操作'); wrapper.unmount()
})
it('does not pretend the target is TH20 when the version cannot be read', async () => {
  vi.mocked(getSettings).mockRejectedValue(new Error('offline'))
  const wrapper = mount(GameVersionSummary); await flushPromises()
  expect(wrapper.text()).toContain('无法读取全局版本'); expect(wrapper.text()).not.toContain('TH20'); wrapper.unmount()
})
