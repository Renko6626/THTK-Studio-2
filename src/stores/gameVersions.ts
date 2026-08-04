import { defineStore } from 'pinia'
import { listGameVersions } from '../api'
import { versionsForTool, formatVersionLabel } from '../services/toolchains/gameVersions'
import type { GameVersionView } from '../types'

/**
 * 版本表来自后端，全应用只拉一次。
 *
 * 刻意**不做**「加载失败就退回硬编码列表」——那正是此前 THECL_VERSION_OPTIONS
 * 造成的问题：一张按 thecl 写的表被复用给五个集合不同的工具。宁可选项为空
 * 并暴露 error，也不给出错误的可选项。
 */
export const useGameVersionsStore = defineStore('gameVersions', {
  state: () => ({
    table: [] as GameVersionView[],
    loaded: false,
    error: '' as string,
    /** 在途请求，用于并发去重；失败后清空以便重试 */
    inFlight: null as Promise<void> | null
  }),
  getters: {
    optionsForTool: (state) => (toolId: string) =>
      versionsForTool(state.table, toolId).map((entry) => ({
        label: formatVersionLabel(entry),
        value: String(entry.id)
      })),
    /** 项目级版本对所有工具生效，取整张表 */
    allOptions: (state) =>
      state.table.map((entry) => ({
        label: formatVersionLabel(entry),
        value: String(entry.id)
      }))
  },
  actions: {
    async ensureLoaded() {
      if (this.loaded) return
      if (this.inFlight) return this.inFlight

      this.inFlight = (async () => {
        try {
          this.table = await listGameVersions()
          this.error = ''
          // 只在成功后置位：失败时保持 false，下次调用可重试。
          this.loaded = true
        } catch (e) {
          this.error = e instanceof Error ? e.message : String(e)
          this.table = []
        } finally {
          this.inFlight = null
        }
      })()

      return this.inFlight
    }
  }
})
