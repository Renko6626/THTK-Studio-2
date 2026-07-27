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
        projectRoot: projectStore.rootPath,
        projectConfig: projectStore.projectConfig
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

  // 全局工具链设置不在 Pinia 里，只能靠事件通知
  function handleToolchainSettingsChanged() {
    void refreshSemanticVocabulary()
  }

  // 同时盯住 projectConfig：词表来源里的版本、eclmap 和 thtk 目录都来自它。
  // 只盯 rootPath 不够——loadProject 是先同步设 rootPath、await 之后才填
  // projectConfig，只监听前者会用上一个项目的配置加载词表且再也不刷新。
  // 这条 watch 同时覆盖了"保存项目设置后立即生效"，不需要额外的事件。
  watch(
    [() => projectStore.rootPath, () => projectStore.projectConfig],
    ([nextPath]) => {
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
