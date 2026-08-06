import * as monaco from 'monaco-editor'
import { analyzeStd, jumpSpecFor } from './jumpNavigation'

/**
 * STD 语言服务：只做**跳转导航**，不做补全 / 高亮 / 诊断。
 *
 * 为什么只有导航：thstd 的跳转目标是裸字节偏移，看着 `jmp(0, 1200)` 完全不知道
 * 跳去哪。生成真 `goto label` 需要把 label 写进 `.dstd`，而那样文件就喂不回
 * thstd 了。导航在编辑器层给到同样的可读性，且不碰文件格式。
 *
 * 换算规格见 `jumpNavigation.ts`，全部照抄 ExpHP/truth 的声明式定义。
 */

export const stdLanguageId = 'thtk-std'

interface StdRegistryState {
  registered: boolean
  /** 当前项目的游戏版本（thtk 数字形式）。决定跳转 opcode 与偏移换算 */
  gameVersion: number | null
  disposables: monaco.IDisposable[]
}

const state: StdRegistryState = {
  registered: false,
  gameVersion: null,
  disposables: []
}

/**
 * 注入当前项目的游戏版本。
 *
 * 拿不到版本就**不提供导航**——不同版本的跳转 opcode 和偏移含义都不同
 * （v0 存指令序号、v1+ 存字节偏移），猜错了会指向完全不相干的行。
 */
export function setStdGameVersion(version: string | number | null | undefined): void {
  const parsed =
    typeof version === 'number' ? version : Number(String(version ?? '').replace(/^th/i, ''))
  state.gameVersion = Number.isFinite(parsed) && parsed > 0 ? parsed : null
}

/** 供测试与调用方检查当前生效的版本 */
export function getStdGameVersion(): number | null {
  return state.gameVersion
}

function analyzeModel(model: monaco.editor.ITextModel) {
  if (state.gameVersion === null) return null
  if (!jumpSpecFor(state.gameVersion)) return null
  return analyzeStd(model.getValue(), state.gameVersion)
}

/**
 * 让跳转行上的偏移实参可 Ctrl+Click / F12 跳到目标行。
 *
 * 用 DefinitionProvider 而不是自定义命令：这是编辑器原生的"跳转到定义"，
 * 用户已有肌肉记忆，也自带返回栈。
 */
function createDefinitionProvider(): monaco.languages.DefinitionProvider {
  return {
    provideDefinition(model, position) {
      const analysis = analyzeModel(model)
      if (!analysis) return null

      const jump = analysis.jumps.find(
        (j) => j.sourceLine === position.lineNumber && j.targetLine !== null
      )
      if (!jump || jump.targetLine === null) return null

      return {
        uri: model.uri,
        range: new monaco.Range(jump.targetLine, 1, jump.targetLine, 1)
      }
    }
  }
}

/**
 * 在跳转行和目标行上各显示一条 code lens。
 *
 * 目标行那条（「← 被第 N 行跳转」）是真正的增量——光看一行指令，
 * 你不可能知道有别处跳到这里。
 */
function createCodeLensProvider(): monaco.languages.CodeLensProvider {
  return {
    provideCodeLenses(model) {
      const analysis = analyzeModel(model)
      if (!analysis) return { lenses: [], dispose: () => {} }

      const lenses: monaco.languages.CodeLens[] = []
      const incoming = new Map<number, number[]>()

      for (const jump of analysis.jumps) {
        if (jump.targetLine === null) {
          // 偏移没落在指令边界上：说出来，而不是假装没有跳转
          lenses.push({
            range: new monaco.Range(jump.sourceLine, 1, jump.sourceLine, 1),
            command: {
              id: '',
              title: `⚠ 跳转目标 ${jump.rawOffset} 未落在指令边界上`
            }
          })
          continue
        }
        lenses.push({
          range: new monaco.Range(jump.sourceLine, 1, jump.sourceLine, 1),
          command: { id: '', title: `→ 跳转到第 ${jump.targetLine} 行` }
        })
        const list = incoming.get(jump.targetLine) || []
        list.push(jump.sourceLine)
        incoming.set(jump.targetLine, list)
      }

      for (const [targetLine, sources] of incoming) {
        lenses.push({
          range: new monaco.Range(targetLine, 1, targetLine, 1),
          command: {
            id: '',
            title: `← 被第 ${sources.join('、')} 行跳转`
          }
        })
      }

      return { lenses, dispose: () => {} }
    },
    resolveCodeLens(_model, codeLens) {
      return codeLens
    }
  }
}

export function ensureStdLanguageRegistered(): string {
  if (state.registered) return stdLanguageId

  monaco.languages.register({ id: stdLanguageId, extensions: ['.dstd'] })
  state.disposables.push(
    monaco.languages.registerDefinitionProvider(stdLanguageId, createDefinitionProvider()),
    monaco.languages.registerCodeLensProvider(stdLanguageId, createCodeLensProvider())
  )
  state.registered = true
  return stdLanguageId
}

/** 仅供测试重置模块级状态 */
export function __resetStdRegistryForTests(): void {
  state.disposables.forEach((d) => d.dispose())
  state.disposables = []
  state.registered = false
  state.gameVersion = null
}
