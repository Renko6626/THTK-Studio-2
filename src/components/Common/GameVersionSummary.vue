<template>
  <div class="version-summary" :class="compact ? 'version-summary-compact' : ''">
    <div class="min-w-0">
      <span class="text-[#9d9d9d]">目标游戏版本：</span>
      <strong v-if="actual" class="text-[#e7e7e7] font-semibold">{{ label }}</strong>
      <span v-else class="text-[#e2c08d]">{{ hasOverride ? '请选择游戏版本' : detail }}</span>
      <span v-if="actual" class="ml-2 text-[#9d9d9d]">{{ explicit ? '本次操作' : source }}</span>
    </div>
    <button v-if="!hasOverride" type="button" class="version-settings" @click="openSettings">修改版本</button>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useEffectiveGameVersion } from '../../composables/useEffectiveGameVersion'
import { useGameVersionsStore } from '../../stores/gameVersions'
import { useProjectStore } from '../../stores/project'
import { useProjectSettingsStore } from '../../stores/projectSettings'
import { useToolchainSettingsStore } from '../../stores/toolchainSettings'
const props = defineProps<{ version?: string; compact?: boolean }>()
const { version, source, detail } = useEffectiveGameVersion()
const games = useGameVersionsStore(), project = useProjectStore(), projectSettings = useProjectSettingsStore(), toolSettings = useToolchainSettingsStore()
const explicit = computed(() => props.version?.trim() || '')
const hasOverride = computed(() => props.version !== undefined)
const actual = computed(() => hasOverride.value ? explicit.value : version.value)
const label = computed(() => {
  const value = actual.value.replace(/^th/i, '')
  const game = games.table.find(g => g.id === Number(value))
  return `TH${value}${game && !props.compact ? ` ${game.title}` : ''}`
})
onMounted(() => { void games.ensureLoaded() })
function openSettings() {
  if (project.rootPath) projectSettings.open()
  else toolSettings.open()
}
</script>
<style scoped>
.version-summary { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 12px; border: 1px solid #454545; background: #252526; font-size: 12px; line-height: 20px; }
.version-summary-compact { border: 0; background: transparent; padding: 0; gap: 8px; }
.version-settings { appearance: none; background: transparent; border: 0; padding: 0; color: #75beff; cursor: pointer; font: inherit; white-space: nowrap; }
.version-settings:hover { text-decoration: underline; }
.version-settings:focus-visible { outline: 1px solid #0078d4; }
</style>
