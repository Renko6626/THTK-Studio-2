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
  presets: [presetUno()],

  /**
   * 底部面板 chrome 的共享样式，取自 VS Code Dark Modern 的面板配色。
   *
   * 三条规则是这套观感的关键，改动时别破坏：
   * 1. hover 只加半透明底色，**不加边框**——彩色描边是最不像 VS Code 的地方；
   * 2. 活动态是近白色文字 + 近白色下划线（panelTitle.activeBorder = #e7e7e7），
   *    不是蓝色，蓝色只留给键盘焦点；
   * 3. 动作按钮是方形热区里的 16px 图标，圆角 5px。
   *
   * 放在这里而不是各组件的 scoped style：BottomPanelHost 与 TerminalPanel
   * 的头栏上下相邻，样式必须由同一处定义，否则一改就错位。
   */
  shortcuts: {
    'panel-action':
      'w-6 h-6 shrink-0 flex items-center justify-center rounded-[5px] ' +
      'text-[#9d9d9d] hover:text-[#e7e7e7] hover:bg-white/10 ' +
      // 不要再叠 outline-none：它排在后面会把上面这条焦点环覆盖成透明，
      // 键盘用户就彻底看不到焦点了。
      'focus-visible:[outline:1px_solid_#0078d4] transition-colors',
    // 文字型工具按钮（清空等）。与 panel-action 同一套 hover 语言，只是热区随文字宽。
    'panel-text-action':
      'h-6 px-2 shrink-0 flex items-center rounded-[5px] text-[11px] ' +
      'text-[#9d9d9d] hover:text-[#e7e7e7] hover:bg-white/10 ' +
      'focus-visible:[outline:1px_solid_#0078d4] transition-colors',
    'panel-tab':
      'relative h-full px-2 text-[11px] leading-none flex items-center shrink-0 ' +
      'border-b border-transparent text-[#9d9d9d] hover:text-[#e7e7e7] ' +
      'focus-visible:[outline:1px_solid_#0078d4] transition-colors',
    'panel-tab-active': 'text-[#e7e7e7] border-b-[#e7e7e7]'
  }
})
