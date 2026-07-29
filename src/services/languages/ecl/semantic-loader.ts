import { getSettings, getEclMapSemantics } from '../../../api'
import type {
  EclInstructionSpec,
  EclMapSemanticData,
  LoadedEclSemanticData,
  ProjectConfig
} from '../../../types'

export interface LoadSemanticDataOptions {
  projectRoot?: string | null
  projectConfig?: ProjectConfig | null
}

/** 一次成功加载的结果：路径 + 该路径解析出的词表 */
interface LoadedMap {
  path: string
  semantics: EclMapSemanticData
}

function normalizeVersion(version: string | null | undefined): string {
  return String(version || '')
    .trim()
    .toLowerCase()
    .replace(/^th/, '')
}

function joinPath(basePath: string | null | undefined, relativePath: string): string {
  if (!basePath) return relativePath
  const separator = basePath.includes('\\') ? '\\' : '/'
  return `${basePath.replace(/[\\/]+$/, '')}${separator}${relativePath}`
}

/** 同时认 POSIX 的 /path 和 Windows 的 C:\path、\\share */
function isAbsolutePath(path: string): boolean {
  return /^([a-zA-Z]:[\\/]|[\\/])/.test(path)
}

/** 多份 eclmap 合并时按名字去重，靠后的 map 覆盖靠前的（与 thecl 的加载顺序一致） */
function dedupeByName(entries: EclInstructionSpec[]): EclInstructionSpec[] {
  const byName = new Map<string | number, EclInstructionSpec>()
  for (const entry of entries) {
    const key = entry?.name ?? entry?.opcode
    if (key === undefined || key === null) continue
    byName.set(key, entry)
  }
  return [...byName.values()]
}

/** 项目配置里的 map 路径允许写相对路径，按项目根解析（与 Rust 侧 resolve_map_paths 同义） */
function resolveAgainstRoot(
  path: string | null | undefined,
  projectRoot: string | null | undefined
): string {
  const trimmed = String(path || '').trim()
  if (!trimmed) return ''
  return isAbsolutePath(trimmed) ? trimmed : joinPath(projectRoot, trimmed)
}

function createCandidatePaths(version: string, roots: (string | null | undefined)[] = []): string[] {
  if (!version) return []

  const candidates: string[] = []
  for (const root of roots.filter(Boolean)) {
    candidates.push(joinPath(root, `th${version}.eclm`))
    candidates.push(joinPath(root, `${version}.eclm`))
    candidates.push(joinPath(root, joinPath('maps', `th${version}.eclm`)))
    candidates.push(joinPath(root, joinPath('maps', `${version}.eclm`)))
  }

  return [...new Set(candidates)]
}

/**
 * 加载编辑器用的 ECL 词表。
 *
 * 项目配置优先于全局设置：编辑器的补全 / 悬停必须和 thecl 实际使用的 eclmap 同源，
 * 否则一个项目里高亮出来的指令和编译器接受的指令会对不上。工具链调用侧
 * （useTheclActions、Rust 的 effective_toolchain_config）已经是项目优先，这里跟上。
 */
export async function loadDefaultEclSemanticData({
  projectRoot,
  projectConfig
}: LoadSemanticDataOptions = {}): Promise<LoadedEclSemanticData> {
  const settings = await getSettings()
  const version = normalizeVersion(projectConfig?.gameVersion || settings?.default_game_version)

  // 项目声明的**全部** map 都要进词表：thecl 和 mcp 侧都是把 mapPaths 整体传下去的，
  // 只取第一条会让 maps[1..] 里定义的指令在编辑器里查无此项，正好是这套改动
  // 想要消除的"编辑器和编译器不同源"。
  const projectMapPaths = (projectConfig?.mapPaths || [])
    .map(path => resolveAgainstRoot(path, projectRoot))
    .filter(Boolean)

  const globalMapPath = String(settings?.eclmap_path || '').trim()

  // 按 thtk 目录推导 thXX.eclm 的候选位置。这里是"依次尝试"，和后端
  // toolchain::effective_config 的"项目值直接顶掉全局值"不是同一套语义——
  // 后端只需要挑一个 exe 目录，这里多试几个位置能少让用户手配一次。
  const roots = [projectRoot, projectConfig?.toolchain?.thtkDir, settings?.thtk_dir]

  let lastError: unknown = null

  /** 全部加载并合并（项目声明的多份 map 是并列关系，都要进词表） */
  async function loadAll(paths: string[]): Promise<LoadedMap[]> {
    const loaded: LoadedMap[] = []
    for (const path of paths) {
      try {
        loaded.push({ path, semantics: await getEclMapSemantics(path) })
      } catch (error) {
        lastError = error
      }
    }
    return loaded
  }

  /** 取第一个能加载的（推导出的候选是同一份 map 的不同可能位置，不是并列关系） */
  async function loadFirst(paths: string[]): Promise<LoadedMap[]> {
    for (const path of paths) {
      try {
        return [{ path, semantics: await getEclMapSemantics(path) }]
      } catch (error) {
        lastError = error
      }
    }
    return []
  }

  // 三级回落。每一级整体失败才降级——项目里一条过期的 map 路径不该把整个
  // ECL 语言服务打哑，哪怕用户的全局 eclmap 本来是好的。
  let loaded = await loadAll(projectMapPaths)
  if (!loaded.length && globalMapPath) {
    loaded = await loadFirst([globalMapPath])
  }
  if (!loaded.length) {
    loaded = await loadFirst(createCandidatePaths(version, roots))
  }

  if (!loaded.length) {
    return {
      version,
      sourcePath: '',
      resolvedPath: '',
      instructions: [],
      builtins: [],
      error: lastError ? String(lastError) : ''
    }
  }

  const base = loaded[0].semantics
  return {
    ...base,
    instructions: dedupeByName(loaded.flatMap(item => item.semantics?.instructions || [])),
    // builtins 是字符串数组，按值去重（dedupeByName 是给指令对象用的）
    builtins: [...new Set(loaded.flatMap(item => item.semantics?.builtins || []))],
    resolvedPath: loaded.map(item => item.path).join(' + '),
    version: base?.version || version
  }
}
