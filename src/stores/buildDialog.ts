import { defineStore } from 'pinia'
import { createDefaultBuildPayload } from '../services/toolchains/registry'
import type { BuildDialogPayload, TheclBuildPayload, ToolchainId } from '../types'

interface BuildDialogState {
  visible: boolean
  payload: BuildDialogPayload
}

export const useBuildDialogStore = defineStore('buildDialog', {
  state: (): BuildDialogState => ({
    visible: false,
    payload: createDefaultBuildPayload() as BuildDialogPayload
  }),

  actions: {
    openDialog(payload: Partial<TheclBuildPayload> & { tool?: ToolchainId } = {}) {
      const tool = payload.tool || 'thecl'
      // mapPaths 对非 thecl 的载荷其实是多余字段，但现有行为如此，
      // 迁移不改行为——等这些工具真正接上构建对话框时再收拾。
      // as 断言的原因：createDefaultBuildPayload 还在 registry.js 里（Task 5 迁），
      // 拿不到返回类型。registry 迁完后可以去掉。
      this.payload = {
        ...createDefaultBuildPayload(tool),
        ...payload,
        mapPaths: [...(payload.mapPaths || [])]
      } as BuildDialogPayload
      this.visible = true
    },

    openTheclDialog(payload: Partial<TheclBuildPayload> = {}) {
      this.openDialog(payload)
    },

    close() {
      this.visible = false
    }
  }
})
