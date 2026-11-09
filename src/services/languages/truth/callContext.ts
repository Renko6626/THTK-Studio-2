/** Read just the active call prefix. This is editor context, not a language parser. */
export function truthCallContext(prefix: string): { name: string; parameter: number } | null {
  const stack: { name: string; parameter: number }[] = []
  let quote = false, escaped = false, lineComment = false, blockComment = false
  for (let i = 0; i < prefix.length; i++) {
    const ch = prefix[i], next = prefix[i + 1]
    if (lineComment) { if (ch === '\n') lineComment = false; continue }
    if (blockComment) { if (ch === '*' && next === '/') { blockComment = false; i++ } continue }
    if (quote) { if (escaped) escaped = false; else if (ch === '\\') escaped = true; else if (ch === '"') quote = false; continue }
    if (ch === '/' && next === '/') { lineComment = true; i++; continue }
    if (ch === '/' && next === '*') { blockComment = true; i++; continue }
    if (ch === '"') { quote = true; continue }
    if (ch === '(' || ch === '[' || ch === '{') {
      const name = ch === '(' ? /([A-Za-z_]\w*)\s*$/.exec(prefix.slice(0, i))?.[1] || '' : ''
      stack.push({ name, parameter: 0 })
    } else if (ch === ')' || ch === ']' || ch === '}') stack.pop()
    else if (ch === ',' && stack.length) stack[stack.length - 1].parameter++
  }
  return [...stack].reverse().find(call => call.name !== '') || null
}
