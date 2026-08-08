import type * as monaco from 'monaco-editor'

/**
 * 工作台级的 Monaco 主题。
 *
 * **不是 ECL 专属**——它由 `monaco.editor.defineTheme` 注册一次，并作为编辑器的
 * `theme` 选项全局生效，token 名在 .decl / .dstd / .dmsg 之间共享。此前它放在
 * `languages/ecl/theme.ts` 且叫 `eclTheme*`，名字会让人以为改它只影响 ECL。
 *
 * 新增语言时应尽量**复用下面已有的 token 名**，这样颜色天然一致，也不必每加一种
 * 文件格式就往主题里堆规则。
 */
export const workbenchThemeName = 'thtk-vscode-dark'

export const workbenchThemeRules: monaco.editor.ITokenThemeRule[] = [
  { token: 'keyword.directive', foreground: '4FC1FF' },
  { token: 'keyword.difficulty', foreground: 'C586C0', fontStyle: 'bold' },
  { token: 'preprocessor', foreground: 'C586C0' },
  { token: 'entity.name.function', foreground: 'DCDCAA' },
  { token: 'entity.name.function.builtin', foreground: '4EC9B0' },
  /**
   * 查不到名字的操作号（`.dstd` / `.dmsg` 里的 `ins_250`）。
   *
   * 刻意**压暗**而不是标红：它不是错误，只是我们的 mapfile 还没有这个名字。
   * th19 的 MSG 就有 16 个这样的操作号，全标红会让整个文件在尖叫。`ins_` 这个
   * 前缀本身已经足够醒目，颜色只需要让它从已命名的指令里退后一步。
   */
  { token: 'entity.name.function.unresolved', foreground: '858585' },
  { token: 'entity.name.label', foreground: 'D7BA7D' },
  { token: 'variable.special', foreground: '9CDCFE' },
  { token: 'variable.predefined', foreground: '4FC1FF' },
  { token: 'meta.stack', foreground: 'CE9178' }
]

export const workbenchThemeDefinition: monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: workbenchThemeRules,
  colors: {}
}
