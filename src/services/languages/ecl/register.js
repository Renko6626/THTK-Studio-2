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
import { eclThemeDefinition, eclThemeName } from './theme'

const registryStateKey = '__THTK_ECL_LANGUAGE_REGISTRY__'

function createRegistryState() {
  return {
    registered: false,
    completionProviderDisposable: null,
    definitionProviderDisposable: null,
    referencesProviderDisposable: null,
    hoverProviderDisposable: null,
    signatureHelpProviderDisposable: null
  }
}

function getRegistryState() {
  const globalScope = globalThis
  if (!globalScope[registryStateKey]) {
    globalScope[registryStateKey] = createRegistryState()
  }
  return globalScope[registryStateKey]
}

function disposeProvider(providerDisposable) {
  if (providerDisposable && typeof providerDisposable.dispose === 'function') {
    providerDisposable.dispose()
  }
}

function disposeRegisteredProviders(state) {
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
  monaco.editor.defineTheme(eclThemeName, eclThemeDefinition)
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

export { eclThemeName }

export function updateEclSemanticVocabulary(scopeKey, semanticData) {
  updateScopedEclSemanticData(scopeKey, semanticData)
  setActiveEclSemanticScope(scopeKey)
  if (!getRegistryState().registered) return

  monaco.languages.setMonarchTokensProvider(
    eclLanguageId,
    buildEclMonarchLanguage(getActiveEclSemanticData())
  )
}

export function clearEclSemanticVocabulary(scopeKey = '__global__') {
  clearScopedEclSemanticData(scopeKey)
  setActiveEclSemanticScope(scopeKey)
  if (!getRegistryState().registered) return

  monaco.languages.setMonarchTokensProvider(
    eclLanguageId,
    buildEclMonarchLanguage(getActiveEclSemanticData() || createEmptyEclSemanticData())
  )
}

export function inferMonacoLanguageId(tab) {
  const path = String(tab?.path || '').toLowerCase()
  const language = String(tab?.language || '').toLowerCase()

  if (path.endsWith('.decl') || path.endsWith('.tecl')) return eclLanguageId
  if (language === 'json' || path.endsWith('.json')) return 'json'
  if (language === 'js' || path.endsWith('.js')) return 'javascript'
  if (language === 'ts' || path.endsWith('.ts')) return 'typescript'
  if (language === 'html' || path.endsWith('.vue') || path.endsWith('.html')) return 'html'
  if (language === 'c' || path.endsWith('.c')) return 'c'
  if (language === 'cpp' || path.endsWith('.cpp') || path.endsWith('.h')) return 'cpp'

  return 'plaintext'
}
