# 代码审阅记录

最近一次审阅日期：2026-04-08

---

## 1. 总体评估

项目架构清晰，前后端分离得当。第一条 thecl 工作流已完整闭环。
FileTree.vue 已拆分为 3 个文件（原 854 行 → 456 + 149 + 221），职责明确。
文件树已改为懒加载模式，外部文件变更检测已上线。

---

## 2. 已修复的高优先级项

| 项目 | 状态 | 说明 |
|------|------|------|
| thecl 结果 → Monaco 诊断标记 | ✅ 已完成 | toolchain-diagnostics.js 根据 model 计算 marker 宽度，Rust 端规范化诊断路径 |
| 问题面板点击跳转 | ✅ 已完成 | ProblemsPanel.vue 添加 nextTick 修复时序 |
| 文件树展开状态持久化 | ✅ 已完成 | localStorage 按项目保存，恢复时逐层预加载 |
| 外部文件变更检测 | ✅ 已完成 | Rust notify crate + Tauri 事件 + useFileWatcher composable |
| 文件树懒加载 | ✅ 已完成 | Rust 浅层扫描 + NTree on-load + project store loadChildren |
| FileTree.vue 拆分 | ✅ 已完成 | useFileTreeDnD + useFileTreeActions 提取 |
| FileTree expandSaveTimer 清理 | ✅ 已完成 | onBeforeUnmount 中清理 |

---

## 3. 当前风险点

### 3.1 Rust 端 panic 风险（高）

**config.rs** 第 49、73、81、87 行存在 `unwrap()` / `expect()` 调用：
- `ProjectDirs::from().expect()` — 配置目录获取失败会 panic
- `self.config.lock().unwrap()` — Mutex 中毒会 panic（3 处）

**建议**：替换为 `unwrap_or_else(|e| e.into_inner())` 或返回 `Result`。

**error_parser.rs** 第 16、18 行 Regex `unwrap()`：
- 模式是编译时常量，实际不会失败，但应替换为 `expect("valid regex pattern")`。

### 3.2 内存无限增长（中）

**terminal.js** `lines` 数组和 **workbenchReports.js** `outputEntries` / `problemEntries` 数组无上限。长时间编译会话可能导致内存持续增长。

**建议**：添加 `MAX_LINES = 5000` 等上限，超出时截断旧条目。

### 3.3 路径比较跨平台兼容（中）

**toolchain-diagnostics.js** `normalizePath()` 强制转换为反斜杠 + 小写，仅适用于 Windows。若未来支持 macOS/Linux 需调整。

当前仅部署 Windows，可暂不处理。

### 3.4 拖放操作竞态（低）

**useFileTreeDnD.js** 的 `handleTreeDrop` / `handleRootDrop` 中 `projectStore.refresh()` 是异步的，快速连续拖放可能导致状态不一致。

**影响**：低，因用户很难在 refresh 完成前再次触发拖放。

### 3.5 无结构化日志（低）

Rust 端无 `tracing` 或 `log` crate，调试输出仅靠 `println!()` 和 `eprintln!()`。

**建议**：引入 `tracing` + `tracing-subscriber`，方便生产环境排查。

---

## 4. 架构观察

### 4.1 前端

- **正确方向**：Composable 拆分模式（useFileTreeDnD / useFileTreeActions / useFileWatcher / useTheclActions 等）保持了组件精简
- **注意**：App.vue 现在挂载了 6 个 composable（session / shortcuts / semantic / fileWatcher / beforeUnload + 布局逻辑），如果继续增长应考虑提取 WorkbenchBootstrap 组件
- **前端仍为 JavaScript**：新文件建议用 TypeScript，旧文件渐进迁移

### 4.2 后端

- **正确方向**：common/ 模块化清晰（file_watcher / fs_utils / fs_ops / toolchain / cmd_runner / terminal / system_clipboard）
- **懒加载设计合理**：`list_dir_shallow` + `get_dir_children` 分离，`dir_is_empty` 避免空目录误触 on-load
- **FileNode serde**：仅 `is_leaf` 使用 `#[serde(rename = "isLeaf")]`，其余字段保持 snake_case 以兼容现有前端代码

### 4.3 集成层

- 前端 API 模块（src/api/modules/*.js）与 Rust 命令签名已验证一致
- 事件通信：file-system-changed 事件从 Rust emit 到前端 listen，路径清晰
- Store 快照序列化/反序列化逻辑正确

---

## 5. 下一步审阅重点

- msg/std/anm 工具链接入后的注册表模式是否统一
- 项目配置文件格式设计是否合理
- Rust unwrap 清理进度
- 内存限制是否已添加
- 新增 composable 的生命周期管理是否完整
