function normalizeStringList(values) {
  return [...new Set(
    (Array.isArray(values) ? values : [])
      .filter((value) => typeof value === 'string' && value.trim())
      .map((value) => value.trim())
  )]
}

export function createEmptyEclSemanticData() {
  return {
    version: '',
    instructions: [],
    builtins: [],
    sourcePath: ''
  }
}

export function normalizeEclSemanticData(data) {
  const instructions = Array.isArray(data?.instructions)
    ? data.instructions
        .filter((item) => typeof item?.name === 'string' && item.name.trim())
        .map((item) => ({
          opcode: Number(item.opcode ?? 0),
          name: item.name.trim(),
          section: typeof item.section === 'string' ? item.section : null,
          signature: typeof item.signature === 'string' ? item.signature : null,
          params: Array.isArray(item.params) ? item.params : []
        }))
    : []

  return {
    version: typeof data?.version === 'string' ? data.version : '',
    sourcePath: typeof data?.sourcePath === 'string' ? data.sourcePath : '',
    instructions,
    builtins: normalizeStringList([
      ...(Array.isArray(data?.builtins) ? data.builtins : []),
      ...instructions.map((item) => item.name)
    ])
  }
}
