import { ensureTruthLanguagesRegistered, msgLanguageId } from '../truth/register'
export { msgLanguageId }
export function ensureMsgLanguageRegistered(): string { ensureTruthLanguagesRegistered(); return msgLanguageId }
