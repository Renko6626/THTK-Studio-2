# 工具链平台层设计

本文档用于说明为什么 THTK-Studio 不应继续把 `thecl` 当成唯一特例，而应把 `thtk` 相关工具统一抽象成一层可扩展的“工具链平台层”。

当前目标不是一次性接完所有工具，而是先把“接入新工具需要什么”定义清楚，让后续 `thmsg / thanm / thstd / thdat` 的接入可以沿用同一套结构。

---

## 1. 背景

目前项目已经验证了几件事：

- 外部工具链会升级，路径可能变化
- 某个工具可能需要单独替换，不一定总是跟随 `thtk_dir`
- 每个工具都需要：
  - 路径配置
  - 版本检测
  - 执行入口
  - 结果输出
  - 问题诊断
  - UI 入口

如果继续把这些逻辑只写成 `thecl` 特例，后面每接一个工具，就会复制一套：

- `get_xxx_status`
- `xxx_path`
- `run_xxx_operation`
- `XxxBuildForm`
- `useXxxActions`

这会让前后端都迅速失控。

---

## 2. 当前已经落下的基础

目前项目里已经有了第一版“工具链平台层”雏形：

### Rust 侧

- `AppConfig` 中已有：
  - `thtk_dir`
  - `thecl_path`
  - `tool_overrides`
- `common/toolchain.rs`
  - 统一描述工具链元数据
  - 统一解析工具路径
  - 统一调用 `-v` 做版本检测
  - 统一返回 `ToolchainStatus`
- `main.rs`
  - `get_toolchain_status(tool)`
  - `get_toolchain_statuses()`

### 前端侧

- `services/toolchains/registry.js`
  - 工具链注册表
- `stores/buildDialog.js`
  - 已经按 `tool` 生成默认 payload
- `BuildConfigDialog.vue`
  - 已经按注册表决定表单组件和执行器
- `ToolchainSettingsDialog.vue`
  - 已经能按“工具链列表”显示状态和路径覆盖

这说明后续扩展点已经不需要从零开始。

---

## 3. 建议的统一抽象

### 3.1 后端统一状态结构

所有工具链状态建议统一为：

```ts
interface ToolchainStatus {
  tool: string
  label: string
  exeName: string
  configuredPath: string
  resolvedPath: string
  available: boolean
  version: string
  message: string
}
```

这样设置页、欢迎页、诊断提示都可以复用同一套数据。

### 3.2 前端工具链注册表

每个工具链都应以 descriptor 的形式注册，而不是散落在组件逻辑中。

建议 descriptor 至少包含：

```ts
interface ToolchainDescriptor {
  id: string
  label: string
  exeName: string
  supportsBuildDialog: boolean
  defaultPayload: () => Record<string, unknown>
  buildFormComponent?: Component
  createRequest?: (payload) => unknown
  execute?: (context, request, payload) => Promise<unknown>
}
```

后续如果某个工具没有图形化构建表单，也应该先注册 descriptor，只是先不提供 `buildFormComponent` 和 `execute`。

### 3.3 通用配置模型

建议配置层逐步收敛为：

```ts
interface ToolchainConfig {
  thtkDir: string
  toolOverrides: Record<string, string>
  defaultGameVersion: string
}
```

其中：

- `thtkDir` 负责提供默认根目录
- `toolOverrides` 负责单工具路径覆盖
- 旧的 `thecl_path` 目前保留为兼容字段，但长期应视为过渡方案

---

## 4. 后续接入新工具的推荐流程

以后接一个新工具时，建议按这个顺序做：

1. 在 Rust `common/toolchain.rs` 里确认 descriptor 已注册
2. 在前端 `registry.js` 里补 descriptor
3. 如果需要图形化表单，新增专用 `BuildForm`
4. 在 Rust 侧实现统一 `run_xxx_operation`
5. 将结果适配到统一 `Output / Problems` 模型
6. 视文件类型决定是否进入：
   - 文本编辑视图
   - 二进制说明视图
   - 专用预览 / 资源视图

不要从菜单组件或单个按钮开始做起。

---

## 5. 推荐接入顺序

建议后续工具接入顺序：

1. `thmsg`
2. `thstd`
3. `thanm`
4. `thdat`

原因：

- `thmsg` 和 `thecl` 最像，最适合验证抽象是否合理
- `thstd` 复杂度中等
- `thanm` 和 `thdat` 更容易牵涉资源视图、预览和打包流程，放后面更稳

---

## 6. 当前阶段最重要的约束

后续继续开发时，需要明确这几个原则：

- 不要为每个工具单独发明一套设置界面
- 不要把工具链路径、版本检测、状态提示写死在某个组件里
- 不要让构建弹窗重新回到 `if tool === xxx` 的堆叠模式
- 不要把工具链结果模型做成某一种脚本格式专用

也就是说：

- 设置页应该按“工具链列表”工作
- 构建弹窗应该按“注册表”工作
- 输出 / 问题面板应该按“通用结果”工作
- 工作区主区域应该按“视图类型”工作

---

## 7. 结论

`thecl` 不应该被继续当成特例去补功能，而应该被当成第一种已经跑通的参考实现。

后续接 `thmsg / thanm / thstd / thdat` 时，目标不应是“把同样功能再写四遍”，而应是：

**让它们进入同一套工具链平台层、设置系统、构建弹窗系统和工作区视图系统。**
