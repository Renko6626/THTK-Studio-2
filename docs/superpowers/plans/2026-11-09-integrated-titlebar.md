# 一体式标题栏

用户已确认：取消系统外层标题栏，将窗口控制并入现有菜单。

- tauri.conf：decorations=false，shadow=false；保留 resizable 默认行为，窗口背景匹配编辑器。
- MenuBar：现有菜单 + 可拖动项目标题 + WindowControls；拖动区与菜单/按钮分开，避免事件冲突。MenuBar 作为全宽 titlebar，TabGroup 留在编辑区顶部；保留编辑区既有68px顶部总高度。
- WindowControls：最小化、最大化/还原、关闭，最大化图标随窗口 resize 更新；错误提示，监听卸载清理。
- useWindowCloseGuard：系统关闭与自绘关闭按钮同走 onCloseRequested。脏文件确认、取消不退出；允许退出前刷新会话快照。复用 beforeunload guard，避免重复确认。
- capabilities：仅主窗口开放所需窗口命令。onCloseRequested API 内部通过 destroy 完成获准关闭，需对应权限。
- 验证：关闭保护行为测试、typecheck、前端构建、Rust配置/权限编译；样式与原生行为验证分开，不做自动浏览器交互。

## 完成记录

已按上述方案实施；底部面板美化保留。关闭确认沿用会话恢复机制，允许退出前尝试刷新快照，beforeunload 不重复提示。关闭提示明确警告未保存修改可能丢失，不承诺快照一定恢复。

验证：npm run typecheck、npm test（审阅前123 passed）、npm run build、tauri-env linux cargo check、tauri-env win cargo xwin check --release --target x86_64-pc-windows-msvc、git diff --check 通过。关闭保护测试覆盖取消/允许、重复请求和 beforeunload 不重复阻止。浏览器仅以静态样例截图核对全宽顶部和底部面板，未执行自动交互。

Windows/Linux 实机操作拖动、缩放和关闭尚未验收；静态编译与截图不代替这些原生行为验收。

## sol 审阅修正

- 修正关闭提示的无条件草稿保留承诺，明确提示未保存修改可能丢失。保留原有尽力保存的会话快照，不新增持久化机制。删除与实际存储吞错行为不符的 mock 抛错用例，保留取消/允许退出、并发关闭与卸载清理用例。
- 菜单组可收缩和横向滚动，标题可缩小，右侧窗口控制固定保留，避免大倍率缩放裁掉关闭按钮。
- 最大化图标查询时序及销毁失败后的状态恢复暂缓；未验证原生实机交互。

审阅后验证：122 项前端测试和 typecheck 通过；1200px 与 412px CSS 视口截图及元素边界检查均确认窗口控制完整可见（412px 对应最小1024px窗口在高倍率缩放下的布局）。sol 审阅无其他阻断项。后续构建结果以本次生成安装包为准。
