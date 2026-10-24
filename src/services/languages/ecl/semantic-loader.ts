import { getSettings, getEclMapSemantics, getDependencyStatus } from '../../../api'
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

/**
 * eclmap 文件名里的版本与项目版本不符时返回提示文案，否则 null。
 *
 * **只警告不拦截**：用户可能在用改造版或自制 eclmap，文件名未必反映内容。
 * 只要有任意一份 map 的文件名与项目版本相符，就认为这套配置是有意的。
 * 文件名完全不含版本（如 `custom.eclm`）同样不报——那是常见的自定义命名。
 *
 * @param version 纯数字形式的版本（后端已归一），例如 `"20"`
 */
export function detectEclmapVersionMismatch(
  mapPaths: string[],
  version: string
): string | null {
  const trimmed = String(version || '').trim()
  if (!trimmed || !mapPaths.length) return null

  const expected = `th${trimmed.toLowerCase().replace(/^th/, '')}`

  const named = mapPaths
    .map((path) => /(?:^|[\\/])(th\d+)\b/i.exec(path)?.[1]?.toLowerCase())
    .filter((code): code is string => Boolean(code))

  if (!named.length) return null
  if (named.includes(expected)) return null

  const found = [...new Set(named)].join(' / ')
  return `项目版本是 ${expected}，但 eclmap 看起来是 ${found} 的——补全与编译可能用到错误的指令签名。`
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

/**
 * 内置基线的规范文件名，对应 Rust `dependencies::BUNDLED_ECL_MAPS`。
 *
 * 内置版本会先把规范名放进候选首位；注意 th095 的规范名是 `th095.eclm`，
 * 但补零 stem 只对 id < 10 生效，所以不会额外生成 `095.eclm` / `095.eclmap`。
 */
const BUNDLED_MAP_FILE_BY_ID: Readonly<Record<number, string>> = {
  6: 'th06.eclm',
  7: 'th07.eclm',
  8: 'th08.eclm',
  9: 'th09.eclm',
  95: 'th095.eclm',
  10: 'th10.eclm',
  103: 'th103.eclm',
  11: 'th11.eclm',
  12: 'th12.eclm',
  125: 'th125.eclm',
  15: 'th15.eclm',
  18: 'th18.eclm',
  185: 'th185.eclm',
  19: 'th19.eclm',
  20: 'th20.eclm'
}

/** 纯数字版本 id；无法解析时返回 null（不拿可疑值去拼文件路径） */
function parseVersionId(version: string): number | null {
  const trimmed = String(version || '').trim().toLowerCase().replace(/^th/, '')
  if (!/^\d+$/.test(trimmed)) return null
  return Number(trimmed)
}

/**
 * 版本号在 map 文件名里的 stem 写法，与 Rust `dependencies::map_file_names` 一致：
 * id < 10 用 `[id, 0id]`，其余只用 `[id]`。th095 的补零规范名由
 * `BUNDLED_MAP_FILE_BY_ID` 单独提供，不在这里重复生成。
 */
function versionStems(version: string): string[] {
  const id = parseVersionId(version)
  if (id === null) return []

  const stems = [String(id)]
  if (id < 10) stems.push(`0${id}`)
  return [...new Set(stems)]
}

/**
 * 某个版本可能的 map 文件名，顺序与 Rust `dependencies::map_file_names` 对齐：
 * 内置规范名（若有）> 每个 stem 的 `th{stem}.eclm` / `{stem}.eclm` / `{stem}.eclmap`。
 */
function mapFileNames(version: string): string[] {
  const id = parseVersionId(version)
  if (id === null) return []

  const names: string[] = []
  const push = (name: string) => {
    if (!names.includes(name)) names.push(name)
  }

  const canonical = BUNDLED_MAP_FILE_BY_ID[id]
  if (canonical) push(canonical)

  for (const stem of versionStems(version)) {
    push(`th${stem}.eclm`)
    push(`${stem}.eclm`)
    push(`${stem}.eclmap`)
  }

  return names
}

/**
 * 把候选目录与文件名展开成实际路径。
 *
 * 目录由调用方按优先级预先展开（托管目录、有效 thtk 的 `maps/`），
 * 这里不再自动追加 `maps`，避免同一目录被重复尝试、也避免把 thtkDir 根级
 * 误当成 map 目录——Rust 的 `resolve_default_maps` 只查 `<thtkDir>/maps`。
 */
function createCandidatePaths(
  version: string,
  directories: (string | null | undefined)[] = []
): string[] {
  const names = mapFileNames(version)
  if (!names.length) return []

  const candidates: string[] = []
  for (const directory of directories.filter(Boolean)) {
    for (const name of names) {
      candidates.push(joinPath(directory, name))
    }
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
  // 后端 project_config::canonicalize_game_version 已把项目值归一成纯数字，
  // 这里不再二次处理。全局默认值来自设置对话框的下拉框，同样是纯数字。
  const version = String(projectConfig?.gameVersion || settings?.default_game_version || '').trim()

  // 项目声明的**全部** map 都要进词表：thecl 和 mcp 侧都是把 mapPaths 整体传下去的，
  // 只取第一条会让 maps[1..] 里定义的指令在编辑器里查无此项，正好是这套改动
  // 想要消除的"编辑器和编译器不同源"。
  const projectMapPaths = (projectConfig?.mapPaths || [])
    .map(path => resolveAgainstRoot(path, projectRoot))
    .filter(Boolean)

  // 只针对项目声明的 map 做校验：全局 eclmap 与推导出的候选路径本来就是
  // 跨项目复用的，拿项目版本去比对没有意义。
  const versionWarning = detectEclmapVersionMismatch(projectConfig?.mapPaths || [], version)

  const globalMapPath = String(settings?.eclmap_path || '').trim()

  // IDE 托管的 map（deps/maps/bundled）路径只从后端取，不在前端拼用户目录。
  // 拿不到状态时只是少一级候选，项目/全局 map 与 thtk_dir 推导照常工作。
  let bundledMapDir = ''
  try {
    const dependencyStatus = await getDependencyStatus()
    bundledMapDir = String(dependencyStatus?.maps?.bundledDir || '').trim()
  } catch {
    bundledMapDir = ''
  }

  // 与 Rust resolve_default_maps 一致：内置基线 > 有效 thtkDir/maps。
  // 项目目录里的 map 需要通过 mapPaths 显式指定；项目 thtkDir 非空时覆盖全局目录。
  const effectiveThtkDir = projectConfig?.toolchain?.thtkDir?.trim() || settings?.thtk_dir?.trim()
  const candidateDirs = [
    bundledMapDir,
    effectiveThtkDir ? joinPath(effectiveThtkDir, 'maps') : ''
  ]

  let lastError: unknown = null

  // 显式路径必须全部加载成功。与编译器保持相同输入，不发布残缺词表或静默换源。
  async function loadExplicit(paths: string[]): Promise<LoadedMap[]> {
    const loaded: LoadedMap[] = []
    for (const path of paths) {
      try {
        loaded.push({ path, semantics: await getEclMapSemantics(path) })
      } catch (error) {
        lastError = `无法加载 ECL map ${path}：${String(error)}`
        return []
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

  const loaded = projectMapPaths.length
    ? await loadExplicit(projectMapPaths)
    : globalMapPath
      ? await loadExplicit([globalMapPath])
      : await loadFirst(createCandidatePaths(version, candidateDirs))

  if (!loaded.length) {
    return {
      version,
      sourcePath: '',
      resolvedPath: '',
      instructions: [],
      builtins: [],
      error: lastError ? String(lastError) : '',
      versionWarning: versionWarning || undefined
    }
  }

  const base = loaded[0].semantics
  return {
    ...base,
    instructions: dedupeByName(loaded.flatMap(item => item.semantics?.instructions || [])),
    // builtins 是字符串数组，按值去重（dedupeByName 是给指令对象用的）
    builtins: [...new Set(loaded.flatMap(item => item.semantics?.builtins || []))],
    resolvedPath: loaded.map(item => item.path).join(' + '),
    version: base?.version || version,
    versionWarning: versionWarning || undefined
  }
}
