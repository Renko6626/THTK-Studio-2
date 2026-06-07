# 前端开发建议

本文档面向 `src/` 下的 Vue / Monaco / Pinia 前端代码。

目标不是把前端做成“普通网页应用”，而是把它做成桌面 IDE 的 UI 层。

---

## 1. 前端职责边界

前端应该负责：

- 工作台布局
- 文件树与标签栏交互
- 编辑器 UI
- 面板系统
- 状态展示
- 用户输入与命令分发
- 将 Rust 返回的结构化数据可视化

前端不应该负责：

- 大规模文件扫描
- 复杂解析
- 符号索引
- 构建流程编排
- 工具链参数推导
- 长时间阻塞任务

原则：

- 前端负责“交互与展示”
- Rust 负责“性能敏感逻辑、系统能力、领域逻辑”

---

## 2. 组件组织建议

当前组件已经开始形成编辑器壳子，但仍然要继续控制复杂度。

建议：

- `App.vue` 只做工作台装配和全局级事件接线
- 文件树、标签栏、终端、编辑器、菜单栏各自保持独立
- 复杂交互逻辑继续下沉到 `composables/`
- 不要把资源管理器业务逻辑继续堆进单个 SFC

当前重点：

- [`src/components/Sidebar/FileTree.vue`](../src/components/Sidebar/FileTree.vue)
  - 现在已经承载了创建、重命名、删除、多选、剪贴板、拖拽、根目录投放等多条链路
  - 后续再加复杂能力时，建议拆出：
    - 资源管理器拖拽逻辑
    - 剪贴板与粘贴逻辑
    - 根目录投放逻辑
    - 节点工具函数

---

## 3. 状态管理建议

当前 Pinia store 划分方向是对的，但还可以继续收敛边界。

建议保持：

- `project store`
  - 工作区路径
  - 文件树数据
  - 加载状态
- `editor store`
  - 标签页
  - 激活文件
  - 脏状态
  - 会话恢复
- `terminal store`
  - 终端面板状态
  - 终端输出
  - shell/cwd
- `view store`
  - 纯 UI 选择状态
  - 面板可见性
  - 资源管理器选中项

不要混合：

- 项目领域状态和 UI 选中状态
- 文件系统真实状态和前端临时展示状态

---

## 4. Monaco 集成建议

Monaco 目前已经承担了编辑器主体，后续建议保持“轻接线，不重耦合”。

建议：

- 继续通过事件或动作桥接菜单栏与 Monaco
- Monaco 内只处理编辑器动作，不直接处理业务逻辑
- 与语言服务的结合后续通过独立适配层完成
- 不要把 Touhou 脚本语义直接硬塞进组件内部

当前建议优先补的 Monaco 相关能力：

- 关闭其他 / 关闭右侧标签页
- 更完整的查找替换体验校验
- 诊断标记与错误跳转
- 文件切换时更稳定的光标与视图状态恢复

---

## 5. 前端目前值得注意的风险

### 5.1 `App.vue` 已经有继续膨胀的趋势

[`src/App.vue`](../src/App.vue) 现在同时承担：

- 工作台布局
- 快捷键分发
- 会话恢复
- 持久化调度
- 关闭保护
- 一部分编辑器动作桥接

建议：

- 如果后续再加更多快捷键或工作台行为，优先拆出：
  - `useWorkbenchShortcuts`
  - `useWorkbenchSession`
  - `useBeforeUnloadGuard`

### 5.2 资源管理器组件已完成拆分 ✅

`FileTree.vue` 已从 854 行拆分为三个文件：

- `FileTree.vue`（456 行）：模板 + 展开状态 + 节点渲染 + 菜单分发 + 懒加载
- `useFileTreeDnD.js`（149 行）：拖放逻辑（树内拖放 + 根目录拖放区域）
- `useFileTreeActions.js`（221 行）：文件操作（剪切/复制/粘贴/删除）+ 路径工具函数

新增功能应继续遵循 composable 拆分模式。

### 5.3 当前前端主要仍是 JavaScript

规划文档目标是 Vue 3 + TypeScript，但当前前端主要还是 `.js`。

这不是立刻必须处理的问题，但需要注意：

- 新增复杂模块时，优先考虑直接用 TypeScript
- 逐步把 store、API 层、领域边界明显的模块迁移到 TS

---

## 6. 前端下一步建议

已完成项：
- ~~文件树展开状态持久化~~ ✅（localStorage 按项目保存，恢复时逐层预加载懒加载目录）
- ~~外部文件变更提示~~ ✅（useFileWatcher composable，监听 Rust notify 事件）
- ~~输出 / 问题面板闭环~~ ✅（thecl → Monaco markers + 问题面板点击跳转）

当前建议优先顺序：

1. msg/std/anm 工作区视图和工具链接入
2. 更完整的标签关闭行为
3. 右侧边栏真实面板（大纲）
4. 真终端前端壳子
5. 语言服务适配层

### 当前 composable 清单

| Composable | 职责 |
|------------|------|
| useWorkbenchSession | 会话恢复与快照自动保存 |
| useWorkbenchShortcuts | 全局快捷键 |
| useBeforeUnloadGuard | 关闭前未保存保护 |
| useFileWatcher | 外部文件变更检测与自动重载 |
| useEclSemanticVocabulary | ECL eclmap 语义数据加载 |
| useEditorActionBridge | 编辑器动作事件桥接 |
| useTheclActions | thecl 编译/反编译/header 调度 |
| useFileOperations | 文件创建/重命名基础操作 |
| useContextMenu | 右键菜单状态与选项生成 |
| useFileTreeDnD | 文件树拖放逻辑 |
| useFileTreeActions | 文件树剪切/复制/粘贴/删除 + 路径工具 |

---

## 7. 前端编码约束

建议继续遵守：

- 新功能先判断属于 UI 层还是领域层
- 前端只接结构化结果，不接混乱文本
- 组件尽量短小明确
- 复杂行为优先抽成 composable
- 共享逻辑优先抽离，不在多个组件里复制粘贴
