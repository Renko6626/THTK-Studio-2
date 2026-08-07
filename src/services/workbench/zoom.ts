/**
 * 工作台整体缩放（对应 VS Code 的 `View: Zoom In / Out / Reset`）。
 *
 * ## 为什么是整窗缩放而不是"改字号"
 *
 * 本应用的字号分散在三处、机制互不相同：
 *
 * - Monaco 的 `fontSize`（JS 选项，要显式 `updateOptions`）
 * - xterm 的 `fontSize`（JS 选项，改完还要重新 `fit`，否则行列数算错）
 * - UI 的 119 处 UnoCSS 绝对 px 原子类（`text-[11px]` 之类，没有统一入口）
 *
 * 想靠调字号做到"整体变大"，等于要同时维护这三条路径，而且 UI 那 119 处得先
 * 全改成相对单位。VS Code 自己也不这么做——它的 Ctrl+= 走的是 Electron 的
 * `webFrame.setZoomLevel()`，即**浏览器级缩放**。
 *
 * Tauri v2 有对应的 `getCurrentWebview().setZoom()`，一行搞定全部三处，
 * 而且 Monaco / xterm 会把它当作设备像素比变化正常处理——它们本来就要应付
 * 用户按浏览器的 Ctrl+滚轮。
 *
 * ## 刻度
 *
 * 用等比而非线性：低倍率时步进要小（1.0 → 1.1 是明显变化），高倍率时步进要大。
 * 直接照搬 VS Code 的 1.2 倍公比。
 */

import { getCurrentWebview } from '@tauri-apps/api/webview'

const STORAGE_KEY = 'thtk-studio:workbench-zoom'

/** VS Code 的公比 */
const RATIO = 1.2
/** 上下限。再小看不清，再大一屏放不下几行——VS Code 也在这个量级 */
export const MIN_LEVEL = -5
export const MAX_LEVEL = 8

/** 缩放等级 → 倍率。0 = 100% */
export function levelToScale(level: number): number {
  return RATIO ** level
}

export function clampLevel(level: number): number {
  if (!Number.isFinite(level)) return 0
  return Math.min(MAX_LEVEL, Math.max(MIN_LEVEL, Math.round(level)))
}

/** 给 UI 显示的百分比，如 `120%` */
export function formatScale(level: number): string {
  return `${Math.round(levelToScale(level) * 100)}%`
}

/**
 * 读取上次的缩放等级。
 *
 * localStorage 里的内容不可信（可能是旧版本写的、也可能被手改过），
 * 解析不出来就当 0，绝不让一个坏值把界面缩到看不见。
 */
export function loadZoomLevel(): number {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (raw === null) return 0
    return clampLevel(Number(raw))
  } catch {
    return 0
  }
}

export function saveZoomLevel(level: number): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, String(clampLevel(level)))
  } catch {
    // 隐私模式等场景下写不进去。缩放本身已经生效，丢的只是"下次还记得"
  }
}

/**
 * 应用缩放。失败不抛出——在浏览器里跑（无 Tauri）或平台不支持时，
 * 应用其余部分不该因此崩掉。
 */
export async function applyZoom(level: number): Promise<void> {
  try {
    await getCurrentWebview().setZoom(levelToScale(clampLevel(level)))
  } catch {
    // 无 Tauri 环境或平台不支持：静默降级，不影响其他功能
  }
}
