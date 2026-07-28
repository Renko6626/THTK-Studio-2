import { invoke } from '@tauri-apps/api/core'
import type {
  EclResult,
  MsgResult,
  StdResult,
  TheclRequest,
  ThdatResult
} from '../../types'

/** thecl 的 object-arg 形式；mapPaths 留空时后端按项目 / 全局配置推导 */
export interface EclFileParams {
  sourcePath: string
  mapPaths?: string[]
}

/** thmsg / thstd 共用的 object-arg 形式 */
export interface ScriptFileParams {
  inputPath: string
  outputPath?: string | null
  withComments?: boolean
}

export interface ExtractDatParams {
  archivePath: string
  targetDir: string
}

export interface PackDatParams {
  sourceDir: string
  archivePath: string
}


/**
 * 运行统一的 thecl 操作。
 * request 结构与 Rust 侧 TheclRequest 对齐，使用 camelCase 字段。
 */
export function runTheclOperation(request: TheclRequest): Promise<EclResult> {
  return invoke('run_thecl_operation', { request })
}

/**
 * 编译 ECL 文件 (.decl -> .ecl)
 * @param {string} sourcePath - .decl 文件路径
 * @param {Array<string>} mapPaths - .eclmap 文件路径列表 (可选)
 */
export function compileEcl(sourcePath: string, mapPaths: string[] = []): Promise<EclResult> {
  return invoke('compile_ecl_file', { sourcePath, mapPaths })
}

/**
 * 反编译 ECL 文件 (.ecl -> .decl)
 * @param {string} binaryPath - .ecl 文件路径
 * @param {Array<string>} mapPaths - .eclmap 文件路径列表 (可选)
 */
export function decompileEcl(binaryPath: string, mapPaths: string[] = []): Promise<EclResult> {
  return invoke('decompile_ecl_file', { binaryPath, mapPaths })
}

/**
 * 编译 ECL 文件 (.decl -> .ecl) —— 与 thmsg/thstd 同款 object-arg 形式,
 * 走 effective_thecl_version / effective map_paths 的快速路径。
 * 后端命令 `compile_ecl_file` 返回 { success, message, outputPath }。
 * @param {{ sourcePath: string, mapPaths?: Array<string> }} params
 */
export function compileEclFile({ sourcePath, mapPaths = [] }: EclFileParams): Promise<EclResult> {
  return invoke('compile_ecl_file', { sourcePath, mapPaths })
}

/**
 * 反编译 ECL 文件 (.ecl -> .decl) —— 与 thmsg/thstd 同款 object-arg 形式。
 * 注意:Rust 端参数名为 binary_path,Tauri 会做 camelCase 转换,
 * 这里对前端统一暴露成 sourcePath 以保持 4 个工具调用方式一致。
 * @param {{ sourcePath: string, mapPaths?: Array<string> }} params
 */
export function decompileEclFile({ sourcePath, mapPaths = [] }: EclFileParams): Promise<EclResult> {
  return invoke('decompile_ecl_file', { binaryPath: sourcePath, mapPaths })
}

/**
 * 反编译 MSG 文件 (.msg -> .dmsg)
 * @param {{ inputPath: string, outputPath?: string|null, withComments?: boolean }} params
 */
export function decompileMsgFile({
  inputPath,
  outputPath = null,
  withComments = true
}: ScriptFileParams): Promise<MsgResult> {
  return invoke('decompile_msg_file', { inputPath, outputPath, withComments })
}

/**
 * 编译 MSG 文件 (.dmsg -> .msg)
 * @param {{ inputPath: string, outputPath?: string|null }} params
 */
export function compileMsgFile({ inputPath, outputPath = null }: ScriptFileParams): Promise<MsgResult> {
  return invoke('compile_msg_file', { inputPath, outputPath })
}

/**
 * 反编译 STD 文件 (.std -> .dstd)
 * @param {{ inputPath: string, outputPath?: string|null, withComments?: boolean }} params
 */
export function decompileStdFile({
  inputPath,
  outputPath = null,
  withComments = true
}: ScriptFileParams): Promise<StdResult> {
  return invoke('decompile_std_file', { inputPath, outputPath, withComments })
}

/**
 * 编译 STD 文件 (.dstd -> .std)
 * @param {{ inputPath: string, outputPath?: string|null }} params
 */
export function compileStdFile({ inputPath, outputPath = null }: ScriptFileParams): Promise<StdResult> {
  return invoke('compile_std_file', { inputPath, outputPath })
}

/**
 * 解包 .dat 容器到目标目录
 * @param {{ archivePath: string, targetDir: string }} params
 */
export function extractDatFile({ archivePath, targetDir }: ExtractDatParams): Promise<ThdatResult> {
  return invoke('extract_dat_file', { archivePath, targetDir })
}

/**
 * 把目录打包为 .dat 容器
 * @param {{ sourceDir: string, archivePath: string }} params
 */
export function packDatFile({ sourceDir, archivePath }: PackDatParams): Promise<ThdatResult> {
  return invoke('pack_dat_file', { sourceDir, archivePath })
}
