import { expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { getToolchainDescriptor } from '../../src/services/toolchains/registry'
import { useToolchainActions } from '../../src/composables/useToolchainActions'
import { useEditorStore } from '../../src/stores/editor'
import { useWorkbenchReportsStore } from '../../src/stores/workbenchReports'
import { compileStdFile } from '../../src/api'
vi.mock('../../src/api', () => ({ compileStdFile: vi.fn() }))
it('routes a custom STD output through the real action and reports compiler errors', async () => {
  setActivePinia(createPinia())
  vi.mocked(compileStdFile).mockResolvedValue({ success: false, tool: 'trustd', mode: 'compile', scriptKind: 'std', inputPath: '/p/a.stdspec', outputPath: null, message: 'bad argument', diagnostics: [{ path: '/p/a.stdspec', line: 4, column: 2, severity: 'error', message: 'expects 5 arguments' }] })
  const actions = useToolchainActions()
  const result = await getToolchainDescriptor('trustd')!.runFromPayload!({ runTheclRequest: vi.fn(), runCompileStd: actions.runCompileStd }, { tool: 'trustd', mode: 'compile', inputPath: '/p/a.stdspec', outputPath: '/p/custom.std' })
  expect(compileStdFile).toHaveBeenCalledWith({ inputPath: '/p/a.stdspec', outputPath: '/p/custom.std' })
  expect(result?.success).toBe(false)
  expect(useWorkbenchReportsStore().problemEntries[0]).toMatchObject({ path: '/p/a.stdspec', line: 4, message: 'expects 5 arguments' })
  expect(useEditorStore().compiling).toBe(false)
})
