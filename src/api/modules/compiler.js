import { invoke } from '@tauri-apps/api/core'

/**
 * 运行统一的 thecl 操作。
 * request 结构与 Rust 侧 TheclRequest 对齐，使用 camelCase 字段。
 */
export function runTheclOperation(request) {
  return invoke('run_thecl_operation', { request })
}

/**
 * 编译 ECL 文件 (.decl -> .ecl)
 * @param {string} sourcePath - .decl 文件路径
 * @param {Array<string>} mapPaths - .eclmap 文件路径列表 (可选)
 */
export function compileEcl(sourcePath, mapPaths = []) {
  return invoke('compile_ecl_file', { sourcePath, mapPaths })
}

/**
 * 反编译 ECL 文件 (.ecl -> .decl)
 * @param {string} binaryPath - .ecl 文件路径
 * @param {Array<string>} mapPaths - .eclmap 文件路径列表 (可选)
 */
export function decompileEcl(binaryPath, mapPaths = []) {
  return invoke('decompile_ecl_file', { binaryPath, mapPaths })
}

/**
 * 运行原始 thtk 命令 (调试用)
 * @param {string} toolPath 
 * @param {Array<string>} args 
 * @param {string|null} workDir 
 */
export function runThtkRaw(toolPath, args, workDir = null) {
  return invoke('run_thtk_raw', { toolPath, args, workDir })
}
