import { afterEach, describe, expect, it, vi } from 'vitest'
import { editor, languages } from 'monaco-editor/esm/vs/editor/editor.api'
import type { CancellationToken } from 'monaco-editor'
import th18 from '../../src-tauri/assets/ecl-reference/th18.json?raw'
import { createEclCompletionProvider } from '../../src/services/languages/ecl/completion-provider'
import { createEclHoverProvider } from '../../src/services/languages/ecl/hover-provider'
import { createEclSignatureHelpProvider } from '../../src/services/languages/ecl/signature-help-provider'
import type { EclGameReference, EclInstructionSpec } from '../../src/types'

vi.mock('monaco-editor', async () => import('monaco-editor/esm/vs/editor/editor.api'))

const reference: EclGameReference = { version: '18', instructions: JSON.parse(th18).instructions }
const token = {} as CancellationToken
const signatureContext = { triggerKind: languages.SignatureHelpTriggerKind.Invoke, isRetrigger: false }
const movement: EclInstructionSpec = {
  opcode: 404, name: 'customMove', section: null, signature: 'ff',
  params: [{ name: 'float1', type: 'float' }, { name: 'float2', type: 'float' }]
}
afterEach(() => editor.getModels().forEach(model => model.dispose()))

describe('ECL 编辑器参数契约', () => {
  it('项目别名的补全、悬停和输入提示使用同一份逐作参数', async () => {
    const data = () => ({ reference, instructions: [movement] })
    const model = editor.createModel('customMove(\n  (1 + 2), ')
    const start = model.getPositionAt(2)
    const hover = await createEclHoverProvider(data).provideHover(model, start, token)
    expect(hover?.contents.map(part => part.value).join('\n')).toContain('px/frame')
    const result = await createEclCompletionProvider(data).provideCompletionItems(model, start,
      { triggerKind: languages.CompletionTriggerKind.Invoke }, token)
    const completion = result?.suggestions.find(item => item.label === 'customMove')
    expect(completion?.insertText).toBe('customMove(${1:angle}, ${2:speed});')
    expect(result?.suggestions.some(item => item.label === 'move_velocity_abs')).toBe(false)
    const help = await createEclSignatureHelpProvider(data).provideSignatureHelp(model,
      model.getPositionAt(model.getValueLength()), token, signatureContext)
    expect(help?.value.activeParameter).toBe(1)
    expect(help?.value.signatures[0].parameters[1].documentation).toContain('px/frame')
  })

  it('原始编号可独立查表，可变实参始终使用尾项说明', async () => {
    const provider = createEclSignatureHelpProvider(() => ({ reference, instructions: [] }))
    const model = editor.createModel('ins_11("sub,a", 1, 2, ')
    const help = await provider.provideSignatureHelp(model, model.getPositionAt(model.getValueLength()), token, signatureContext)
    expect(help?.value.activeParameter).toBe(1)
    expect(help?.value.signatures[0].parameters[1].label).toBe('args…')
    expect(help?.value.signatures[0].parameters[1].documentation).toContain('按值')
    model.setValue('ins_324(')
    const positionHelp = await provider.provideSignatureHelp(model, model.getPositionAt(model.getValueLength()), token, signatureContext)
    expect(positionHelp?.value.signatures[0].parameters[0].documentation).toContain('可写变量')
  })

  it('参数按位置合并；项目覆盖为空签名时优先采用零参数', async () => {
    const row = reference.instructions.find(item => item.opcode === 404)!
    const partial = { ...reference, instructions: [{ ...row, params: row.params?.slice(1) }] }
    let instruction = movement
    const provider = createEclSignatureHelpProvider(() => ({ reference: partial, instructions: [instruction] }))
    const model = editor.createModel('customMove(')
    const help = () => provider.provideSignatureHelp(model, model.getPositionAt(model.getValueLength()), token, signatureContext)
    expect((await help())?.value.signatures[0].label).toBe('customMove(float float1, float speed)')
    instruction = { ...movement, signature: '', params: [] }
    expect((await help())?.value.signatures[0].label).toBe('customMove()')
  })
})
