import { invoke } from '@tauri-apps/api/core'

export function getEclMapSemantics(path) {
  return invoke('get_ecl_map_semantics', { path })
}

export function generateAiAssistPack() {
  return invoke('generate_ai_assist_pack')
}
