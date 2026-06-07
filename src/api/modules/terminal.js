import { invoke } from '@tauri-apps/api/core'

/**
 * 运行内嵌终端命令
 * @param {string} shell - pwsh | cmd
 * @param {string} command
 * @param {string|null} cwd
 */
export function runShellCommand(shell, command, cwd = null) {
  return invoke('run_shell_command', { shell, command, cwd })
}

/**
 * 校验并解析目录
 * @param {string|null} baseDir
 * @param {string} target
 */
export function resolveDirectory(baseDir, target) {
  return invoke('resolve_directory', { baseDir, target })
}
