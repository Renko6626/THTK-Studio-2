import { defineConfig, presetUno } from 'unocss'

/**
 * 这个文件此前名为 `uno.config..js`（多一个点），UnoCSS 从未读到过它，
 * 一直跑的是默认配置。改名让它真正生效的同时去掉了两样东西：
 *
 * - `presetAttributify()`：属性模式 `<div flex>`。代码里一次都没用过，
 *   启用后只是给每条工具类多生成一份 `[flex=""]` 选择器（实测 +1.7KB），
 *   还会让「组件 prop 名恰好撞上工具类名」变成潜在的样式污染源。
 * - `flex-center` / `panel-bg` 两个 shortcut：同样从未被引用。
 *
 * 因此本文件生效前后的 CSS 产物是逐字节相同的——它现在是个诚实的空壳，
 * 好处是以后往里加配置能真的起作用。
 *
 * 注意：仓库没有引入 `@unocss/reset`，所以**没有**全局 CSS reset。
 * 裸 `<button>` / `<input>` 会按操作系统原生样式渲染，写组件时要自己处理
 * （`Welcome/WelcomeView.vue` 的 `.recent-entry` 就是为此加的局部重置）。
 */
export default defineConfig({
  presets: [presetUno()]
})
