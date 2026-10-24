import * as monaco from 'monaco-editor'
import type { EditorTab } from '../../stores/editor'
import { inferMonacoLanguageId } from '../languages/dispatch'

export interface EditorModelSession {
  model: monaco.editor.ITextModel
  viewState: monaco.editor.ICodeEditorViewState | null
}

/** 只管理当前窗口的 Monaco 模型与视图状态，文件读写和脏状态仍由 editor store 负责。 */
export class EditorModelSessions {
  readonly sessions = new Map<string, EditorModelSession>()

  constructor(private readonly onClose?: (model: monaco.editor.ITextModel) => void) {}

  open(tab: EditorTab): EditorModelSession {
    let session = this.sessions.get(tab.path)
    const language = inferMonacoLanguageId(tab)
    if (!session) {
      session = {
        model: monaco.editor.createModel(tab.content, language, monaco.Uri.file(tab.path)),
        viewState: null
      }
      this.sessions.set(tab.path, session)
    } else {
      if (session.model.getLanguageId() !== language) {
        monaco.editor.setModelLanguage(session.model, language)
      }
      // 外部重载才替换内容，正常切换不清空撤销历史。
      if (session.model.getValue() !== tab.content) session.model.setValue(tab.content)
    }
    return session
  }

  retain(paths: string[]): void {
    const openPaths = new Set(paths)
    for (const [path, session] of this.sessions) {
      if (openPaths.has(path)) continue
      this.onClose?.(session.model)
      session.model.dispose()
      this.sessions.delete(path)
    }
  }

  dispose(): void {
    this.retain([])
  }
}
