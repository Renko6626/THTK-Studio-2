import * as monaco from 'monaco-editor'
import { dialectLanguageConfiguration } from '../dialect-config'
import { buildMsgMonarchLanguage } from './tokenizer'

/**
 * MSG 语言服务：目前**只有语法高亮**。
 *
 * 没有跳转导航——MSG 没有跳转指令（`.msgm` 里没有任何 `o` 签名，thmsg 的
 * `th06_find_format` 也没有跳转格式）。没有补全/诊断，与 `modules/msg/` 的
 * 「不做花活」范围一致。
 *
 * 在此之前 `.dmsg` 一路落到 `plaintext`：既没有高亮，连 Ctrl+/ 都没有。
 */
export const msgLanguageId = 'thtk-msg'

interface MsgRegistryState {
  registered: boolean
}

/**
 * Monaco 的注册是全局的，而 Vite 的 HMR 会重复执行本模块——
 * 状态挂 globalThis，重载时不会重复注册。与 ECL 的做法一致。
 */
declare global {
  // eslint-disable-next-line no-var
  var __THTK_MSG_LANGUAGE_REGISTRY__: MsgRegistryState | undefined
}

function getState(): MsgRegistryState {
  if (!globalThis.__THTK_MSG_LANGUAGE_REGISTRY__) {
    globalThis.__THTK_MSG_LANGUAGE_REGISTRY__ = { registered: false }
  }
  return globalThis.__THTK_MSG_LANGUAGE_REGISTRY__
}

export function ensureMsgLanguageRegistered(): string {
  const state = getState()
  if (state.registered) return msgLanguageId

  monaco.languages.register({ id: msgLanguageId, extensions: ['.dmsg'] })
  monaco.languages.setLanguageConfiguration(msgLanguageId, dialectLanguageConfiguration)
  monaco.languages.setMonarchTokensProvider(msgLanguageId, buildMsgMonarchLanguage())
  state.registered = true
  return msgLanguageId
}

/** 仅供测试重置模块级状态 */
export function __resetMsgRegistryForTests(): void {
  globalThis.__THTK_MSG_LANGUAGE_REGISTRY__ = { registered: false }
}
