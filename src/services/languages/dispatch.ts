import { eclLanguageId } from './ecl/language-config'
import { msgLanguageId } from './msg/register'
import { stdLanguageId } from './std/register'
import { endingLanguageId } from './truth/register'

/** ECL 与 truth 分开；truth 三类源码共享语法，各自加载对应 map。 */
export function inferMonacoLanguageId(
  tab: { path?: string | null; language?: string | null } | null | undefined
): string {
  const path = String(tab?.path || '').toLowerCase()
  const language = String(tab?.language || '').toLowerCase()

  if (path.endsWith('.decl') || path.endsWith('.tecl')) return eclLanguageId
  if (path.endsWith('.stdspec')) return stdLanguageId
  if (path.endsWith('.endspec')) return endingLanguageId
  if (path.endsWith('.msgspec')) return msgLanguageId
  if (language === 'json' || path.endsWith('.json')) return 'json'
  if (language === 'js' || path.endsWith('.js')) return 'javascript'
  if (language === 'ts' || path.endsWith('.ts')) return 'typescript'
  if (language === 'html' || path.endsWith('.vue') || path.endsWith('.html')) return 'html'
  if (language === 'c' || path.endsWith('.c')) return 'c'
  if (language === 'cpp' || path.endsWith('.cpp') || path.endsWith('.h')) return 'cpp'

  return 'plaintext'
}
