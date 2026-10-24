import { invoke } from '@tauri-apps/api/core'
import type { DependencyStatus } from '../../types'

/**
 * 只读依赖状态：thtk 手动配置 + 内置 ECL map 覆盖情况。
 * 后端会在返回前幂等释放内置 map 基线，所以 `maps.bundledDir` 里的文件可直接使用。
 * 该命令不写入任何配置。
 */
export function getDependencyStatus(): Promise<DependencyStatus> {
  return invoke('get_dependency_status')
}
