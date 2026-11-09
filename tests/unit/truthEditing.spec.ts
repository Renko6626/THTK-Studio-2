import { describe, expect, it } from 'vitest'
import { truthCallContext } from '../../src/services/languages/truth/callContext'
import { truthTokens } from '../../src/services/languages/truth/register'
import { createTokenizer } from '../helpers/monarch'

describe('truth editing', () => {
  it('counts arguments outside strings, nested calls and array expressions', () => {
    expect(truthCallContext('script main { text("a,b", nested(1, 2), ')).toEqual({ name: 'text', parameter: 2 })
    expect(truthCallContext('foo([1, 2], ')).toEqual({ name: 'foo', parameter: 1 })
    expect(truthCallContext('foo(1, /* , */ ')).toEqual({ name: 'foo', parameter: 1 })
    expect(truthCallContext('foo(1); // bar(')).toBeNull()
  })
  it('colors calls, raw opcodes, labels, strings and control keywords', () => {
    const tokenize = createTokenizer('truth', truthTokens)
    const tokens = tokenize('start: text("comma, // text"); ins_17("a"); if (true) {')
    const token = (text: string) => tokens.find(t => t.text.trim() === text)?.type
    expect(token('start')).toContain('entity.name.label')
    expect(token('text')).toContain('entity.name.function')
    expect(token('ins_17')).toContain('unresolved')
    expect(token('if')).toContain('keyword')
    expect(tokens.find(t => t.text.includes('comma'))?.type).toContain('string')
  })
})
