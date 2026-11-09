import type { ToolchainStatus } from '../../types'

export type ToolchainReadinessState = 'ready' | 'partial' | 'missing'

export interface ToolchainReadiness {
  state: ToolchainReadinessState
  /** 面向用户的一句话说明；ready 时为空串 */
  message: string
  /** 不可用的工具 id，按 statuses 顺序 */
  missing: string[]
}

/** 状态尚未加载时不闪现缺失提示；具体工具来源由设置面板展示。 */
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
        '工具链均不可用，请检查工具链设置。truth 随安装包提供，thtk 需另行配置。',
      missing
    }
  }

  return {
    state: 'partial',
    message: `工具不可用：${missing.join(' / ')}，用到这些工具的操作会失败。`,
    missing
  }
}
