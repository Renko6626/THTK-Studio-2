import { defineStore } from 'pinia'
import { clearRecentProjects, listRecentProjects, removeRecentProject } from '../api'

/**
 * 最近打开的项目。列表由 Rust 侧维护（去重、置顶、10 项上限），
 * 这里只负责缓存展示数据和转发用户操作。
 *
 * `available` 是后端每次读取时现算的，失效路径仍会列出并标记为不可用，
 * 由用户决定移除——磁盘临时离线不该让记录消失。
 */
export const useRecentProjectsStore = defineStore('recentProjects', {
  state: () => ({
    items: [],
    isLoading: false,
    error: null
  }),

  getters: {
    isEmpty: (state) => state.items.length === 0
  },

  actions: {
    async refresh() {
      this.isLoading = true
      try {
        this.items = await listRecentProjects()
        this.error = null
      } catch (error) {
        // 读不出来时保留上一次的列表，只记录原因；清空会让用户以为记录丢了
        this.error = String(error)
      } finally {
        this.isLoading = false
      }
    },

    async remove(path) {
      this.items = await removeRecentProject(path)
      this.error = null
    },

    async clear() {
      await clearRecentProjects()
      this.items = []
      this.error = null
    }
  }
})
