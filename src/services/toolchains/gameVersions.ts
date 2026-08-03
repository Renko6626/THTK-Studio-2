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
