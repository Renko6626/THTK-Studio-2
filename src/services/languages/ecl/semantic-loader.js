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

export async function loadDefaultEclSemanticData({ projectRoot } = {}) {
  const settings = await getSettings()
  const version = normalizeVersion(settings?.default_game_version)
  const configuredMapPath = String(settings?.eclmap_path || '').trim()

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

  const roots = [projectRoot, settings?.thtk_dir]
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
