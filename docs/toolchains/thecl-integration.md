# thecl 接入设计

本文档基于 `thecl` 官方手册，目标是为 THTK-Studio 设计一套可扩展的 `thecl` 接入方案。

参考来源：

- `thecl` 官方手册：<https://thtk.thpatch.net/master/thecl.1.html>

## 1. 现状

当前项目已经有最基础的 ECL 编译 / 反编译能力：

- 前端：
  - `src/api/modules/compiler.js`
- Rust：
  - `src-tauri/src/modules/ecl/commands.rs`
  - `src-tauri/src/modules/ecl/compiler.rs`
  - `src-tauri/src/modules/ecl/error_parser.rs`

当前实现能完成：

- `.decl -> .ecl` 编译
- `.ecl -> .decl` 反编译
- 自动尝试加载默认 `eclmap`
- 将 `stderr` 解析成 diagnostics

当前实现还缺：

- `thecl -h` header 生成功能
- 更完整的命令参数模型
- 更明确的任务类型区分
- 与输出 / 问题面板的正式闭环
- 为未来多脚本工具接入保留一致接口

## 2. 官方能力总结

根据官方手册，`thecl` 的核心命令是：

- `-c version`
  - 编译 enemy script
- `-d version`
  - dump / 反编译 enemy script
- `-h version`
  - 从输入生成 forward declarations header

官方还明确支持这些选项：

- `-m eclmap`
  - 加载一个或多个 map 文件
- `-j`
  - Shift-JIS / UTF-8 转换
- `-r`
  - 反编译时抑制某些代码转换
- `-s`
  - simple creation mode，编译时不自动补额外指令
- `-x`
  - dump 时输出地址信息

因此从 IDE 接入角度看，`thecl` 不应该只被抽象成：

- compile
- decompile

而应该被抽象成：

- compile
- decompile
- generate header

再叠加一组可选参数。

## 3. 建议的数据模型

### 3.1 前端请求模型

建议新增统一请求结构，而不是继续为每个命令只传裸路径：

```ts
type TheclMode = 'compile' | 'decompile' | 'header'

interface TheclRequest {
  mode: TheclMode
  version: string
  inputPath: string
  outputPath?: string | null
  mapPaths?: string[]
  useShiftJis?: boolean
  rawDump?: boolean
  simpleCreation?: boolean
  showOffsets?: boolean
}
```

说明：

- `version` 不应完全依赖全局默认值，任务级参数应允许覆盖
- `outputPath` 应允许为空，由后端推断默认路径
- 选项字段直接对应 `thecl` 手册能力

### 3.2 通用结果模型

建议最终统一成与 IDE 输出 / 问题面板兼容的结构：

```ts
interface ToolDiagnostic {
  line: number
  column?: number | null
  severity: 'error' | 'warning' | 'info'
  message: string
}

interface ToolOperationResult {
  success: boolean
  tool: 'thecl'
  mode: 'compile' | 'decompile' | 'header'
  scriptKind: 'ecl'
  inputPath: string
  outputPath?: string | null
  message: string
  diagnostics: ToolDiagnostic[]
}
```

这样做的好处：

- 结果可以直接送进底部 `Output / Problems` 面板
- 后续接 `thmsg / thanm / thstd` 时可以继续复用

## 4. Rust 侧建议

### 4.1 不再只保留两个 command

当前只有：

- `compile_ecl_file`
- `decompile_ecl_file`

建议演进为：

- `run_thecl_operation(request: TheclRequest) -> ToolOperationResult`

这样后端可以：

- 统一参数推导
- 统一输出路径推导
- 统一命令拼装
- 统一 diagnostics 解析

如果为了兼容现有前端，短期也可以保留：

- `compile_ecl_file`
- `decompile_ecl_file`

但内部都转发到同一个 `run_thecl_operation` 纯逻辑函数。

### 4.2 命令拼装建议

建议 Rust 内部先做一个纯参数构建函数：

```rust
fn build_thecl_args(request: &TheclRequest) -> Vec<String>
```

行为应包括：

- `compile` 对应 `-c version`
- `decompile` 对应 `-d version`
- `header` 对应 `-h version`
- `mapPaths` 展开为多个 `-m`
- `useShiftJis` 决定是否追加 `-j`
- `rawDump` 对应 `-r`
- `simpleCreation` 对应 `-s`
- `showOffsets` 对应 `-x`

注意：

- `-r`、`-x` 只应在 dump / decompile 模式启用
- `-s` 只应在 compile 模式启用

后端应负责做这些模式约束，不要把无效组合完全交给前端。

### 4.3 输出路径推导建议

建议统一放到 Rust 侧：

- compile:
  - `.decl -> .ecl`
- decompile:
  - `.ecl -> .decl`
- header:
  - 可以考虑默认 `.decl -> .h`
  - 或 `.ecl -> .h`

如果用户显式传了 `outputPath`，优先使用传入值。

## 5. 前端接入建议

### 5.1 菜单与命令层

建议在工作台层把 `thecl` 接入成明确命令，而不是只在点击文件时做隐式行为。

建议命令：

- 编译当前 ECL 源文件
- 反编译当前 ECL 二进制文件
- 为当前 ECL 生成头文件

后续它们都应走统一调用入口。

### 5.2 输出 / 问题面板闭环

当前项目已经有：

- `workbenchReportsStore`
- `OutputPanel`
- `ProblemsPanel`

建议 `thecl` 调用完成后：

1. 把 `message` 写入输出面板
2. 把 `diagnostics` 写入问题面板
3. 如果成功并产出文件：
   - 可选择自动刷新文件树
   - 可选择打开输出文件

### 5.3 编辑器跳转

`ProblemsPanel` 已支持：

- 打开文件
- 跳转到行列

所以 `thecl` 的 diagnostics 只要继续输出：

- `line`
- `column`
- `severity`
- `message`

就能直接闭环。

## 6. 推荐实施顺序

建议按下面顺序推进：

1. Rust 内部抽象统一的 `TheclRequest / ToolOperationResult`
2. 保留旧 command，对内转发到统一实现
3. 前端新增统一 `runTheclOperation` API
4. 将 ECL 编译 / 反编译结果接到 `Output / Problems`
5. 补 header 生成功能
6. 再考虑暴露高级选项：
   - `-r`
   - `-s`
   - `-x`
   - 多 map

## 7. 与多脚本类型兼容的设计原则

因为项目未来不只支持 ECL，所以这里必须避免做成“只适配 ECL 的特殊流”。

建议保持：

- `tool` 和 `scriptKind` 分离
  - `tool = thecl`
  - `scriptKind = ecl`
- 输出 / 问题面板只消费通用结果结构
- 前端命令层按“操作类型”组织，不按“某个 Rust command 名字”组织

也就是说，未来：

- `thecl` 负责 `ecl`
- `thmsg` 负责 `msg`
- `thanm` 负责 `anm`
- `thstd` 负责 `std`

但它们都应该能返回同一套：

- output
- diagnostics
- outputPath

这样 IDE 的上层工作台就不需要知道太多工具链细节。

## 8. 当前最适合的下一步

最适合现在继续做的不是立刻支持所有 `thecl` 高级参数，而是：

1. 先把 ECL 编译 / 反编译正式接进 `Output / Problems`
2. 然后把 Rust 侧 command 统一为一个内部通用 `thecl` 任务入口
3. 最后再补 `header` 和高级选项

这样可以先让第一条真实脚本工作流闭环，再逐步扩展。
