# 文件树 Batch 1:数据一致性 + 视图正确性

日期:2026-06-07 状态:已批准
审计原文:`docs/file-tree-audit-2026-06-07.md`

## 目标

把 12 个 Critical+High findings 一次修完。集中在 `fs_ops.rs` / `file_watcher.rs` / `project.js` / `FileTree.vue` 四个文件,它们互相 conflict,所以**一批做完**。

## 修复清单(对照审计编号)

### 后端 Rust 侧

**C1 — fs_ops 零路径安全** (`src-tauri/src/common/fs_ops.rs`)
新增 `pub(crate) fn assert_within_project(path: &Path, project_root: &Path) -> Result<(), String>`(canonicalize → starts_with);所有 5 个命令前置校验。从 `AppState.current_project_root` 取根。无项目根时拒绝所有操作(除非命令 spec 允许)。
✗ 注意:已打开文件的绝对路径校验也走这里(create_file 的 target_path 必须在 root 下)

**C2 — rename_entry 静默覆盖** (`fs_ops.rs:21`)
rename 前 `if new_path.exists() { return Err("Destination already exists: ..."); }`。

**C3 — copy_dir_all 跟随符号链接** (`fs_ops.rs:69`)
用 `symlink_metadata()` 区分。Unix 上 symlink 用 `std::os::unix::fs::symlink` 复制为 symlink,Windows 上(本期目标平台)用 `std::os::windows::fs::{symlink_file, symlink_dir}` 或者更简单——**直接报错"refuse to copy symlinks"**(本期 modding 用例罕见);两选一**取报错方案**(简单且安全)。

**C4 — file_watcher 目录事件过滤** (`file_watcher.rs:59`)
去掉 `event.path.is_dir() || ...` 中 `is_dir()` 过滤;改成 `if !seen.insert(...) { continue }`。kind 判断对目录也走 `if exists() "modify" else "remove"` 路径,这样新建/删除目录都能通过事件流到前端。

**M21(顺手) — TOCTOU 双击新建** (`fs_ops.rs:14`)
`create_file` 用 `OpenOptions::new().write(true).create_new(true).open(...)` 替代 `fs::write(path, "")`,获得原子 O_EXCL。

**新加测试覆盖**(L29 部分还掉):至少给 fs_ops 加 6 个测试,覆盖 assert_within_project / rename existing dest 拒绝 / copy symlink 拒绝 / create_new atomic / delete within-root / cross-platform path normalization。

### 前端 JS 侧

**C5 — paste 中途失败** (`useFileTreeActions.js:171`)
循环改 per-entry try/catch,收集 `succeeded` / `failed` 列表,完成后:
- isCut 模式:**只清成功项对应的剪贴板条目**(若混合 cut+copy 不存在,简化为"全部成功才清剪贴板,否则不清")
- **总是** `await projectStore.refresh()`
- message 改成 `${succeeded.length} 成功,${failed.length} 失败:${failed[0].message}` 摘要卡片

**C6 — system clipboard paste 绕过 cycle 检查** (`useFileTreeActions.js:156`)
新增 backend 命令 `stat_entry(path) -> { is_dir, size, exists }`(`fs_ops.rs` 加),paste 前对系统剪贴板每个 path 探测一次,正确设置 `is_dir`。然后正常的 `canCopyEntryIntoDir`/`canMoveEntryIntoDir` 守卫生效。

**H7 — inline rename 后 selectedKeys 不更新** (`useFileOperations.js:85`)
`submitInput` rename 成功后,调用 caller 提供的 `onRenamed(newPath)` 回调;`FileTree.vue` 用它更新 `selectedKeys` + `explorerViewStore`。

**H8 — 重命名展开目录后展开状态丢失** (`FileTree.vue:200` 附近 + 新建工具)
新增 helper `remapExpandedKeys(oldPath, newPath)`:把 `expandedKeys` 里所有以 oldPath 或 `${oldPath}${sep}` 开头的键替换为 newPath 前缀。rename 成功(inline + DnD)都调用。

**H9 — refresh() 无并发守卫** (`stores/project.js:97`)
加 `_refreshPromise` state:`refresh()` 检查若有 in-flight 就 await 同一个,并在结束时清空。**再次刷新需要的话**(in-flight 期间又有变更)加一个"挂起的刷新"标志,完成后再跑一次。
```js
async refresh() {
  if (!this.rootPath) return
  if (this._refreshPromise) {
    this._refreshPending = true  // queue another after current
    return this._refreshPromise
  }
  this._refreshPromise = this._doRefresh()
    .finally(() => {
      this._refreshPromise = null
      if (this._refreshPending) {
        this._refreshPending = false
        this.refresh()
      }
    })
  return this._refreshPromise
}
```

**H10 — 空目录展开状态丢失** (`stores/project.js:118`)
```js
if (node.is_dir && node.children !== undefined) result.add(node.path)
```
(把 `children?.length` 改成 `children !== undefined`——加载过但空也算"已展开"。)

**H11 — restoreExpandedKeys 与 loadProject 竞态** (`FileTree.vue:197`)
把 `restoreExpandedKeys` 的触发从 `watch(rootPath)` 改到 `watch(() => projectStore.files, ..., { flush: 'post' })`,等树更新后再恢复。或在 `project.js loadProject` 末尾 emit 一个 ready signal。**取 watch files 的方案**(改 FileTree.vue 一处)。

**H12 — 文件名无非法字符校验** (`useFileOperations.js:55`)
`submitInput` 前正则校验 `^[^/\\:*?"<>|\x00-\x1f]+$`,失败显式弹错。同时禁止纯空格/全角空格/`.`/`..`。

## 不在范围(留给 Batch 2/3)

- 🟠 H13(inotify 失败)/ H14(DnD refresh 失败回滚)/ H15(路径分隔符 normalize)→ Batch 2
- 🟡 M16-M23 全部 → Batch 2
- 🟢 L24-L30 → Batch 3
- 历史的 **L30 trash** 暂不做,delete 仍是永久删除

## 实现路线

按文件分组,降低 conflict:

### Step 1 — fs_ops.rs(C1+C2+C3+M21)
所有后端命令的安全护栏;**新增 6+ 个测试**。

### Step 2 — file_watcher.rs(C4)
目录事件透传;无新测试(集成测试要 desktop 环境,跳)。

### Step 3 — project.js(H9+H10)
refresh 并发守卫 + _collectLoadedDirs 修正。

### Step 4 — useFileTreeActions.js + 新增 stat_entry 命令(C5+C6)
- fs_ops.rs 加 `stat_entry`;main.rs 注册
- pasteIntoTarget 改 per-entry 错误处理 + 系统剪贴板探测

### Step 5 — useFileOperations.js + FileTree.vue(H7+H8+H11+H12)
- submitInput 加 onRenamed 回调 + 非法字符校验
- FileTree.vue 加 remapExpandedKeys + 改 restore watch 源

### Step 6 — 验证 + 文档
- cargo test(应 +6 ~ +8 个新测试)
- npm run build
- 更新 `editor-shell-status.md` 加 Batch 1 完成条目
- 把 audit 报告里 Critical/High 划掉对应项

## 任务拆分(给 subagent-driven)

| Task | 文件 | 描述 |
|---|---|---|
| 1 | `fs_ops.rs` + tests | C1+C2+C3+M21 + stat_entry 新增 + 6 测试(后续 C6 复用) |
| 2 | `file_watcher.rs` | C4 — 去 is_dir 过滤 |
| 3 | `project.js` | H9+H10 |
| 4 | `useFileTreeActions.js` | C5+C6(依赖 Task 1 的 stat_entry) |
| 5 | `useFileOperations.js` + `FileTree.vue` | H7+H8+H11+H12 |
| 6 | docs | 收尾 |
