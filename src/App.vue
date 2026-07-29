<template>
  <n-config-provider :theme="darkTheme">
    <n-dialog-provider>
      <n-message-provider>
        <WorkbenchRoot />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
// 这里只搭 naive-ui 的 provider。工作台本身在 WorkbenchRoot 里——
// useMessage / useDialog 必须在 provider 的后代中调用，放在本组件的 setup 取不到。
import { darkTheme, NConfigProvider, NDialogProvider, NMessageProvider } from 'naive-ui'
import WorkbenchRoot from './components/Layout/WorkbenchRoot.vue'
</script>

<style>
/* 全局重置 */
body {
  margin: 0;
  padding: 0;
  background-color: #1e1e1e;
  overflow: hidden; /* 防止浏览器出现原生滚动条 */
}

/* 强制覆盖 naive-ui 可能产生的一点背景色差异 */
.n-layout-header, .n-layout-footer {
  box-sizing: border-box;
}

/* Monaco 的查找/替换说明浮层需要盖过顶部标签栏 */
.context-view,
.overflowingOverlayWidgets,
.overflowingContentWidgets,
.monaco-editor .find-widget,
.monaco-editor .suggest-widget,
.monaco-editor .monaco-hover {
  z-index: 3000 !important;
}
</style>
