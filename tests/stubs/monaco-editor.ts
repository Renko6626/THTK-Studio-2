/**
 * monaco-editor 的测试替身。
 *
 * 真包在 happy-dom 里加载不了（依赖 worker / canvas / DOM API），而且 vitest 的
 * 覆盖范围本来就不含组件渲染。这里只提供语言服务注册用到的那几个 API，
 * 让 `services/languages/**` 里的**纯逻辑**可测——比如 STD 的版本注入与偏移换算。
 */
export class Range {
  constructor(
    public startLineNumber: number,
    public startColumn: number,
    public endLineNumber: number,
    public endColumn: number
  ) {}
}

const noopDisposable = () => ({ dispose: () => {} })

export const languages = {
  register: () => {},
  setLanguageConfiguration: noopDisposable,
  setMonarchTokensProvider: noopDisposable,
  registerDefinitionProvider: noopDisposable,
  registerCodeLensProvider: noopDisposable,
  registerCompletionItemProvider: noopDisposable,
  registerReferenceProvider: noopDisposable,
  registerHoverProvider: noopDisposable,
  registerSignatureHelpProvider: noopDisposable,
  registerDocumentSymbolProvider: noopDisposable
}

export const editor = {
  defineTheme: () => {},
  setModelLanguage: () => {}
}
