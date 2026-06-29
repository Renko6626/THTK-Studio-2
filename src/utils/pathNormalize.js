/**
 * 跨平台路径归一化:Windows 反斜杠 → 正斜杠,去除尾部分隔符(根除外)。
 * 仅作字符串处理,不动磁盘;前后端边界用,后端继续吐 OS-native。
 */
export function normalizePath(path) {
  if (typeof path !== 'string' || !path) return path
  let result = path.replace(/\\/g, '/')
  // 去除尾部 / (但 "/" 自身或形如 "C:/" 的盘根要保留)
  if (result.length > 1 && result.endsWith('/') && !/^[A-Za-z]:\/$/.test(result)) {
    result = result.slice(0, -1)
  }
  return result
}

/**
 * 比较两个文件路径是否指向同一位置(跨 Windows/Unix 分隔符)。
 */
export function pathsEqual(a, b) {
  if (a === b) return true
  if (!a || !b) return false
  return normalizePath(a) === normalizePath(b)
}
