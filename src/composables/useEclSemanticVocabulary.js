import { onBeforeUnmount, onMounted, watch } from 'vue'
import { loadDefaultEclSemanticData } from '../services/languages/ecl/semantic-loader'
import {
  clearEclSemanticVocabulary,
  updateEclSemanticVocabulary
} from '../services/languages/ecl/register'

export function useEclSemanticVocabulary({
  projectStore,
  showReloadNotice
}) {
  let disposed = false
  let loadingToken = 0
  let previousScopeKey = '__global__'

  function getScopeKey() {
    return projectStore.rootPath || '__global__'
  }

  async function refreshSemanticVocabulary() {
    const token = ++loadingToken
    const scopeKey = getScopeKey()

    try {
      const semanticData = await loadDefaultEclSemanticData({
        projectRoot: projectStore.rootPath
      })

      if (disposed || token !== loadingToken) return

      if (semanticData?.instructions?.length) {
        updateEclSemanticVocabulary(scopeKey, semanticData)
        showReloadNotice?.(`已加载 ECL 词表: ${semanticData.version || 'unknown'} (${semanticData.instructions.length} 条指令)`)
        return
      }

      clearEclSemanticVocabulary(scopeKey)
    } catch {
      if (disposed || token !== loadingToken) return
      clearEclSemanticVocabulary(scopeKey)
    }
  }

  function handleToolchainSettingsChanged() {
    void refreshSemanticVocabulary()
  }

  watch(
    () => projectStore.rootPath,
    (nextPath) => {
      const nextScopeKey = nextPath || '__global__'
      if (previousScopeKey && previousScopeKey !== nextScopeKey) {
        clearEclSemanticVocabulary(previousScopeKey)
      }
      previousScopeKey = nextScopeKey
      void refreshSemanticVocabulary()
    }
  )

  onMounted(() => {
    window.addEventListener('thtk:toolchain-settings-saved', handleToolchainSettingsChanged)
    void refreshSemanticVocabulary()
  })

  onBeforeUnmount(() => {
    disposed = true
    window.removeEventListener('thtk:toolchain-settings-saved', handleToolchainSettingsChanged)
  })

  return {
    refreshSemanticVocabulary
  }
}
