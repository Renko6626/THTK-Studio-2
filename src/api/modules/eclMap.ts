import { invoke } from '@tauri-apps/api/core'
import type { AiPackResult, EclMapSemanticData } from '../../types'

export function getEclMapSemantics(path: string): Promise<EclMapSemanticData> {
  return invoke('get_ecl_map_semantics', { path })
}

export function generateAiAssistPack(force = false): Promise<AiPackResult> {
  return invoke('generate_ai_assist_pack', { force })
}
