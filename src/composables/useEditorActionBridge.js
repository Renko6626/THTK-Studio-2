export function dispatchEditorAction(action) {
  window.dispatchEvent(new CustomEvent('thtk:editor-action', { detail: { action } }))
}

export function dispatchEditorRevealLocation({ path, line, column }) {
  window.dispatchEvent(new CustomEvent('thtk:editor-reveal-location', {
    detail: { path, line, column }
  }))
}
