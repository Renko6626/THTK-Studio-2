export const eclThemeName = 'thtk-vscode-dark'

export const eclThemeRules = [
  { token: 'keyword.directive', foreground: '4FC1FF' },
  { token: 'keyword.difficulty', foreground: 'C586C0', fontStyle: 'bold' },
  { token: 'preprocessor', foreground: 'C586C0' },
  { token: 'entity.name.function', foreground: 'DCDCAA' },
  { token: 'entity.name.function.builtin', foreground: '4EC9B0' },
  { token: 'entity.name.label', foreground: 'D7BA7D' },
  { token: 'variable.special', foreground: '9CDCFE' },
  { token: 'variable.predefined', foreground: '4FC1FF' },
  { token: 'meta.stack', foreground: 'CE9178' }
]

export const eclThemeDefinition = {
  base: 'vs-dark',
  inherit: true,
  rules: eclThemeRules,
  colors: {}
}
