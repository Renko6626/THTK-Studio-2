/** Rust modules/truth/maps.rs. Explicit map data, not the compiler's full environment. */
export type TruthScriptKind = 'msg' | 'ending' | 'std'

export interface TruthParameter {
  type: string
  encoding: string
  enumName: string | null
  /** Original attribute values; null denotes a flag such as furibug. */
  attributes: Record<string, string | null>
}

export interface TruthInstruction {
  opcode: number
  description: string | null
  name: string | null
  aliases: string[]
  /** null = unknown signature; empty string = explicitly no arguments. */
  signature: string | null
  params: TruthParameter[] | null
  intrinsic: string | null
  nameSource: string | null
  signatureSource: string | null
}

export interface TruthDefinitions {
  gameVersion: number
  scriptKind: TruthScriptKind
  sourcePaths: string[]
  instructions: TruthInstruction[]
  /** Enum name -> member name -> integer value. */
  enums: Record<string, Record<string, number>>
}

export interface TruthAnalysis {
  events: { script: string; line: number; time: number | null; name: string; kind: string }[]
  labels: { script: string; name: string; line: number }[]
  jumps: { line: number; column: number; endColumn: number; targetLine: number | null }[]
}
