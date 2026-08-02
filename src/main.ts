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