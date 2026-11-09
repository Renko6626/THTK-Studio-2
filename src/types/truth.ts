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
  constants: { script: string; name: string; line: number }[]
  scripts: { name: string; line: number; endLine: number }[]
  events: { script: string; line: number; time: number | null; name: string; kind: string }[]
  labels: { script: string; name: string; line: number }[]
  jumps: { line: number; column: number; endColumn: number; targetLine: number | null }[]
  timeline: TruthTimelineEntry[]
}

/** Source structure from Rust timeline.rs. Time names retain their original +/- spelling. */
export interface TruthTimelineNode {
  id: number
  kind: 'time' | 'instruction' | 'loop' | 'branch' | 'block' | 'interrupt' | 'label' | 'jump'
  name: string
  source: string
  line: number
  endLine: number
  /** Jump destination, or a loop's continuation/condition location. */
  targetLine: number | null
  children: TruthTimelineNode[]
}

export interface TruthTimelineEntry {
  name: string
  line: number
  endLine: number
  nodes: TruthTimelineNode[]
}

/** Rust modules/truth/ai_pack.rs. */
export interface TruthAssistPackResult {
  skillPath: string
  skillWritten: boolean
  skillExisted: boolean
  referenceFiles: string[]
  projectRulesPath: string
  projectRulesWritten: boolean
  suggestedRulesText: string
}
