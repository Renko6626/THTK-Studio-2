import { invoke } from '@tauri-apps/api/core'
import type { TruthAnalysis, TruthDefinitions, TruthScriptKind } from '../../types'

/** Paths are explicit and ordered; no builtin or environment maps are injected. */
export function getTruthMapDefinitions(
  gameVersion: number,
  scriptKind: TruthScriptKind,
  mapPaths: string[]
): Promise<TruthDefinitions> {
  return invoke('get_truth_map_definitions', { gameVersion, scriptKind, mapPaths })
}

export function getTruthSourceDefinitions(inputPath: string, scriptKind: TruthScriptKind, sourceText: string): Promise<TruthDefinitions> {
  return invoke('get_truth_source_definitions', { inputPath, scriptKind, sourceText })
}
export function analyzeTruthSource(sourceText: string): Promise<TruthAnalysis> {
  return invoke('analyze_truth_source', { sourceText })
}

export function generateTruthAssistPack(): Promise<string> { return invoke('generate_truth_assist_pack') }
