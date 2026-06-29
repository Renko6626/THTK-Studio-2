# 文件树子系统审阅报告

日期:2026-06-07
范围:~1854 行,5 角度并行 finder 扫描
扫描的文件:
- 前端:`src/components/Sidebar/FileTree.vue` (479)、`src/composables/{useFileTreeActions,useFileTreeDnD,useFileOperations,useContextMenu,useFileWatcher}.js` (~750)、`src/stores/{project,explorerView,explorerClipboard}.js` (~190)
- 后端:`src-tauri/src/common/{fs_utils,fs_ops,file_watcher,system_clipboard}.rs` (~435)

总计 39 个候选,去重后 **30 个独立问题**,按优先级分 4 档。

---

## 🔴 Critical(真 bug — 会丢数据 / 破视图 / 安全)

### C1 · `src-tauri/src/common/fs_ops.rs:6+` — 零路径安全

所有 fs_ops 命令(create_directory / create_file / rename_entry / copy_entry / delete_entry)接受 frontend 传来的 `String`,直接喂给 `std::fs`,**无项目根包含校验、无 `..` 拦截、无符号链接守卫、无文件类型白名单**。

失败场景:被 frontend bug 或恶意 MCP 工具调用 `delete_entry("/home/user/.ssh")` → Rust 直接 `remove_dir_all` 递归删除,等价于任意位置 `rm -rf`。

修法:加 canonicalize → startswith(project_root) 守卫,封装成 `assert_within_project(path)` helper,所有命令前置校验。

### C2 · `src-tauri/src/common/fs_ops.rs:21` — rename_entry 无目标存在检查 + 静默覆盖

`fs::rename` 在 POSIX 上**静默覆盖**目标文件,Windows 行为不同。无 `existing destination` 检查。

失败场景:用户右键 `a.ecl` 重命名为 `b.ecl`(b.ecl 已存在,有未保存的工作)→ b.ecl 被静默 clobber,数据丢失。`copy_entry` 已有此检查,rename 没有。

修法:rename 前加目标 exists 检查,返回 `Err("destination already exists: ...")`。

### C3 · `src-tauri/src/common/fs_ops.rs:69` — copy_dir_all 跟随符号链接

`entry.file_type().is_dir()` 在符号链接指向目录时返回 true,然后递归——会跟着 symlink 拷出工作区,**循环 symlink 直接死循环**直到栈溢出或磁盘满。

失败场景:项目里有 `shared -> /usr/share`,用户复制项目目录 → 把整个 /usr/share 拷出来。或 `a/b/c -> ../../a`(循环)→ 死循环。

修法:用 `symlink_metadata()` 区分真实目录与符号链接,符号链接复制为符号链接(`std::os::unix::fs::symlink` / Windows API)而非跟随。

### C4 · `src-tauri/src/common/file_watcher.rs:59` — 目录事件全被过滤

```rust
if event.path.is_dir() || !seen.insert(...) { continue; }
```

`is_dir()` 对未删除的目录返回 true → 目录 create 直接被跳过。删除的目录 `is_dir()` 返回 false 滑进去但被分类为"file remove"。

失败场景:用户在外部 shell `mkdir build/` → notify 触发 → 被过滤 → 前端永远不刷新。这是你说的"零散 bug"的核心症状。

修法:去掉 `event.path.is_dir()` 过滤(目录事件需要透传),或者改成"任何事件都发,前端自己决定怎么处理"。

### C5 · `src/composables/useFileTreeActions.js:171` — paste 中途失败状态卡死

```js
try { for (entry of entries) { await renameEntry(...) } }
catch (err) { message.error(err); return }  // ← clipboard 未清,refresh 未调
```

失败场景:5 个文件粘到第 3 个挂(EBUSY/权限)→ 后两个静默跳过,`movedAny=true` 但 isCut 模式下剪贴板没清空,下次粘贴再炸。

修法:循环里 per-entry try/catch,记录成功/失败列表,完成后报"X 成功 Y 失败"卡片,**总是清剪贴板 + refresh**。

### C6 · `src/composables/useFileTreeActions.js:156` — 系统剪贴板粘贴目录绕过 cycle 检查

```js
const entry = { path, name, is_dir: false }  // ← 硬编码 false
```

失败场景:OS 文件管理器复制 `/proj/src/`,IDE 内右键 `/proj/src/sub/` 粘贴 → `canCopyEntryIntoDir` 的 `is_dir` 检查跳过 → 后端递归拷贝 `src` 到自己里面,直到栈/路径长度爆。

修法:粘贴前先调 backend 探测 `path` 是文件还是目录(新增 `stat_entry` 命令),或者用 `fs::metadata` 在每次粘贴时探一下。

---

## 🟠 High(影响日常使用)

### H7 · `src/composables/useFileOperations.js:85` — rename 后 selectedKeys 不更新

inline rename(右键 → 输入新名 → Enter)后,`selectedKeys` 还指向旧路径。n-tree 静默丢未知键,用户视觉上失去选中焦点;后续键盘导航起点没了;右键菜单读 selectedKeys 拿不到东西。

参照:`useFileTreeDnD.js:67` 的 DnD rename **有**更新,inline rename 没有。

修法:`submitInput` 成功后,设 `selectedKeys.value = [newPath]` + 同步 `explorerViewStore.setSelectedPaths`。

### H8 · `src/components/Sidebar/FileTree.vue:200` — 重命名展开目录后展开状态丢失

`expandedKeys` 包含 `/foo/sub` 等子路径。重命名 `/foo` → `/bar` 后,watcher 过滤掉所有不在新树里的路径(包括 sub),**但不会把 /foo/sub 重映射成 /bar/sub**。

失败场景:用户展开一个层级很深的目录,改名顶层 → 整个子树全折叠。

修法:rename 时 explicit 转换 expandedKeys:`oldPath` 开头的所有键替换前缀为 `newPath`。

### H9 · `src/stores/project.js:97` — refresh() 无并发守卫

多个 refresh 同时进行 → `getFileTree` 结果互相覆盖,`_collectLoadedDirs` 的并发 `loadChildren` 写入交错。

失败场景:用户删除文件触发 refresh A,500ms 后 watcher 防抖也触发 refresh B,两者重叠时 expanded subtree 随机消失。

修法:加 `_refreshInFlight` Promise,refresh 调用时若 in-flight 就 await 同一个 Promise,完成后再决定要不要再刷一次。

### H10 · `src/stores/project.js:118` — 空目录展开状态丢失

```js
if (node.is_dir && node.children?.length) { result.add(node.path); ... }
```

`children.length > 0` 的检查导致空目录(`children: []`)被排除,refresh 后这个 dir 又显示成"未加载"。

修法:改成 `node.is_dir && node.children !== undefined`(用是否加载过判断,而非有没有子)。

### H11 · `src/components/Sidebar/FileTree.vue:197` — restoreExpandedKeys 与 loadProject 竞态

`watch(rootPath)` 立即触发,此时 `getFileTree` 还在 await,`projectStore.files` 还是上个项目的(或空)。`restoreExpandedKeys` 对错的树调 `loadChildren`,`_mergeChildren` 找不到节点静默 no-op。

修法:`watch` 等到 `files` 实际更新后再恢复,或者把恢复逻辑挪到 `loadProject` 的最后(getFileTree 之后)。

### H12 · `src/composables/useFileOperations.js:55` — 文件名无非法字符校验

用户输 `stage/01` 直接调 backend。Linux 静默创建嵌套目录(不是用户想要的);Windows OS 报错冒泡到 message.error 但无指引。

修法:`submitInput` 前正则校验 `^[^\s/\\:*?"<>|]+$`,失败给具体提示"文件名不能包含 / \ : * ? \" < > |"。

### H13 · `src-tauri/src/common/file_watcher.rs:88` — inotify watch 失败静默

`watch().watch(&root, Recursive)` 错误时 `eprintln` 后 `return`,state 留空 → watcher 完全停掉,前端不知道。

失败场景:Linux 项目大文件多超 `fs.inotify.max_user_watches` → 整个文件改动监听挂掉,用户继续编辑外部修改的脏 buffer。

修法:start_watching 返回 Result,失败时通过 Tauri 事件 emit 警告卡片("文件监听不可用,需手动刷新")。

### H14 · `src/composables/useFileTreeDnD.js:65` — DnD rename 后 refresh 失败不回滚

rename 成功 → selectedKeys 乐观更新到新路径 → refresh 失败 → 树仍显示旧路径但 selection 指向不存在的新路径,UI 静默不一致。

修法:用 `try/finally`,refresh 失败也强制再 refresh 一次或显示"树状态可能过期,点刷新"。

### H15 · `src/composables/useFileWatcher.js:25` — 路径分隔符不一致致 watcher↔tab 匹配失败

watcher 给的是 OS-native (`\\` Windows);tab.path 可能 `/`(从 dialog 来)或 `\\`(JS 拼的)。严格 `===` 匹配失败。

失败场景:用户在 Windows 上编辑外部被修改的文件 → "外部修改"提示不弹,用户继续在 stale buffer 上编辑保存,**外部更改丢失**。

修法:前后端边界统一 normalize 到 `/`(或一个 `normalizePath` helper),或者比较时双向 normalize。

---

## 🟡 Medium(健壮性)

| # | 位置 | 问题 |
|---|---|---|
| M16 | `useFileTreeActions.js:134` | deleteEntries 部分失败不刷新树 → phantom 节点 |
| M17 | `useFileTreeDnD.js:23` | OS 外部拖进文件树静默忽略,根 drop 区还高亮误导 |
| M18 | `useContextMenu.js:80` | 多选时菜单仍显示"重命名",点了弹 toast 才知不支持 |
| M19 | `useFileTreeActions.js:150` | 粘贴大目录无 loading,UI 静默冻结几秒 |
| M20 | `fs_utils.rs:58` | non-UTF8 文件名 lossy 转换后 IDE 无法 rename/delete(Linux 解包 SJIS 命名的 .dat 触发) |
| M21 | `fs_ops.rs:14` | TOCTOU 双击新建会覆盖(改用 `OpenOptions::create_new`) |
| M22 | `FileTree.vue:256` | treeRenderKey 强制重挂载 n-tree 丢滚动/焦点/拖拽中状态 |
| M23 | `FileTree.vue:94` | 空状态 UI 与空 n-tree 同时渲染浪费空间 |

---

## 🟢 Cleanup(整洁/重构)

| # | 内容 |
|---|---|
| L24 | `useContextMenu` / `useFileTreeActions` 重复 `getParentPath` + 一处 `isPathWithin` 落了 `is_dir` 守卫(latent bug) |
| L25 | UA-sniffed separator(`useFileOperations.js:18`)应从路径本身推断,UA 在 Tauri/Wine 不可靠 |
| L26 | `inputState` 模块级单例 + 三个独立 `watch` + 死代码 `handleDelete` |
| L27 | `selectedKeys` 双源(local ref + store)10+ 处手动同步,改 computed 单一来源 |
| L28 | `useFileWatcher` 直接改 tab 对象,bypass store,Monaco model 同步会绕过 |
| L29 | `fs_ops.rs` 零测试覆盖(5 个破坏性命令全裸奔) |
| L30 | delete 没有系统回收站(可选,接 `trash` crate) |

---

## 修复批次规划

### Batch 1(必修)— 数据一致性 + 视图正确性

包含:🔴 全部 6 条 + 🟠 H7/H8/H9/H10/H11/H12

集中在 `fs_ops.rs` + `file_watcher.rs` + `project.js` + `FileTree.vue` 四个文件,会互相 conflict,所以**一批做完**。

### Batch 2(应修)— 错误处理与边界

包含:🟠 H13/H14/H15 + 🟡 全部 8 条

主题:让失败可见、可恢复、可重试。

### Batch 3(整洁)— 减少未来 bug 面

包含:🟢 全部 7 条

主题:去重 + 单一职责 + 测试覆盖。

---

## Spec / Plan 文件命名约定

每个 batch 一个 spec:
- `docs/superpowers/specs/2026-06-07-file-tree-batch-1-data-integrity.md`
- `docs/superpowers/specs/2026-06-07-file-tree-batch-2-error-boundaries.md`
- `docs/superpowers/specs/2026-06-07-file-tree-batch-3-cleanup.md`
