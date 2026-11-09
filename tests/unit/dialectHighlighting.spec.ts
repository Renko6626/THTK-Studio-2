import { describe, it, expect, vi } from 'vitest'
vi.mock('../../src/services/languages/truth/register', () => ({ msgLanguageId: 'thtk-truth-msg', endingLanguageId: 'thtk-truth-ending', stdLanguageId: 'thtk-truth-std' }))
import { inferMonacoLanguageId } from '../../src/services/languages/dispatch'
import { dialectLanguageConfiguration } from '../../src/services/languages/dialect-config'
describe('truth source dispatch', () => {
  it('distinguishes stage, ending and background source', () => {
    expect(inferMonacoLanguageId({ path: '/p/e01.msgspec' })).toBe('thtk-truth-msg')
    expect(inferMonacoLanguageId({ path: '/p/renamed.endspec' })).toBe('thtk-truth-ending')
    expect(inferMonacoLanguageId({ path: '/p/st01.stdspec' })).toBe('thtk-truth-std')
    expect(inferMonacoLanguageId({ path: '/p/old.dmsg' })).toBe('plaintext')
  })
  it('keeps native comment editing available', () => {
    expect(dialectLanguageConfiguration.comments?.lineComment).toBe('//')
  })
})
