function nameBeforeParen(prefix: string, paren: number): string {
  let end = paren
  while (end > 0 && /\s/.test(prefix[end - 1])) end--
  let start = end
  while (start > 0 && /\w/.test(prefix[start - 1])) start--
  if (start === end || !/[A-Za-z_]/.test(prefix[start])) return ''
  // ECL @name(...) calls a script subroutine, even if a map uses the same name.
  if (prefix[start - 1] === '@') start--
  return prefix.slice(start, end)
}

/** Shared editing context for nested calls, quoted strings and comments. */
export function getCallContext(prefix: string): { name: string; parameter: number } | null {
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
      const name = ch === '(' ? nameBeforeParen(prefix, i) : ''
      stack.push({ name, parameter: 0 })
    } else if (ch === ')' || ch === ']' || ch === '}') stack.pop()
    else if (ch === ',' && stack.length) stack[stack.length - 1].parameter++
  }
  return [...stack].reverse().find(call => call.name !== '') || null
}
