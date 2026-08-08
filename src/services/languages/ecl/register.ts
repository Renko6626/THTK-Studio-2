import * as monaco from 'monaco-editor'
import { createEclCompletionProvider } from './completion-provider'
import { createEclDefinitionProvider } from './definition-provider'
import { createEclHoverProvider } from './hover-provider'
import { eclLanguageConfiguration, eclLanguageId } from './language-config'
import { createEclReferencesProvider } from './references-provider'
import { createEclSignatureHelpProvider } from './signature-help-provider'
import { createEmptyEclSemanticData } from './dynamic-vocabulary'
import {
  clearScopedEclSemanticData,
  getActiveEclSemanticData,
  getEclSemanticDataForModel,
  setActiveEclSemanticScope,
  updateScopedEclSemanticData
} from './semantic-state'
import { buildEclMonarchLanguage } from './tokenizer'
import { workbenchThemeDefinition, workbenchThemeName } from '../theme'

/**
 * Monaco 的 provider 注册是全局的，而 Vite 的 HMR 会重复执行本模块。
 * 把注册状态挂在 globalThis 上，重载时能先 dispose 掉上一批，避免补全项翻倍。
 */
interface EclLanguageRegistryState {
  registered: boolean
  completionProviderDisposable: monaco.IDisposable | null
  definitionProviderDisposable: monaco.IDisposable | null
  referencesProviderDisposable: monaco.IDisposable | null
  hoverProviderDisposable: monaco.IDisposable | null
  signatureHelpProviderDisposable: monaco.IDisposable | null
}

declare global {
  // eslint-disable-next-line no-var
  var __THTK_ECL_LANGUAGE_REGISTRY__: EclLanguageRegistryState | undefined
}


function createRegistryState(): EclLanguageRegistryState {
  return {
    registered: false,
    completionProviderDisposable: null,
    definitionProviderDisposable: null,
    referencesProviderDisposable: null,
    hoverProviderDisposable: null,
    signatureHelpProviderDisposable: null
  }
}

function getRegistryState(): EclLanguageRegistryState {
  if (!globalThis.__THTK_ECL_LANGUAGE_REGISTRY__) {
    globalThis.__THTK_ECL_LANGUAGE_REGISTRY__ = createRegistryState()
  }
  return globalThis.__THTK_ECL_LANGUAGE_REGISTRY__
}

function disposeProvider(providerDisposable: monaco.IDisposable | null): void {
  if (providerDisposable && typeof providerDisposable.dispose === 'function') {
    providerDisposable.dispose()
  }
}

function disposeRegisteredProviders(state: EclLanguageRegistryState): void {
  disposeProvider(state.completionProviderDisposable)
  disposeProvider(state.definitionProviderDisposable)
  disposeProvider(state.referencesProviderDisposable)
  disposeProvider(state.hoverProviderDisposable)
  disposeProvider(state.signatureHelpProviderDisposable)
  state.completionProviderDisposable = null
  state.definitionProviderDisposable = null
  state.referencesProviderDisposable = null
  state.hoverProviderDisposable = null
  state.signatureHelpProviderDisposable = null
}

export function ensureEclLanguageRegistered() {
  const state = getRegistryState()
  if (state.registered) return eclLanguageId

  monaco.languages.register({ id: eclLanguageId })
  monaco.languages.setLanguageConfiguration(eclLanguageId, eclLanguageConfiguration)
  monaco.languages.setMonarchTokensProvider(
    eclLanguageId,
    buildEclMonarchLanguage(getActiveEclSemanticData())
  )
  monaco.editor.defineTheme(workbenchThemeName, workbenchThemeDefinition)
  disposeRegisteredProviders(state)
  state.completionProviderDisposable = monaco.languages.registerCompletionItemProvider(
    eclLanguageId,
    createEclCompletionProvider((model) => getEclSemanticDataForModel(model))
  )
  state.definitionProviderDisposable = monaco.languages.registerDefinitionProvider(
    eclLanguageId,
    createEclDefinitionProvider()
  )
  state.referencesProviderDisposable = monaco.languages.registerReferenceProvider(
    eclLanguageId,
    createEclReferencesProvider()
  )
  state.hoverProviderDisposable = monaco.languages.registerHoverProvider(
    eclLanguageId,
    createEclHoverProvider((model) => getEclSemanticDataForModel(model))
  )
  state.signatureHelpProviderDisposable = monaco.languages.registerSignatureHelpProvider(
    eclLanguageId,
    createEclSignatureHelpProvider((model) => getEclSemanticDataForModel(model))
  )

  state.registered = true
  return eclLanguageId
}

export { workbenchThemeName }

export function updateEclSemanticVocabulary(
  scopeKey: string | null | undefined,
  semanticData: unknown
): void {
  updateScopedEclSemanticData(scopeKey, semanticData)
  setActiveEclSemanticScope(scopeKey)
  if (!getRegistryState().registered) return

  monaco.languages.setMonarchTokensProvider(
    eclLanguageId,
    buildEclMonarchLanguage(getActiveEclSemanticData())
  )
}

export function clearEclSemanticVocabulary(scopeKey: string = '__global__'): void {
  clearScopedEclSemanticData(scopeKey)
  setActiveEclSemanticScope(scopeKey)
  if (!getRegistryState().registered) return

  monaco.languages.setMonarchTokensProvider(
    eclLanguageId,
    buildEclMonarchLanguage(getActiveEclSemanticData() || createEmptyEclSemanticData())
  )
}
