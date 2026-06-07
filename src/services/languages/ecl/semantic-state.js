import {
  createEmptyEclSemanticData,
  normalizeEclSemanticData
} from './dynamic-vocabulary'

const scopedSemanticData = new Map()
let activeScopeKey = '__global__'

function normalizeScopeKey(scopeKey) {
  const normalized = String(scopeKey || '').trim()
  return normalized || '__global__'
}

function getScopeCandidates(model) {
  const modelPath = String(model?.uri?.fsPath || model?.uri?.path || '')
  const keys = [...scopedSemanticData.keys()]

  return keys
    .filter((key) => key !== '__global__' && modelPath.startsWith(key))
    .sort((left, right) => right.length - left.length)
}

export function setActiveEclSemanticScope(scopeKey) {
  activeScopeKey = normalizeScopeKey(scopeKey)
}

export function updateScopedEclSemanticData(scopeKey, semanticData) {
  scopedSemanticData.set(
    normalizeScopeKey(scopeKey),
    normalizeEclSemanticData(semanticData)
  )
}

export function clearScopedEclSemanticData(scopeKey) {
  scopedSemanticData.delete(normalizeScopeKey(scopeKey))
}

export function getEclSemanticDataForModel(model) {
  const matchedScope = getScopeCandidates(model)[0]
  if (matchedScope) {
    return scopedSemanticData.get(matchedScope) || createEmptyEclSemanticData()
  }

  return scopedSemanticData.get(activeScopeKey) || createEmptyEclSemanticData()
}

export function getActiveEclSemanticData() {
  return scopedSemanticData.get(activeScopeKey) || createEmptyEclSemanticData()
}
