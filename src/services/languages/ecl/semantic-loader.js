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
  const projectMapPath = resolveAgainstRoot(projectConfig?.mapPaths?.[0], projectRoot)
  const configuredMapPath = projectMapPath || String(settings?.eclmap_path || '').trim()

  if (configuredMapPath) {
    try {
      const semantics = await getEclMapSemantics(configuredMapPath)
      return {
        ...semantics,
        resolvedPath: configuredMapPath,
        version: semantics?.version || version
      }
    } catch (error) {
      return {
        version,
        sourcePath: '',
        resolvedPath: configuredMapPath,
        instructions: [],
        builtins: [],
        error: String(error)
      }
    }
  }

  // 项目级 thtk 目录覆盖也参与候选路径推导，顺序与 toolchain::effective_config 一致
  const roots = [projectRoot, projectConfig?.toolchain?.thtkDir, settings?.thtk_dir]
  const candidates = createCandidatePaths(version, roots)

  let lastError = null
  for (const candidate of candidates) {
    try {
      const semantics = await getEclMapSemantics(candidate)
      return {
        ...semantics,
        resolvedPath: candidate,
        version: semantics?.version || version
      }
    } catch (error) {
      lastError = error
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
