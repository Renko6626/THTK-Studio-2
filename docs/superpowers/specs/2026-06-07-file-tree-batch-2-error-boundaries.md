# 文件树 Batch 2:错误处理与边界

日期:2026-06-07 状态:已批准
审计原文:`docs/file-tree-audit-2026-06-07.md`
前置:Batch 1 已完成(commit `f05dc47`)

## 目标

剩下 3 个 High + 7 个 Medium = **10 条**。主题:让失败可见 / 可恢复 / 可重试,边界情况不再静默。

## 修复清单(对照审计编号)

### H13 — inotify watch 失败静默

`src-tauri/src/common/file_watcher.rs`:`start_watching` 现在返回 `Result<(), String>`,失败时 main.rs 的 `set_project_root` 经 `mcp://report` 发警告卡片(沿用 register_clients 的卡片模式)。

### H14 — DnD rename 后 refresh 失败不回滚

`src/composables/useFileTreeDnD.js`:`handleTreeDrop` / `handleRootDrop` 把 `await projectStore.refresh()` 用 `try/catch`,失败时弹 `message.warning("树状态可能过期,请手动刷新")`。

### H15 — 路径分隔符不一致

新建工具 `src/utils/pathNormalize.js`(导出 `normalizePath(path)`):全部 backslash 转 forward slash,trim 尾部 separator(根目录除外)。

应用点:
- `useFileWatcher.js:25` 比较 tab.path === change.path 前双向 normalize
- `useFileOperations.js` 构建 newPath 时用 normalize 后形式
- 其他文件路径比较点(grep `===.*path` 找出)逐一审核

**保持** backend 输出 OS-native 路径(不强行改 Rust 侧),只在 frontend 边界 normalize。

### M16 — deleteEntries 部分失败不刷新

`src/composables/useFileTreeActions.js:134`:同 C5 模式,per-entry 收集成功/失败,**总是 refresh**,publish 卡片报"删除 X 成功 Y 失败"。

### M17 — OS 外部拖进文件树静默忽略

`src/composables/useFileTreeDnD.js`:
- `handleRootDrop` 检测 `event.dataTransfer.files.length > 0` → 弹 message.info("暂不支持从系统拖入,请用菜单'打开文件夹'")
- `handleRootDragEnter` 同样检测,**不**激活蓝色高亮

### M18 — 多选时菜单仍显示"重命名"

`src/composables/useContextMenu.js:80`:menuOptions 计算时,`selectedKeys.length > 1` 则给"重命名"项 `disabled: true`,而不是只在 select handler 里 toast。

### M19 — 粘贴大目录无 loading

`useFileTreeActions.js:150` `pasteIntoTarget`:进入时 `projectStore.isLoading = true`,finally 还原。`FileTree.vue` 已有 `<n-spin :show="projectStore.isLoading">` 包住 n-tree,这就自动有 loading 转圈。同理给 deleteEntries 加。

### M20 — non-UTF8 文件名 lossy 转换无法操作

**只做"显式报错"半步**:`fs_utils::list_dir_shallow` 在遇到 `name.to_string_lossy()` 出现替换字符 U+FFFD 时,把 FileNode 的 path 改成 None 或加 `__lossy: true` 标志,frontend 看到该标志后操作给出错误"该文件名包含非 UTF-8 字符,IDE 暂不支持"。完整 OsString-backed 路径方案太大,留给后续。

### M22 — treeRenderKey 强制重挂载

`FileTree.vue:256`:取消 `treeRenderKey :key` 重新挂载方案。改用 `v-if` 控制 input 出现/消失。如果绝对必要保留 key,只在真的需要重置 n-tree 内部状态时 bump。

调查:n-tree 在 expandedKeys/selectedKeys 是 prop 时是否会自动 reactive?如果 reactive 失败才必须 remount——大概率 reactive 没问题,可以直接删 `:key`。

### M23 — 空状态 UI 与空 n-tree 同时渲染

`FileTree.vue:94`:`!projectStore.rootPath` 时,n-spin/n-tree 也不应渲染。把 n-spin 块也加 `v-if="projectStore.rootPath"`,空状态 div 改成 `v-else`,二选一。

## 不在范围

- 🟢 L24-L30 → Batch 3
- 已 fixed 的 M21 不重复

## 任务拆分

| Task | 文件 | 描述 |
|---|---|---|
| 1 | `file_watcher.rs` + `main.rs` | H13 — Result 返回 + emit 警告卡 |
| 2 | `useFileWatcher.js` + `useFileOperations.js` + 新 `pathNormalize.js` | H15 — 统一 normalize helper |
| 3 | `useFileTreeDnD.js` | H14 + M17 |
| 4 | `useFileTreeActions.js` + `useContextMenu.js` | M16 + M18 + M19 |
| 5 | `fs_utils.rs` | M20 — lossy 检测 + 标志 |
| 6 | `FileTree.vue` | M22 + M23 |
| 7 | docs 收尾 |

## 验证

每个 task 后 cargo test + npm run build,task 5 可能要加 1-2 个 lossy 检测测试。期望:Rust 测试 87 → 88-89(+1-2),前端构建 ✅。

## 实际落地(2026-06-07)

Rust 测试 88 passed,前端构建 ✅。

| Task | Commit | 内容 |
|---|---|---|
| 1 | df0ba0e | watcher 启动失败 Result + emit 警告卡(H13) |
| 2 | bc85a09 | pathNormalize 工具 + 边界比较升级(H15) |
| 3 | 181904c | DnD refresh 失败提示 + OS 外部拖入拒绝(H14+M17) |
| 4 | 16b665b | delete per-entry + 菜单禁用 + isLoading(M16+M18+M19) |
| 5 | 2049dff | fs_utils lossy 检测 + 前端拒操作 + ⚠️ 标记(M20) |
| 6 | f36f942 | 去 treeRenderKey + 空状态门控(M22+M23) |
| 7 | (此提交) | 验证 + 文档 |
