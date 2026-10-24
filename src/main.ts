// 必须先于编辑器模块加载，让原生命令面板、查找框和右键菜单使用中文。
import 'monaco-editor/esm/nls.messages.zh-cn.js'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

// 引入 UnoCSS 样式 (必须)
import 'virtual:uno.css'

// 引入通用字体 (可选，Naive UI 推荐)
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css' // 等宽字体，编辑器用

const app = createApp(App)
app.use(createPinia())
app.mount('#app')