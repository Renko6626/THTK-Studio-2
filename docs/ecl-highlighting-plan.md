# ECL 语法高亮设计计划

这份文档用于规划 THTK-Studio 中 `.decl` / ECL 脚本的语法高亮体系。

结论先行：高亮系统应拆成两层，而不是只依赖 `eclm`。

- 第一层：固定语法层
- 第二层：`eclm` 驱动的动态语义层

这样既能覆盖 ECL 自身稳定的语言骨架，也能根据不同游戏版本获得更准确的指令名和语义信息。

## 1. 为什么不能只靠 `eclm`

`eclm` 很重要，但它更适合描述：

- 指令名
- 参数名
- 参数类型
- 一部分内建语义

它不适合单独承担完整语言高亮，因为 ECL 里还有很多稳定但不依赖 map 的语法元素：

- 关键字和控制流
- 子程序定义和声明
- `global` 定义
- `anim` / `ecli` 头部块
- `#include` / `#eclmap` / `#set` 这类预处理指令
- 难度标签
- 栈访问表达式
- 变量和内建寄存器式符号

因此正确方案不是“选一种”，而是“固定规则定义骨架，`eclm` 补充版本化词表”。

## 2. 真实样本给出的约束

以 [`tools/st01.decl`](../tools/st01.decl) 为例，真实反编译结果至少包含这些语法形态：

- 文件头：`anim { ... }`、`ecli { ... }`
- 子程序声明和定义：`void MainBossSpell();`、`void GirlA01(var A)`
- 普通调用：`enemy_set_hitbox(...)`
- 异步调用：`@GirlA01_at() async;`
- 标签与跳转：`GirlA01_428:`、`goto GirlA01_448 @ 0;`
- 难度标签：`!HL67`、`!E67`、`!N67`、`!*`
- 局部变量与寄存器式变量：`var A;`、`$A = 100;`
- 内建随机符号：`%RAND_FLOAT_SIGNED`、`$RAND_INT`
- 栈访问：`[-1]`、`[-1.0f]`

这说明：

- 仅靠普通 C-like 规则会误判大量 ECL 语法
- 仅靠 `eclm` 又无法完整覆盖控制流和结构语法
- 高亮设计必须以真实反编译结果为基准，而不是凭空假设

## 3. 总体架构

推荐结构：

1. Rust 侧解析 `eclm`
2. Rust 输出稳定的结构化语义数据
3. 前端用固定规则注册 ECL 语言骨架
4. 前端再把 `eclm` 的指令词表和元数据叠加进去

这样分层有几个好处：

- `eclm` 解析属于领域逻辑，放 Rust 更稳
- 前端不必自己解析 map 文本
- Monaco 侧可以先实现基础高亮，再逐步升级到补全、hover 和 diagnostics

## 4. 第一层：固定语法层

这一层负责版本无关、语言本体稳定的语法元素。

建议先覆盖这些 token：

- 关键字：`if` `else` `switch` `while` `times` `break` `continue` `return` `goto` `global`
- 类型：`void` `int` `float` `var`
- 头部结构：`anim` `ecli`
- 预处理：`#include` `#eclmap` `#set` `#ifset` `#message` `#nowarn`
- 难度标签：`!E` `!N` `!H` `!L` `!*` 及其变体
- 变量：普通标识符、`$A` 风格变量、`$RAND_INT` 等内建变量
- 内建常量符号：`%RAND_FLOAT_SIGNED` 一类
- 数字：整数、浮点、带 `f` 后缀
- 字符串
- 注释
- 标签定义：`LabelName:`
- 标签跳转：`goto LabelName @ 0;`
- 栈访问：`[-1]`、`[-1.0f]`
- 子程序声明与定义
- 全局定义：`global NAME = value;`

实现上建议优先用 Monaco Monarch tokenizer 和 language configuration。

## 5. 第二层：`eclm` 动态语义层

这一层负责版本化、游戏相关、map 驱动的信息。

建议从 `eclm` 中提取：

- 指令名
- 指令 opcode
- 参数列表
- 参数类型
- 枚举或具名常量信息（如果 map 中可提取）
- 其他适合用于 hover / completion 的说明文本

前端第一阶段不必把这些全部做成完整语言服务，先做两件事就够：

1. 把指令名注入高亮词表
2. 为后续补全和 hover 预留结构

## 6. 推荐的数据结构

Rust 到前端建议传稳定结构，而不是原始 map 文本。

示例：

```ts
type EclInstructionParameter = {
  name: string
  type: string
}

type EclInstructionSpec = {
  opcode: number
  name: string
  params: EclInstructionParameter[]
}

type EclMapSemanticData = {
  version: string
  instructions: EclInstructionSpec[]
  enums?: Array<{ name: string; members: string[] }>
  builtins?: string[]
}
```

关键原则：

- 前端消费结构化数据
- 后端负责解析和归一化
- 不把 map 原文解析逻辑塞进 Vue 或 Monaco 组件

## 7. 前端建议模块划分

建议在前端拆成这些模块：

- `src/services/languages/ecl/monarch.ts`
  - 固定语法 tokenizer
- `src/services/languages/ecl/language-config.ts`
  - 注释、括号、自动闭合、folding 等
- `src/services/languages/ecl/semantic-tokens.ts`
  - 把 `eclm` 词表转换成高亮辅助数据
- `src/services/languages/ecl/completion-provider.ts`
  - 后续自动补全
- `src/services/languages/ecl/hover-provider.ts`
  - 后续悬浮说明
- `src/services/languages/ecl/model.ts`
  - 统一类型定义

后端可以对应拆成：

- `src-tauri/src/modules/ecl/map_parser.rs`
- `src-tauri/src/modules/ecl/map_service.rs`

## 8. 高亮分类建议

为了让界面更像一门完整语言，而不是“所有东西一个颜色”，建议至少区分：

- `keyword`
- `type`
- `number`
- `string`
- `comment`
- `preprocessor`
- `difficultyLabel`
- `headerDirective`
- `globalName`
- `subroutineName`
- `labelName`
- `instructionName`
- `builtinVariable`
- `builtinConstant`
- `stackAccess`

这套分类后续也能自然过渡到 hover、outline 和 diagnostics。

## 9. 分阶段实施顺序

### 第一阶段：稳定高亮

先不依赖 `eclm`，把真实 `.decl` 看起来像一门完整语言：

1. 注册 ECL 语言
2. 加固定 Monarch 规则
3. 正确高亮头部块、关键字、函数、变量、难度标签、栈访问
4. 先让 [`tools/st01.decl`](../tools/st01.decl) 看起来基本正确

### 第二阶段：接入 `eclm` 词表

1. Rust 解析 `eclm`
2. 前端获取当前版本 map 的结构化数据
3. 把指令名注入高亮词表
4. 把一部分内建符号或枚举名标成 builtin

### 第三阶段：升级到语言服务

1. 自动补全
2. 参数提示
3. hover 文档
4. 基于结构规则的 outline
5. 再把真实 `thecl` 诊断和静态规则一起接进 Monaco

## 10. 不建议一开始做的事

一开始不要：

- 假装已经有完整 ECL parser
- 让前端直接解析原始 `eclm`
- 把高亮、补全、诊断写成一团
- 只按教程抽象，不用真实 `.decl` 样本验证

更稳的方式是：

- 先把高亮做准
- 再逐步把 `eclm` 接成补充层
- 再往语言服务推进

## 11. 当前最推荐的下一步

基于这个计划，下一步最值得做的是：

1. 先为 Monaco 注册第一版 ECL Monarch 语法
2. 用 [`tools/st01.decl`](../tools/st01.decl) 做人工校验样本
3. 再设计 Rust 侧 `eclm` 解析接口

这样可以最快把编辑体验从“纯文本”提升到“有领域感的脚本编辑器”，同时不把架构写乱。

## 12. 当前 Marker 自动检测范围

目前编辑器内已经接入了第一版 ECL 静态 marker 检查。它属于“本地快速规则”，不依赖真实运行 `thecl`，用于在编辑时尽早给出明显问题提示。

当前会自动检测这些问题：

- 未定义的 `goto` 标签
  - 例如 `goto MissingLabel @ 0;`
  - 严重级别：`error`
- 当前文件中找不到定义的子程序调用
  - 例如 `@MissingSub()`
  - 严重级别：`warning`
  - 说明：当前阶段只检查“当前文件”；后续如果接入 `#include`、跨文件索引或项目符号表，这条规则需要升级
- 重复子程序定义
  - 例如同一个 `.decl` 里出现两个同名 `void Foo(...)`
  - 严重级别：`error`
- 重复标签定义
  - 例如同一个文件里两次定义 `LoopStart:`
  - 严重级别：`error`
- 重复全局定义
  - 例如同一个文件里两次定义 `global SOME_NAME`
  - 严重级别：`error`

当前不会自动检测，或还只是部分支持的内容：

- 跨文件子程序是否存在
- `#include` 引入后的符号可见性
- 指令参数个数/类型是否正确
- `thecl` 真实编译错误到 Monaco marker 的完整同步
- 更深层的控制流或语义错误

因此当前这套 marker 的定位应该明确为：

- 负责快速发现明显的编辑错误
- 减少一些无意义的 `thecl` 调用
- 为后续真实 diagnostics 和项目级语言服务打基础

而不是替代真正的 `thecl` 编译检查。
