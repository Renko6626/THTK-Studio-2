// uno.config.js
import { defineConfig, presetUno, presetAttributify } from 'unocss'

export default defineConfig({
  presets: [
    presetUno(),       // 默认预设 (兼容 Tailwind CSS)
    presetAttributify(), // 属性模式 (允许直接写 <div flex red-500>，超好用)
  ],
  // 自定义你的快捷方式
  shortcuts: {
    'flex-center': 'flex items-center justify-center',
    'panel-bg': 'bg-gray-100 dark:bg-gray-900', // 适配深色模式的背景
  }
})