// src/composables/useResizable.js
// 通用分隔条拖动逻辑:pointer capture + 元素级 move/up 监听,
// 取值/写值与方向由调用方提供,横竖三处分隔条共用。
export function useResizable({
  getValue,
  setValue,
  axis = 'y',
  invert = false,
  onDragStart = null
}) {
  function onPointerdown(event) {
    if (event.button !== 0) return
    event.preventDefault()
    onDragStart?.()

    // onDragStart 可能改变状态(如退出最大化),之后再取起始值
    const startPos = axis === 'y' ? event.clientY : event.clientX
    const startValue = getValue()
    const previousCursor = document.body.style.cursor
    const previousUserSelect = document.body.style.userSelect
    document.body.style.cursor = axis === 'y' ? 'row-resize' : 'col-resize'
    document.body.style.userSelect = 'none'

    const target = event.currentTarget
    target.setPointerCapture(event.pointerId)

    function onMove(moveEvent) {
      const pos = axis === 'y' ? moveEvent.clientY : moveEvent.clientX
      let delta = pos - startPos
      if (invert) delta = -delta
      setValue(startValue + delta)
    }

    function onUp() {
      target.removeEventListener('pointermove', onMove)
      target.removeEventListener('pointerup', onUp)
      target.removeEventListener('pointercancel', onUp)
      document.body.style.cursor = previousCursor
      document.body.style.userSelect = previousUserSelect
    }

    target.addEventListener('pointermove', onMove)
    target.addEventListener('pointerup', onUp)
    target.addEventListener('pointercancel', onUp)
  }

  return { onPointerdown }
}
