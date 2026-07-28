import { defineComponent } from 'vue'
import { mount } from '@vue/test-utils'

/**
 * 在真实组件上下文里执行 composable，让 onMounted / onBeforeUnmount / watch 正常触发。
 * 直接调用 composable 的话这些生命周期钩子会被 Vue 忽略，测不到恢复流程。
 */
export function withSetup(composable) {
  let result
  const wrapper = mount(
    defineComponent({
      setup() {
        result = composable()
        return () => null
      }
    })
  )
  return { result, wrapper }
}

/** 够用的假 store：只提供 composable 真正会碰的接口 */
export function createFakeStore(state = {}) {
  return {
    ...state,
    $subscribe: () => () => {},
    toSnapshot: () => ({ ...state })
  }
}
