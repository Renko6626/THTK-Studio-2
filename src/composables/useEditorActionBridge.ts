/** MonacoEditor 通过 window 事件接收的动作，避免组件间直接耦合 */
export const editorActionIds = {
  undo: 'undo',
  redo: 'redo',
  cut: 'editor.action.clipboardCutAction',
  copy: 'editor.action.clipboardCopyAction',
  paste: 'editor.action.clipboardPasteAction',
  find: 'actions.find',
  replace: 'editor.action.startFindReplaceAction',
  findNext: 'editor.action.nextMatchFindAction',
  findPrevious: 'editor.action.previousMatchFindAction',
  selectAll: 'editor.action.selectAll',
  lineComment: 'editor.action.commentLine',
  indent: 'editor.action.indentLines',
  outdent: 'editor.action.outdentLines',
  moveLineUp: 'editor.action.moveLinesUpAction',
  moveLineDown: 'editor.action.moveLinesDownAction',
  copyLineDown: 'editor.action.copyLinesDownAction',
  deleteLine: 'editor.action.deleteLines',
  selectLine: 'expandLineSelection',
  selectNextMatch: 'editor.action.addSelectionToNextFindMatch',
  selectAllMatches: 'editor.action.selectHighlights',
  addCursorAbove: 'editor.action.insertCursorAbove',
  addCursorBelow: 'editor.action.insertCursorBelow',
  expandSelection: 'editor.action.smartSelect.expand',
  shrinkSelection: 'editor.action.smartSelect.shrink',
  gotoLine: 'editor.action.gotoLine',
  gotoBracket: 'editor.action.jumpToBracket',
  foldAll: 'editor.foldAll',
  unfoldAll: 'editor.unfoldAll',
  commandPalette: 'editor.action.quickCommand'
} as const

export type EditorAction = keyof typeof editorActionIds

export interface EditorRevealLocation {
  path: string
  line: number
  column?: number
}

export function dispatchEditorAction(action: EditorAction): void {
  window.dispatchEvent(new CustomEvent('thtk:editor-action', { detail: { action } }))
}

export function dispatchEditorRevealLocation({ path, line, column }: EditorRevealLocation): void {
  window.dispatchEvent(new CustomEvent('thtk:editor-reveal-location', {
    detail: { path, line, column }
  }))
}
