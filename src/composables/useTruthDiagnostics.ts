import { ref, watch, onBeforeUnmount } from 'vue'
import { checkTruthSource } from '../api'
import { truthKindForPath } from '../services/languages/truth/kinds'
import { useEditorStore } from '../stores/editor'
import { useProjectStore } from '../stores/project'
import { useWorkbenchReportsStore } from '../stores/workbenchReports'

export function useTruthDiagnostics() {
  const editor = useEditorStore(), project = useProjectStore(), reports = useWorkbenchReportsStore()
  const checking = ref(false), status = ref('等待检查')
  let revision = 0, timer: ReturnType<typeof setTimeout> | undefined
  let disposed = false
  const owners = new Set<string>()
  async function check() {
    clearTimeout(timer)
    const tab = editor.activeTab, kind = truthKindForPath(tab?.path)
    if (!tab || !kind) return
    const token = ++revision, path = tab.path, text = tab.content
    const owner = `truth-check:${path}`
    owners.add(owner)
    checking.value = true
    status.value = '正在检查…'
    try {
      const result = await checkTruthSource(path, kind, text)
      if (disposed || token !== revision) return
      for (const previous of new Set(reports.problemEntries.filter(p => p.path === path && p.operation === 'compile' && ['trumsg', 'trustd'].includes(p.source)).map(p => p.ownerKey))) reports.replaceProblems(previous, [])
      reports.replaceProblems(owner, result.diagnostics.map(d => ({ ...d, source: kind === 'std' ? 'trustd' : 'trumsg', operation: 'check', scriptKind: kind })))
      status.value = result.success ? '检查通过' : result.diagnostics.length ? `${result.diagnostics.length} 处问题` : result.message
    } catch (error) {
      if (!disposed && token === revision) { reports.replaceProblems(owner, []); status.value = `检查失败：${String(error)}` }
    } finally { if (!disposed && token === revision) checking.value = false }
  }
  function schedule() {
    revision++
    clearTimeout(timer)
    checking.value = false
    status.value = '等待检查'
    if (truthKindForPath(editor.activeTab?.path)) timer = setTimeout(() => void check(), 800)
  }
  watch([() => editor.activeTab?.path, () => editor.activeTab?.content, () => project.rootPath, () => project.projectConfig], schedule, { immediate: true })
  watch(() => editor.tabs.map(t => t.path), paths => {
    for (const owner of owners) if (!paths.includes(owner.slice('truth-check:'.length))) { reports.replaceProblems(owner, []); owners.delete(owner) }
  })
  window.addEventListener('thtk:toolchain-settings-saved', schedule)
  onBeforeUnmount(() => { disposed = true; revision++; clearTimeout(timer); window.removeEventListener('thtk:toolchain-settings-saved', schedule); for (const owner of owners) reports.replaceProblems(owner, []) })
  return { checking, status, check }
}
