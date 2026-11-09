import { beforeEach, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import TimelinePanel from '../../src/components/Sidebar/TimelinePanel.vue'
import { useEditorStore } from '../../src/stores/editor'
import { analyzeTruthSource } from '../../src/api'
import type { TruthAnalysis, TruthTimelineNode } from '../../src/types'

vi.mock('../../src/api', () => ({ analyzeTruthSource: vi.fn(), readFile: vi.fn().mockResolvedValue('script entry0 {}') }))

function node(id: number, kind: TruthTimelineNode['kind'], name: string, line: number, extra: Partial<TruthTimelineNode> = {}): TruthTimelineNode {
  return { id, kind, name, source: name, line, endLine: line, targetLine: null, children: [], ...extra }
}
const analysis: TruthAnalysis = {
  scripts: [], constants: [], labels: [], jumps: [], events: [],
  timeline: [
    { name: 'entry0', line: 1, endLine: 20, nodes: [
      node(1, 'time', '0', 2, { children: [node(2, 'instruction', 'initialize', 3, { source: 'initialize();' })] }),
      node(3, 'loop', 'loop', 4, { endLine: 10, children: [
        node(4, 'time', '+4', 5, { children: [node(5, 'instruction', 'tick', 6, { source: 'tick(1);' })] })
      ] }),
      node(6, 'interrupt', 'interrupt[1], interrupt[2]', 12),
      node(7, 'jump', 'goto start', 13, { source: 'goto start;', targetLine: 2 })
    ] },
    { name: 'entry1', line: 21, endLine: 25, nodes: [node(8, 'instruction', 'finish', 22, { source: 'finish();' })] }
  ]
}

beforeEach(() => { setActivePinia(createPinia()); vi.mocked(analyzeTruthSource).mockReset() })

it('keeps entry sections and folds a loop independently of interrupt entry points', async () => {
  await useEditorStore().openFile({ path: '/p/st01.stdspec' })
  vi.mocked(analyzeTruthSource).mockResolvedValue(analysis)
  const wrapper = mount(TimelinePanel)
  await flushPromises()
  expect(wrapper.findAll('.timeline-entry')).toHaveLength(2)
  expect(wrapper.text()).toContain('+4 帧')
  expect(wrapper.text()).toContain('tick(1);')
  await wrapper.get('button[aria-label="折叠 loop"]').trigger('click')
  expect(wrapper.text()).not.toContain('tick(1);')
  expect(wrapper.text()).toContain('interrupt[1], interrupt[2]')
  await wrapper.get('button[aria-label="折叠 entry0"]').trigger('click')
  expect(wrapper.text()).not.toContain('interrupt[1]')
  expect(wrapper.text()).toContain('finish();')
  wrapper.unmount()
})

it('navigates to source and jump targets and highlights the cursor event', async () => {
  await useEditorStore().openFile({ path: '/p/st01.stdspec' })
  vi.mocked(analyzeTruthSource).mockResolvedValue(analysis)
  const reveal = vi.fn()
  window.addEventListener('thtk:editor-reveal-location', reveal)
  const wrapper = mount(TimelinePanel)
  await flushPromises()
  await wrapper.get('button[title="tick(1);"]').trigger('click')
  expect((reveal.mock.calls.at(-1)?.[0] as CustomEvent).detail).toMatchObject({ path: '/p/st01.stdspec', line: 6 })
  await wrapper.get('button[title="跳到第 2 行"]').trigger('click')
  expect((reveal.mock.calls.at(-1)?.[0] as CustomEvent).detail.line).toBe(2)
  window.dispatchEvent(new CustomEvent('thtk:editor-cursor-position', { detail: { path: '/p/st01.stdspec', line: 6, column: 1 } }))
  await flushPromises()
  expect(wrapper.get('button[title="tick(1);"]').attributes('aria-current')).toBe('location')
  wrapper.unmount()
  window.removeEventListener('thtk:editor-reveal-location', reveal)
})

it('ignores a previous file analysis that resolves after a newer file', async () => {
  const editor = useEditorStore()
  await editor.openFile({ path: '/p/old.stdspec' })
  let finish!: (data: TruthAnalysis) => void
  vi.mocked(analyzeTruthSource).mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
  const wrapper = mount(TimelinePanel)
  vi.mocked(analyzeTruthSource).mockResolvedValue({ ...analysis, timeline: [{ name: 'new_entry', line: 1, endLine: 2, nodes: [] }] })
  await editor.openFile({ path: '/p/new.msgspec' })
  await flushPromises()
  finish(analysis)
  await flushPromises()
  expect(wrapper.text()).toContain('new_entry')
  expect(wrapper.text()).not.toContain('entry0')
  wrapper.unmount()
})
