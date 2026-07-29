// src/composables/useResizable.ts
// 通用分隔条拖动逻辑:pointer capture + 元素级 move/up 监听,
// 取值/写值与方向由调用方提供,横竖三处分隔条共用。
export interface ResizableOptions {
  getValue: () => number
  setValue: (value: number) => void
  /** 'y' = 竖向拖动（改高度），'x' = 横向（改宽度） */
  axis?: 'x' | 'y'
  /** 分隔条在被调整元素的另一侧时取反 */
  invert?: boolean
  onDragStart?: (() => void) | null
}

export function useResizable({
  getValue,
  setValue,
  axis = 'y',
  invert = false,
  onDragStart = null
}: ResizableOptions) {
  function onPointerdown(event: PointerEvent) {
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

    // 模板里绑在元素上，currentTarget 必然是该元素
    const target = event.currentTarget as HTMLElement
    target.setPointerCapture(event.pointerId)

    function onMove(moveEvent: PointerEvent) {
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
