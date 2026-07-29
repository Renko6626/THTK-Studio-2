import type { EclInstructionSpec, NormalizedEclSemanticData } from '../../../types'

function normalizeStringList(values: unknown): string[] {
  return [...new Set(
    (Array.isArray(values) ? values : [])
      .filter((value): value is string => typeof value === 'string' && Boolean(value.trim()))
      .map((value) => value.trim())
  )]
}

export function createEmptyEclSemanticData(): NormalizedEclSemanticData {
  return {
    version: '',
    instructions: [],
    builtins: [],
    sourcePath: ''
  }
}

/**
 * 把来源不定的词表数据归一化成稳定形状。
 *
 * 入参故意是 unknown：它可能来自 Rust（EclMapSemanticData）、来自加载器
 * （LoadedEclSemanticData），也可能是调用方拼的部分对象。所有字段都逐个校验。
 *
 * 注意产物**不含** globals，且 builtins 会并入全部指令名——见
 * types/ecl.ts 里三个词表类型的说明。
 */
export function normalizeEclSemanticData(data: unknown): NormalizedEclSemanticData {
  const source = (data ?? {}) as Record<string, unknown>

  const instructions: EclInstructionSpec[] = Array.isArray(source.instructions)
    ? (source.instructions as Record<string, unknown>[])
        .filter((item) => typeof item?.name === 'string' && Boolean((item.name as string).trim()))
        .map((item) => ({
          opcode: Number(item.opcode ?? 0),
          name: (item.name as string).trim(),
          section: typeof item.section === 'string' ? item.section : null,
          signature: typeof item.signature === 'string' ? item.signature : null,
          params: Array.isArray(item.params) ? item.params : []
        }))
    : []

  return {
    version: typeof source.version === 'string' ? source.version : '',
    sourcePath: typeof source.sourcePath === 'string' ? source.sourcePath : '',
    instructions,
    builtins: normalizeStringList([
      ...(Array.isArray(source.builtins) ? source.builtins : []),
      ...instructions.map((item) => item.name)
    ])
  }
}
