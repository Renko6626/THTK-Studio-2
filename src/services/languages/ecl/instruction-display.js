function getInstructionParams(instruction) {
  return Array.isArray(instruction?.params) ? instruction.params : []
}

export function buildInstructionParameterLabel(param, index) {
  const typeName = param?.type || 'arg'
  const name = param?.name || `arg${index + 1}`
  return `${typeName} ${name}`
}

export function buildInstructionInsertText(instruction) {
  const params = getInstructionParams(instruction)

  if (!params.length) {
    return `${instruction.name}($0);`
  }

  const placeholders = params.map((param, index) => {
    const placeholderIndex = index + 1
    const label = param?.name || `arg${placeholderIndex}`
    return `\${${placeholderIndex}:${label}}`
  })

  return `${instruction.name}(${placeholders.join(', ')});`
}

export function buildInstructionSignature(instruction) {
  const params = getInstructionParams(instruction)
  if (!params.length) {
    if (instruction?.signature) {
      return `${instruction.name}(/* ${instruction.signature} */)`
    }
    return `${instruction.name}(...)`
  }

  const signature = params.map((param, index) => buildInstructionParameterLabel(param, index))

  return `${instruction.name}(${signature.join(', ')})`
}

export function buildInstructionDocumentation(instruction) {
  return {
    value: [
      '```ecl',
      buildInstructionSignature(instruction),
      '```',
      '',
      `opcode: ${instruction.opcode}`,
      instruction.section ? `section: ${instruction.section}` : '',
      instruction.signature ? `signature: ${instruction.signature}` : ''
    ].filter(Boolean).join('\n')
  }
}
