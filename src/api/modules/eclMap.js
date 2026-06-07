import { invoke } from '@tauri-apps/api/core'

export function getEclMapSemantics(path) {
  return invoke('get_ecl_map_semantics', { path })
}
