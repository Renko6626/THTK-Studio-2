import { defineStore } from 'pinia'
import { getFileTree, getDirChildren, openProject } from '../api'
import { loadProjectConfig, saveProjectConfig } from '../api'

export const useProjectStore = defineStore('project', {
  state: () => ({
    rootPath: null,        // 当前打开的项目根目录
    files: [],             // 文件树数据（浅层，子目录按需加载）
    isLoading: false,
    projectConfig: null,   // .thtk-project.json 的内容，null 表示不可用
    // 'absent'（没有配置文件）/ 'loaded'（可用）/ 'invalid'（文件坏了或字段非法）。
    // 必须区分后两者：把损坏当成"还没配置"会让保存动作静默覆盖用户手写的内容。
    projectConfigStatus: 'absent',
    projectConfigError: null,
    projectConfigPath: '',
    _refreshPromise: null,
    _refreshPending: false
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

    /** 项目级 thtk 目录覆盖，空字符串表示沿用全局设置 */
    projectThtkDir: (state) => state.projectConfig?.toolchain?.thtkDir || '',

    /** 是否已有可用的项目配置文件 */
    hasProjectConfig: (state) => state.projectConfigStatus === 'loaded',

    /** 配置文件存在但无法使用（JSON 损坏或字段非法） */
    hasInvalidProjectConfig: (state) => state.projectConfigStatus === 'invalid'
  },

  actions: {
    // 没有 hydrate：项目根不能凭快照乐观地设上去。恢复失败时那个值会留成僵尸状态，
    // 让 UI 显示着一个其实没打开的项目。恢复统一走 loadProject，成功才提交。

    /**
     * 打开项目。成功之前不改动任何状态——目录不存在或扫描失败时，当前工作区、
     * 文件树与项目配置都保持原样，错误向上抛给调用方决定怎么提示。
     *
     * 后端是事务式的：验证与首层扫描都通过后才提交项目根、切换文件监听、
     * 注册 MCP 客户端并记录最近项目，所以这里失败不会留下半个状态。
     */
    async loadProject(path) {
      this.isLoading = true
      try {
        const result = await openProject(path)
        this.rootPath = result.rootPath
        this.files = result.files || []
        this._applyProjectConfigLoad(result.projectConfig)
        return result
      } finally {
        this.isLoading = false
      }
    },

    /** 回到"没有工作区"的状态，供恢复失败或用户关闭项目时使用 */
    closeProject() {
      this.rootPath = null
      this.files = []
      this._applyProjectConfigLoad({ status: 'absent', value: null, error: null, path: '' })
    },

    /** 重新读取 .thtk-project.json 并记录其三态状态 */
    async reloadProjectConfig() {
      try {
        this._applyProjectConfigLoad(await loadProjectConfig())
      } catch (err) {
        // 命令本身失败（例如项目根未设置）——状态未知，按不可用处理并留下原因，
        // 不要伪装成"没有配置文件"，否则保存动作会以为可以放心创建。
        this._applyProjectConfigLoad({
          status: 'invalid',
          value: null,
          error: String(err),
          path: ''
        })
      }
    },

    _applyProjectConfigLoad(result) {
      this.projectConfigStatus = result?.status || 'absent'
      this.projectConfig = result?.value || null
      this.projectConfigError = result?.error || null
      this.projectConfigPath = result?.path || ''
    },

    /**
     * 保存项目配置到 .thtk-project.json。
     * expectedRoot 是调用方开始编辑时的项目根，后端据此拒绝写错项目。
     */
    async saveConfig(config, expectedRoot) {
      await saveProjectConfig(config, expectedRoot ?? this.rootPath)
      this.projectConfig = config
      this.projectConfigStatus = 'loaded'
      this.projectConfigError = null
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
      if (this._refreshPromise) {
        // 已经在刷,标记"完成后再刷一次"以反映期间的变更
        this._refreshPending = true
        return this._refreshPromise
      }
      this._refreshPromise = this._doRefresh().finally(() => {
        this._refreshPromise = null
        if (this._refreshPending) {
          this._refreshPending = false
          // 不 await,让当前调用尽快返回;后续操作会看到二次 refresh 的效果
          this.refresh()
        }
      })
      return this._refreshPromise
    },

    async _doRefresh() {
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
        if (node.is_dir && node.children !== undefined) {
          result.add(node.path)
          // 仅当有子时才递归;空数组不需要进
          if (node.children.length) {
            this._collectLoadedDirs(node.children, result)
          }
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
