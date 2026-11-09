import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { getSettings } from '../api'
import { useProjectStore } from '../stores/project'

/** Mirrors project-version > global-version, without inventing a UI fallback. */
export function useEffectiveGameVersion() {
  const project = useProjectStore()
  const globalVersion = ref<string | null>(null), error = ref('')
  let disposed = false, request = 0
  async function refresh() {
    const token = ++request
    try {
      const settings = await getSettings()
      if (disposed || token !== request) return
      globalVersion.value = settings.default_game_version.trim()
      error.value = ''
    } catch (e) { if (!disposed && token === request) { globalVersion.value = null; error.value = `无法读取全局版本：${String(e)}` } }
  }
  const version = computed(() => project.projectConfigStatus === 'invalid' ? '' : project.gameVersion.trim() || globalVersion.value || '')
  const source = computed(() => project.gameVersion.trim() ? '项目设置' : '全局工具链设置')
  const detail = computed(() => project.projectConfigStatus === 'invalid' ? '项目配置有误，无法确认目标版本' : version.value ? '' : error.value || (globalVersion.value === null ? '正在读取游戏版本…' : '尚未设置游戏版本'))
  onMounted(() => { void refresh(); window.addEventListener('thtk:toolchain-settings-saved', refresh) })
  onBeforeUnmount(() => { disposed = true; window.removeEventListener('thtk:toolchain-settings-saved', refresh) })
  return { version, source, detail }
}
