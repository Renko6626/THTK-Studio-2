import { ensureTruthLanguagesRegistered, stdLanguageId } from '../truth/register'
export { stdLanguageId }
export function ensureStdLanguageRegistered(): string { ensureTruthLanguagesRegistered(); return stdLanguageId }
