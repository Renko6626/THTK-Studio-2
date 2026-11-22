import { invoke } from '@tauri-apps/api/core'
import type { AiPackResult, EclInstructionReference, EclMapSemanticData } from '../../types'

export function getEclMapSemantics(path: string, overlayPaths?: string[]): Promise<EclMapSemanticData> {
  return invoke('get_ecl_map_semantics', { path, overlayPaths })
}

export function getEclInstructionReference(version: string): Promise<EclInstructionReference[]> {
  return invoke('get_ecl_instruction_reference', { version })
}

export function generateAiAssistPack(force = false): Promise<AiPackResult> {
  return invoke('generate_ai_assist_pack', { force })
}
