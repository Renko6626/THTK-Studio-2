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
    payload: createDefaultBuildPayload()
  }),

  actions: {
    openDialog(payload: Partial<TheclBuildPayload> & { tool?: ToolchainId } = {}) {
      const tool = payload.tool || 'thecl'
      // mapPaths 对非 thecl 的载荷其实是多余字段，但现有行为如此，
      // 迁移不改行为——等这些工具真正接上构建对话框时再收拾。
      // 这里的断言无法去掉：展开合并的结果对 TS 而言是宽对象，而 BuildDialogPayload
      // 是判别联合，编译器无法从运行时的 tool 值推出该选哪一支。
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
