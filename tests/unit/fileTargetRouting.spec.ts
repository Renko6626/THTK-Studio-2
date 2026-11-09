import { beforeEach, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useEditorStore } from '../../src/stores/editor'
import { useBuildDialogStore } from '../../src/stores/buildDialog'
import { readFile } from '../../src/api'
vi.mock('../../src/api', () => ({ readFile: vi.fn() }))
beforeEach(() => { setActivePinia(createPinia()); vi.resetAllMocks() })
it('opens path-only binary targets without reading them as text', async () => {
  const editor = useEditorStore()
  await editor.openFile({ path: '/p/st01.msg' })
  expect(editor.activeTab).toMatchObject({ viewType: 'binary-script', extension: 'msg' })
  expect(readFile).not.toHaveBeenCalled()
})
it('uses the current filename rather than stale binary metadata', async () => {
  vi.mocked(readFile).mockResolvedValue('script main {}')
  const editor = useEditorStore()
  await editor.openFile({ path: '/p/st01.msgspec', extension: 'ecl' })
  expect(editor.activeTab).toMatchObject({ viewType: 'text', extension: 'msgspec' })
})
it('does not let an older read steal focus or create duplicate tabs', async () => {
  const editor = useEditorStore()
  let finish!: (text: string) => void
  vi.mocked(readFile).mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
  const pending = editor.openFile({ path: '/p/slow.msgspec' })
  await editor.openFile({ path: '/p/fast.std' })
  finish('script slow {}'); await pending
  expect(editor.activePath).toBe('/p/fast.std')
  expect(editor.tabs).toHaveLength(2)
})
it('restores source views from filenames even with obsolete session metadata', async () => {
  vi.mocked(readFile).mockResolvedValue('script main {}')
  const editor = useEditorStore()
  await editor.restoreSession({ activePath: '/p/a.msgspec', tabs: [{ path: '/p/a.msgspec', name: 'a.msgspec', isDirty: false, language: '', viewType: 'binary-script', extension: 'ecl', size: null, category: 'binaryScript' }] })
  expect(editor.activeTab).toMatchObject({ viewType: 'text', extension: 'msgspec', content: 'script main {}' })
})
it('routes build requests by file type and infers ending mode', () => {
  const dialog = useBuildDialogStore()
  dialog.openDialog({ inputPath: '/p/e01.msg' })
  expect(dialog.payload).toMatchObject({ tool: 'trumsg', mode: 'decompile', scriptKind: 'ending' })
  dialog.openDialog({ inputPath: '/p/e01.msgspec' })
  expect(dialog.payload).toMatchObject({ tool: 'trumsg', mode: 'compile', scriptKind: 'stage' })
  dialog.openDialog({ inputPath: '/p/a.stdspec' })
  expect(dialog.payload).toMatchObject({ tool: 'trustd', mode: 'compile' })
})

it('does not steal focus after the user selects an existing tab during a read', async () => {
  const editor = useEditorStore()
  await editor.openFile({ path: '/p/first.std' })
  await editor.openFile({ path: '/p/second.std' })
  let finish!: (text: string) => void
  vi.mocked(readFile).mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
  const pending = editor.openFile({ path: '/p/slow.msgspec' })
  editor.activePath = '/p/first.std'
  finish('script slow {}'); await pending
  expect(editor.activePath).toBe('/p/first.std')
})
