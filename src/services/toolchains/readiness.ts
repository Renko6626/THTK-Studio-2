import type { ToolchainStatus } from '../../types'

export type ToolchainReadinessState = 'ready' | 'partial' | 'missing'

export interface ToolchainReadiness {
  state: ToolchainReadinessState
  /** 面向用户的一句话说明；ready 时为空串 */
  message: string
  /** 不可用的工具 id，按 statuses 顺序 */
  missing: string[]
}

/**
 * 判断 thtk 工具链是否可用。
 *
 * 安装包**不含** thtk（`tauri.conf.json` 没配 `bundle.resources`，thtk 是第三方
 * GPL 工具），而 `thtk_dir` 默认是空串、为空时后端退回裸 exe 名走 PATH。
 * 所以全新安装后所有工具链动作都不可用，而界面上此前没有任何提示——
 * 用户要么点了才拿到一句英文报错，要么根本不知道从哪配。
 *
 * 状态还没取到（空数组）时返回 `ready`：宁可不提示，也不要在加载过程中
 * 闪一条"未配置"的假警报。
 */
export function summarizeToolchainReadiness(
  statuses: ToolchainStatus[]
): ToolchainReadiness {
  if (!statuses.length) {
    return { state: 'ready', message: '', missing: [] }
  }

  const missing = statuses.filter((s) => !s.available).map((s) => s.tool)

  if (!missing.length) {
    return { state: 'ready', message: '', missing: [] }
  }

  if (missing.length === statuses.length) {
    return {
      state: 'missing',
      message:
        '尚未配置 thtk 工具链，编译、反编译与解包功能都不可用。thtk 是第三方工具，需要另行下载后在设置里指定其所在目录。',
      missing
    }
  }

  return {
    state: 'partial',
    message: `thtk 目录里缺少 ${missing.join(' / ')}，用到这些工具的操作会失败。`,
    missing
  }
}
