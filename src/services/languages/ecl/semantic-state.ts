import {
  createEmptyEclSemanticData,
  normalizeEclSemanticData
} from './dynamic-vocabulary'
import type { NormalizedEclSemanticData } from '../../../types'

/** 词表取值回调：provider 按当前 model 拿到对应作用域的词表 */
export type SemanticDataGetter = (model: EclModelLike | null | undefined) => unknown

/** Monaco model 的最小形状；只用到 uri 上的路径 */
export interface EclModelLike {
  uri?: { fsPath?: string; path?: string }
}

const scopedSemanticData = new Map<string, NormalizedEclSemanticData>()
let activeScopeKey = '__global__'

function normalizeScopeKey(scopeKey: string | null | undefined): string {
  const normalized = String(scopeKey || '').trim()
  return normalized || '__global__'
}

function getScopeCandidates(model: EclModelLike | null | undefined): string[] {
  const modelPath = String(model?.uri?.fsPath || model?.uri?.path || '')
  const keys = [...scopedSemanticData.keys()]

  return keys
    .filter((key) => key !== '__global__' && modelPath.startsWith(key))
    .sort((left, right) => right.length - left.length)
}

export function setActiveEclSemanticScope(scopeKey: string | null | undefined): void {
  activeScopeKey = normalizeScopeKey(scopeKey)
}

export function updateScopedEclSemanticData(
  scopeKey: string | null | undefined,
  semanticData: unknown
): void {
  scopedSemanticData.set(
    normalizeScopeKey(scopeKey),
    normalizeEclSemanticData(semanticData)
  )
}

export function clearScopedEclSemanticData(scopeKey: string | null | undefined): void {
  scopedSemanticData.delete(normalizeScopeKey(scopeKey))
}

export function getEclSemanticDataForModel(
  model: EclModelLike | null | undefined
): NormalizedEclSemanticData {
  const matchedScope = getScopeCandidates(model)[0]
  if (matchedScope) {
    return scopedSemanticData.get(matchedScope) || createEmptyEclSemanticData()
  }

  return scopedSemanticData.get(activeScopeKey) || createEmptyEclSemanticData()
}

export function getActiveEclSemanticData(): NormalizedEclSemanticData {
  return scopedSemanticData.get(activeScopeKey) || createEmptyEclSemanticData()
}
