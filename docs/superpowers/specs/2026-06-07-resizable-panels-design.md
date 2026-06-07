# 可拖动 + 可最大化的工作台面板

日期:2026-06-07
状态:已批准(个人使用,保持精简)

## 目标

底部面板(终端/输出/问题)支持拖动调整高度与一键最大化;左右侧栏支持拖动调整宽度。交互对标 VS Code。

## 方案

自写 `useResizable` composable(pointer events,~60 行),一次实现接三处。
否决:naive-ui n-split(需重排整个布局模板)、CSS resize(手柄/方向不可控)。

## 设计

1. **`src/composables/useResizable.js`**:通用拖动逻辑。
   入参 `{ getValue, setValue, axis: 'x'|'y', invert, min, max }`,返回 `onPointerdown`;
   内部 `setPointerCapture` + pointermove/up,拖动中 body 设置对应 resize cursor。
2. **`src/stores/workbenchPanels.js`** 扩展:
   - state:`bottomPanelHeight: 240`、`leftSidebarWidth: 280`、`rightSidebarWidth: 320`、`bottomMaximized: false`
   - actions:`setBottomPanelHeight`(clamp 100 ~ 视口高-160)、`setLeftSidebarWidth`/`setRightSidebarWidth`(clamp 160~600)、`toggleBottomMaximized`
   - `toSnapshot`/`hydrate` 持久化三个尺寸(`bottomMaximized` 不持久化,刷新后还原普通高度)
3. **`src/components/Layout/WorkbenchLayout.vue`**:改为直接读 panels store(去掉尺寸 props 仪式);
   三个分隔条(6px 命中区,hover/拖动亮 #3b82f6);最大化时编辑器区 `v-show=false`、底部面板 `flex-1`,状态栏保留。
4. **`src/components/ToolWindow/BottomPanelHost.vue`**:"隐藏"旁加 最大化/还原 按钮(⌃/⌄)。
5. **行为细节**:拖动分隔条时若处于最大化则退出最大化;拖到剩余空间 <60px 自动进入最大化。
6. **终端适配**:TerminalPanel 已有 ResizeObserver→fit,零改动。

## 验证

`npm run build` + 手动验收:三处拖动、最大化往返、拖到顶自动最大化、刷新后尺寸保留。

## 不做

- 双击分隔条重置、键盘调整、面板拖出为独立窗口
- 侧栏最大化(无需求)
