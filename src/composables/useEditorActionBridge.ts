/** MonacoEditor 通过 window 事件接收的动作，避免组件间直接耦合 */
export type EditorAction =
  | 'undo'
  | 'redo'
  | 'find'
  | 'replace'
  | 'findNext'
  | 'findPrevious'
  | 'selectAll'

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
