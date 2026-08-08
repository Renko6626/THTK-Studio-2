import { eclLanguageId } from './ecl/language-config'
import { msgLanguageId } from './msg/register'
import { stdLanguageId } from './std/register'

/**
 * 按标签路径决定用哪个 Monaco 语言。
 *
 * 放在 `languages/` 顶层而不是某个语言模块里：它是**跨语言**的分派，住在
 * `ecl/register.ts` 时形成了 ecl → msg/std 的倒挂依赖，而且导致只想验分派的
 * 测试被迫加载整条 ECL 补全链。
 *
 * 三种方言各有各的文本形状（.decl 是类 C 语法、.dstd 是 `720:` + `name(...);`、
 * .dmsg 是 `@120` + `\tname(...)`），所以是三个语言而不是一个。
 */
export function inferMonacoLanguageId(
  tab: { path?: string | null; language?: string | null } | null | undefined
): string {
  const path = String(tab?.path || '').toLowerCase()
  const language = String(tab?.language || '').toLowerCase()

  if (path.endsWith('.decl') || path.endsWith('.tecl')) return eclLanguageId
  // .dstd / .dmsg 各有自己的语言服务（见 services/languages/{std,msg}/）。
  // 它们的文本形状彼此不同、也与 ECL 不同，不能共用一套高亮。
  if (path.endsWith('.dstd')) return stdLanguageId
  if (path.endsWith('.dmsg')) return msgLanguageId
  if (language === 'json' || path.endsWith('.json')) return 'json'
  if (language === 'js' || path.endsWith('.js')) return 'javascript'
  if (language === 'ts' || path.endsWith('.ts')) return 'typescript'
  if (language === 'html' || path.endsWith('.vue') || path.endsWith('.html')) return 'html'
  if (language === 'c' || path.endsWith('.c')) return 'c'
  if (language === 'cpp' || path.endsWith('.cpp') || path.endsWith('.h')) return 'cpp'

  return 'plaintext'
}
