import { onBeforeUnmount, onMounted, watch } from 'vue'
import { loadDefaultEclSemanticData } from '../services/languages/ecl/semantic-loader'
import {
  clearEclSemanticVocabulary,
  updateEclSemanticVocabulary
} from '../services/languages/ecl/register'
import type { useProjectStore } from '../stores/project'

export interface EclSemanticVocabularyDeps {
  projectStore: ReturnType<typeof useProjectStore>
  showReloadNotice?: (text: string) => void
}

export function useEclSemanticVocabulary({
  projectStore,
  showReloadNotice
}: EclSemanticVocabularyDeps) {
  let disposed = false
  let loadingToken = 0
  let previousScopeKey = '__global__'

  function getScopeKey(): string {
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

  // 同时盯住 projectConfig：词表来源里的版本、eclmap 和 thtk 目录都来自它，
  // 而它在打开项目之外还会被项目设置的保存和重读改变。只盯 rootPath 的话，
  // 改完项目设置词表不会跟着变。（打开项目时两者是同一次提交里一起变的，
  // Vue 会合并成一次触发，不会重复加载。）
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
