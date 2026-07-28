import { getSettings, getEclMapSemantics } from '../../../api'

function normalizeVersion(version) {
  return String(version || '')
    .trim()
    .toLowerCase()
    .replace(/^th/, '')
}

function joinPath(basePath, relativePath) {
  if (!basePath) return relativePath
  const separator = basePath.includes('\\') ? '\\' : '/'
  return `${basePath.replace(/[\\/]+$/, '')}${separator}${relativePath}`
}

/** 同时认 POSIX 的 /path 和 Windows 的 C:\path、\\share */
function isAbsolutePath(path) {
  return /^([a-zA-Z]:[\\/]|[\\/])/.test(path)
}

/** 多份 eclmap 合并时按名字去重，靠后的 map 覆盖靠前的（与 thecl 的加载顺序一致） */
function dedupeByName(entries) {
  const byName = new Map()
  for (const entry of entries) {
    const key = entry?.name ?? entry?.opcode
    if (key === undefined || key === null) continue
    byName.set(key, entry)
  }
  return [...byName.values()]
}

/** 项目配置里的 map 路径允许写相对路径，按项目根解析（与 Rust 侧 resolve_map_paths 同义） */
function resolveAgainstRoot(path, projectRoot) {
  const trimmed = String(path || '').trim()
  if (!trimmed) return ''
  return isAbsolutePath(trimmed) ? trimmed : joinPath(projectRoot, trimmed)
}

function createCandidatePaths(version, roots = []) {
  if (!version) return []

  const candidates = []
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
export async function loadDefaultEclSemanticData({ projectRoot, projectConfig } = {}) {
  const settings = await getSettings()
  const version = normalizeVersion(projectConfig?.gameVersion || settings?.default_game_version)

  // 项目声明的**全部** map 都要进词表：thecl 和 mcp 侧都是把 mapPaths 整体传下去的，
  // 只取第一条会让 maps[1..] 里定义的指令在编辑器里查无此项，正好是这套改动
  // 想要消除的"编辑器和编译器不同源"。
  const projectMapPaths = (projectConfig?.mapPaths || [])
    .map(path => resolveAgainstRoot(path, projectRoot))
    .filter(Boolean)

  const globalMapPath = String(settings?.eclmap_path || '').trim()
  const explicitPaths = projectMapPaths.length ? projectMapPaths : [globalMapPath].filter(Boolean)

  // 按 thtk 目录推导 thXX.eclm 的候选位置。这里是"依次尝试"，和后端
  // toolchain::effective_config 的"项目值直接顶掉全局值"不是同一套语义——
  // 后端只需要挑一个 exe 目录，这里多试几个位置能少让用户手配一次。
  const roots = [projectRoot, projectConfig?.toolchain?.thtkDir, settings?.thtk_dir]

  // 显式配置的路径全部失败时，继续走目录推导；否则项目里一条过期的 map 路径
  // 会把整个 ECL 语言服务打哑，哪怕全局配置本来是好的。
  const candidates = [...explicitPaths, ...createCandidatePaths(version, roots)]

  const merged = { instructions: [], builtins: [] }
  const resolvedPaths = []
  let lastError = null
  let base = null

  for (const candidate of candidates) {
    try {
      const semantics = await getEclMapSemantics(candidate)
      base = base || semantics
      resolvedPaths.push(candidate)
      merged.instructions.push(...(semantics?.instructions || []))
      merged.builtins.push(...(semantics?.builtins || []))

      // 显式配置的多条 map 要合并；目录推导出的候选只取第一个命中的
      if (!explicitPaths.includes(candidate)) break
    } catch (error) {
      lastError = error
    }
  }

  if (resolvedPaths.length) {
    return {
      ...base,
      instructions: dedupeByName(merged.instructions),
      builtins: dedupeByName(merged.builtins),
      resolvedPath: resolvedPaths.join(' + '),
      version: base?.version || version
    }
  }

  return {
    version,
    sourcePath: '',
    resolvedPath: '',
    instructions: [],
    builtins: [],
    error: lastError ? String(lastError) : ''
  }
}
