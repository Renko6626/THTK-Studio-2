/** The path is authoritative; cached FileNode metadata can lag behind a rename. */
export function fileExtension(path: string): string {
  const name = path.split(/[\\/]/).pop() || ''
  const dot = name.lastIndexOf('.')
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : ''
}
