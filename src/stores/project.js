import { defineStore } from 'pinia'
import { getFileTree, getDirChildren, setProjectRoot } from '../api'
import { loadProjectConfig, saveProjectConfig } from '../api'

export const useProjectStore = defineStore('project', {
  state: () => ({
    rootPath: null,        // 当前打开的项目根目录
    files: [],             // 文件树数据（浅层，子目录按需加载）
    isLoading: false,
    projectConfig: null    // .thtk-project.json 的内容，null 表示不存在
  }),

  getters: {
    rootName: (state) => {
      if (!state.rootPath) return ''
      const parts = state.rootPath.split(/[\\/]/).filter(Boolean)
      return parts[parts.length - 1] || state.rootPath
    },

    /** 项目配置的游戏版本，回退到空字符串 */
    gameVersion: (state) => state.projectConfig?.gameVersion || '',

    /** 项目配置的编码 */
    encoding: (state) => state.projectConfig?.encoding || 'shift-jis',

    /** 项目配置的 map 路径列表 */
    mapPaths: (state) => state.projectConfig?.mapPaths || [],

    /** 是否已有项目配置文件 */
    hasProjectConfig: (state) => state.projectConfig !== null
  },

  actions: {
    hydrate(snapshot) {
      if (!snapshot?.rootPath) return
      this.rootPath = snapshot.rootPath
    },

    async loadProject(path) {
      this.rootPath = path
      this.isLoading = true
      try {
        await setProjectRoot(path)
        this.files = await getFileTree(path)
        // 尝试加载项目配置
        try {
          this.projectConfig = await loadProjectConfig()
        } catch {
          this.projectConfig = null
        }
      } catch (err) {
        console.error('Failed to load project:', err)
      } finally {
        this.isLoading = false
      }
    },

    /** 保存项目配置到 .thtk-project.json */
    async saveConfig(config) {
      await saveProjectConfig(config)
      this.projectConfig = config
    },

    /** 更新项目配置的部分字段并保存 */
    async updateConfig(partial) {
      const current = this.projectConfig || {
        gameVersion: '',
        encoding: 'shift-jis',
        mapPaths: [],
        toolchain: { thtkDir: '' }
      }
      const merged = { ...current, ...partial }
      await this.saveConfig(merged)
    },

    /** 按需加载某个目录的子节点，返回子节点列表 */
    async loadChildren(dirPath) {
      const children = await getDirChildren(dirPath)
      this._mergeChildren(this.files, dirPath, children)
      return children
    },

    /** 递归查找目标目录并设置其 children */
    _mergeChildren(nodes, dirPath, children) {
      for (const node of nodes) {
        if (node.path === dirPath) {
          node.children = children
          return true
        }
        if (node.children?.length) {
          if (this._mergeChildren(node.children, dirPath, children)) return true
        }
      }
      return false
    },

    async refresh() {
      if (!this.rootPath) return
      // 记住当前已加载的目录路径，刷新后重新加载它们
      const loadedDirs = new Set()
      this._collectLoadedDirs(this.files, loadedDirs)

      this.files = await getFileTree(this.rootPath)

      // 重新加载之前已展开的子目录
      for (const dirPath of loadedDirs) {
        try {
          await this.loadChildren(dirPath)
        } catch {
          // 目录可能已被删除，忽略
        }
      }
    },

    /** 收集所有已加载了 children 的目录路径 */
    _collectLoadedDirs(nodes, result) {
      for (const node of nodes) {
        if (node.is_dir && node.children?.length) {
          result.add(node.path)
          this._collectLoadedDirs(node.children, result)
        }
      }
    },

    toSnapshot() {
      return {
        rootPath: this.rootPath
      }
    }
  }
})
