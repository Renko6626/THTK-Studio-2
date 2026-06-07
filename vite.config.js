import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import UnoCSS from 'unocss/vite'
import monacoEditorPlugin from 'vite-plugin-monaco-editor'
const host = process.env.TAURI_DEV_HOST || '127.0.0.1';

// https://vite.dev/config/
export default defineConfig(async () => ({
  optimizeDeps: {
    include: [
      'vue', 
      'pinia', 
      'naive-ui', 
      '@vicons/fluent', 
      'monaco-editor', 
      '@tauri-apps/api', 
      '@tauri-apps/plugin-dialog'
    ]
  },
  plugins: [
    vue(),
    UnoCSS(), // 启用原子化 CSS
    monacoEditorPlugin.default({
      // ECL 不需要标准的语言 worker，只加载基础的即可，减少体积
      languageWorkers: ['editorWorkerService', 'json']
    })
  ],
   envPrefix: ['VITE_', 'TAURI_'],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host,
    hmr: {
      protocol: "ws",
      host,
      port: 1421,
    },
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
