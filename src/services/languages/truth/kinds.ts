import type { TruthScriptKind } from '../../../types'
export function truthKindForPath(path: string | null | undefined): TruthScriptKind | null {
  const extension = path?.split('.').pop()?.toLowerCase()
  return extension === 'msgspec' ? 'msg' : extension === 'endspec' ? 'ending' : extension === 'stdspec' ? 'std' : null
}
