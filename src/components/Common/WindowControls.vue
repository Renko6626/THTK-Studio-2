<template>
  <div class="window-controls" aria-label="窗口控制">
    <button type="button" title="最小化" aria-label="最小化" :disabled="!native" @click="run('minimize')"><svg viewBox="0 0 12 12"><path d="M1 6.5h10" /></svg></button>
    <button type="button" :title="maximized ? '还原' : '最大化'" :aria-label="maximized ? '还原' : '最大化'" :disabled="!native" @click="run('maximize')">
      <svg v-if="maximized" viewBox="0 0 12 12"><path d="M3.5 3.5v-2h7v7h-2M1.5 3.5h7v7h-7z" /></svg>
      <svg v-else viewBox="0 0 12 12"><path d="M1.5 1.5h9v9h-9z" /></svg>
    </button>
    <button type="button" class="window-close" title="关闭" aria-label="关闭" :disabled="!native" @click="run('close')"><svg viewBox="0 0 12 12"><path d="m1.5 1.5 9 9m0-9-9 9" /></svg></button>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
const native = isTauri(), maximized = ref(false), message = useMessage()
let disposed = false, stopResize: UnlistenFn | undefined
async function updateMaximized() {
  const value = await getCurrentWindow().isMaximized()
  if (!disposed) maximized.value = value
}
async function run(action: 'minimize' | 'maximize' | 'close') {
  if (!native) return
  try {
    const window = getCurrentWindow()
    if (action === 'minimize') await window.minimize()
    else if (action === 'maximize') { await window.toggleMaximize(); await updateMaximized() }
    else await window.close()
  } catch (e) { message.error(`窗口操作失败：${String(e)}`) }
}
onMounted(async () => {
  if (!native) return
  try {
    await updateMaximized()
    const unlisten = await getCurrentWindow().onResized(() => { updateMaximized().catch(e => message.error(String(e))) })
    if (disposed) unlisten(); else stopResize = unlisten
  } catch (e) { message.error(`读取窗口状态失败：${String(e)}`) }
})
onBeforeUnmount(() => { disposed = true; stopResize?.() })
</script>
<style scoped>
.window-controls { height: 32px; display: flex; flex-shrink: 0; }
.window-controls button { appearance: none; border: 0; background: transparent; color: #ccc; width: 46px; height: 32px; padding: 0; display: grid; place-items: center; cursor: default; }
.window-controls svg { width: 12px; height: 12px; fill: none; stroke: currentColor; stroke-width: 1; }
.window-controls button:hover { background: #ffffff1a; }
.window-controls .window-close:hover { background: #c42b1c; color: white; }
.window-controls button:focus-visible { outline: 1px solid #0078d4; outline-offset: -2px; }
</style>
