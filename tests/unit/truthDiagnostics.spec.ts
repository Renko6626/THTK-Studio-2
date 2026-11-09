import { afterEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { useTruthDiagnostics } from '../../src/composables/useTruthDiagnostics'
import { useEditorStore } from '../../src/stores/editor'
import { useWorkbenchReportsStore } from '../../src/stores/workbenchReports'
import { checkTruthSource } from '../../src/api'
import type { EclResult } from '../../src/types'
vi.mock('../../src/api', () => ({ checkTruthSource: vi.fn() }))
const result = (message: string): EclResult => ({ success: !message, tool: 'trumsg', mode: 'check', scriptKind: 'msg', inputPath: '/p/a.msgspec', outputPath: null, message, diagnostics: message ? [{ path: '/p/a.msgspec', line: 3, column: 2, severity: 'error', message }] : [] })
afterEach(() => { vi.useRealTimers(); vi.resetAllMocks() })
describe('truth buffer diagnostics', () => {
  it('checks the unsaved buffer after idle and removes diagnostics when closed', async () => {
    vi.useFakeTimers(); const pinia = createPinia(); setActivePinia(pinia)
    const editor = useEditorStore(), reports = useWorkbenchReportsStore()
    editor.tabs = [{ path: '/p/a.msgspec', name: 'a.msgspec', content: 'unsaved error', originalContent: 'disk', isDirty: true, language: '', viewType: 'text', size: null, extension: 'msgspec', category: 'sourceScript' }]
    editor.activePath = '/p/a.msgspec'
    vi.mocked(checkTruthSource).mockResolvedValue(result('wrong arguments'))
    const wrapper = mount(defineComponent({ setup() { useTruthDiagnostics(); return () => null } }), { global: { plugins: [pinia] } })
    await vi.advanceTimersByTimeAsync(799); expect(checkTruthSource).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(1)
    expect(checkTruthSource).toHaveBeenCalledWith('/p/a.msgspec', 'msg', 'unsaved error')
    expect(reports.problemEntries[0].message).toBe('wrong arguments')
    expect(editor.tabs[0].originalContent).toBe('disk')
    editor.tabs = []; editor.activePath = null; await nextTick()
    expect(reports.problemEntries).toHaveLength(0); wrapper.unmount()
  })
  it('discards a result from a buffer that changed while the compiler ran', async () => {
    vi.useFakeTimers(); const pinia = createPinia(); setActivePinia(pinia)
    const editor = useEditorStore(), reports = useWorkbenchReportsStore()
    editor.tabs = [{ path: '/p/a.msgspec', name: 'a.msgspec', content: 'old', originalContent: '', isDirty: true, language: '', viewType: 'text', size: null, extension: 'msgspec', category: 'sourceScript' }]; editor.activePath = '/p/a.msgspec'
    let finish!: (value: EclResult) => void
    vi.mocked(checkTruthSource).mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    const wrapper = mount(defineComponent({ setup() { useTruthDiagnostics(); return () => null } }), { global: { plugins: [pinia] } })
    await vi.advanceTimersByTimeAsync(800)
    editor.updateContent('/p/a.msgspec', 'new'); await nextTick()
    finish(result('obsolete')); await vi.advanceTimersByTimeAsync(0)
    expect(reports.problemEntries).toHaveLength(0)
    vi.mocked(checkTruthSource).mockResolvedValue(result(''))
    await vi.advanceTimersByTimeAsync(800)
    expect(checkTruthSource).toHaveBeenLastCalledWith('/p/a.msgspec', 'msg', 'new'); wrapper.unmount()
  })
})
