import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

/**
 * 刻意独立于 vite.config.js：
 * - monaco 插件在 happy-dom 里跑不起来，而且它是测试里最不需要的东西
 * - UnoCSS 只影响样式，跑测试时纯属拖慢启动
 *
 * 覆盖范围是**前端领域逻辑**（store / composable / service），不是组件渲染。
 * 组件层要么依赖 naive-ui 的完整 provider 链，要么依赖真实 Tauri，
 * 在这个环境里做出来的断言价值远低于维护成本；那部分交给 Windows 手动验收。
 */
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['tests/**/*.spec.{js,ts}'],
    restoreMocks: true
  }
})
