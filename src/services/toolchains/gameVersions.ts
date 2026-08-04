import type { GameVersionView } from '../../types'

/**
 * 过滤出支持指定工具的版本。
 *
 * 未知工具返回空数组，**不做「未知即全放行」的兜底**——那会让拼错的工具 id
 * 悄悄拿到全量列表，正是此前 THECL_VERSION_OPTIONS 被复用给五个工具的翻版。
 */
export function versionsForTool(
  table: GameVersionView[],
  toolId: string
): GameVersionView[] {
  return table.filter((entry) => entry.tools.includes(toolId))
}

/** 下拉框标签：版本号在前便于键盘检索，标题在后便于辨认。 */
export function formatVersionLabel(entry: GameVersionView): string {
  return `${entry.code} · ${entry.title}`
}

export interface ToolAvailability {
  enabled: boolean
  /** 禁用原因，供 tooltip 展示；enabled 为 true 时是空串 */
  reason: string
}

/**
 * 某个工具在当前游戏版本下是否可用。
 *
 * 五个工具支持的版本集合并不相同：选了萃夢想（75）就只有 thdat 能用，
 * 选了 Uwabami Breakers（103）则除 thmsg 外都能用。与其让用户点下去
 * 拿一句英文报错，不如在菜单上直接说清楚。
 *
 * **信息不足时一律放行**——版本表没加载好、没选版本、版本不在表内，
 * 三种情况都返回可用。灰掉的按钮不会告诉用户为什么，而后端的报错会。
 */
export function toolAvailability(
  toolId: string,
  version: string | null | undefined,
  table: GameVersionView[]
): ToolAvailability {
  const raw = String(version ?? '').trim()
  if (!table.length || !raw) return { enabled: true, reason: '' }

  // 容忍 th 前缀：后端已统一归一，但配置可能来自尚未重新保存的旧文件
  const id = Number(raw.toLowerCase().replace(/^th/, ''))
  if (!Number.isFinite(id)) return { enabled: true, reason: '' }

  const entry = table.find((item) => item.id === id)
  if (!entry) return { enabled: true, reason: '' }
  if (entry.tools.includes(toolId)) return { enabled: true, reason: '' }

  return {
    enabled: false,
    reason: `${entry.title}（${entry.code}）在 thtk 里只有 ${entry.tools.join(' / ')} 支持`
  }
}
