# ECL 教程整理：入门与工作流

这份文档基于 `docs/ecl-tutorial-source` 的原始教程整理，目标不是逐页翻译，而是提炼出对 THTK-Studio 开发和使用最有价值的 ECL 基础概念。

## 1. 工具链与基本流程

ECL 的常见工作流不是“直接手写二进制”，而是：

1. 用 `thecl` 把 `.ecl` 反编译成 `.decl`
2. 在 `.decl` 中编写或修改脚本
3. 再用 `thecl` 编译回 `.ecl`
4. 用 `eclmap` 补充指令名、参数名和一定程度的可读性

`thecl` 的核心模式：

- `thecl -d <version> input.ecl output.decl`：反编译
- `thecl -c <version> input.decl output.ecl`：编译
- `thecl -h <version> input.ecl output.h`：生成头文件，便于多文件组织

版本参数是纯数字，例如 `20`，不是 `th20`。

## 2. 地图文件（eclmap）的作用

`eclmap` 不是可选装饰，而是实际开发里非常重要的一层：

- 提供指令名和参数语义
- 改善反编译结果的可读性
- 让编译器更好地识别脚本结构

如果没有 map，脚本仍可能工作，但可读性和可维护性会明显下降。

对 IDE 来说，这意味着：

- 构建弹窗应支持显式配置 map 路径
- 工具链测试里应覆盖“缺少 map”的场景
- 未来的语法提示、跳转、文档悬浮都应优先考虑 map 参与后的结果

## 3. ECL 的运行模型

ECL 不是普通的顺序脚本，更接近“敌机实例上的解释执行逻辑”。

关键点：

- 每个敌机实例有自己的执行上下文
- 指令会按帧推进，而不是一口气执行完整个文件
- 子程序（subroutine）是组织逻辑的主要单位
- `main` 具有特殊意义，常作为敌机入口或主控制逻辑

因此，IDE 后续设计不能把 ECL 当成普通 C-like 文本语言简单处理。至少要意识到：

- 子程序与实体生命周期有关
- 某些调用是异步语义
- 一部分行为依赖引擎帧推进，而不是纯语法结构

## 4. 指令、子程序与调用

教程中最重要的模型之一是“指令”和“子程序”的分工：

- 指令：直接作用于引擎对象或状态
- 子程序：把若干指令组织成可复用逻辑

常见特征：

- `@SubName(...)` 风格的调用
- 有参数、局部变量和返回语义
- 存在普通返回和更强的“直接退出”语义
- 某些调用会建立新的调用栈上下文

对 IDE 来说，这意味着后续做：

- 符号索引时，`sub` 应是一级对象
- 大纲视图应优先展示子程序
- 跳转定义、查找引用，应首先围绕子程序和全局定义展开

## 5. `main` 的特殊地位

教程多次体现出 `main` 不只是一个普通函数：

- 它通常是脚本或敌机逻辑的起点
- 常参与敌机生成、初始化、无敌/碰撞等核心流程
- 在部分组织方式里，`main` 再调度其他子程序

因此，IDE 后续可以考虑：

- 在大纲中对 `main` 做特殊标识
- 在二进制脚本说明页里，把“主入口”作为脚本结构摘要的一部分
- 构建结果里优先显示 `main` 附近的错误

## 6. 敌机生成与动画资源绑定

ECL 不只写弹幕逻辑，还承担敌机生成与资源驱动职责。

教程里的基础模式包括：

- 生成敌机实例
- 绑定 ANM 资源和脚本
- 设置 hitbox / collision / movement 等基础属性
- 通过不同指令控制不同类型的对象行为

这对 IDE 的启发是：

- ECL 与 ANM、资源路径之间存在天然关联
- 未来项目视图和资源预览不应只把 `.decl` 当孤立文本
- 可以考虑做“脚本引用的 ANM / ecli / include 资源”提示

## 7. 多文件组织：`#include`、`ecli` 与头文件

教程明确说明了 ECL 不是只能单文件写到底。

几种常见组织方式：

- `#include`：直接把内容并进当前编译单元
- `ecli`：声明外部脚本依赖/入口集合
- `thecl -h`：从脚本生成头文件，辅助多文件拆分

还涉及：

- include guard
- 条件编译式开关，如 `#set / #ifset`
- `#message`、`#nowarn` 这类构建期辅助指令

对 IDE 来说，这是后续很重要的方向：

- 构建前检查要识别 `#include` 和 `ecli`
- 项目索引要区分“文本包含”和“脚本入口依赖”
- 头文件生成应作为正式工具链操作保留

## 8. MERLIN 库的意义

教程末尾提到 MERLIN 库，重点不是“多一个库”，而是它代表一种更现代、更可维护的写法：

- 用具名常量和封装替代大量魔法数字
- 提升可读性
- 降低脚本维护成本

这对 IDE 的意义是：

- 全局定义、常量名、库符号应该是后续索引重点
- 如果未来做悬浮提示或自动补全，优先支持这些可读性增强层

## 9. 对 THTK-Studio 当前开发的直接结论

结合这套教程，当前 IDE 开发上最值得继续做的是：

1. 把 `thecl` 的真实构建结果更稳地接入输出、问题和 Monaco diagnostics
2. 优先围绕 `sub`、`main`、`global`、`#include`、`ecli` 建索引和大纲
3. 在工作区视图里逐步体现“脚本与资源关联”，而不是只做文本编辑
4. 静态检查先做轻量规则，不要假装已经理解了全部 ECL 语义

## 10. 原始来源

原始教程页保存在：

- [ecl-tutorial-source/README.md](./ecl-tutorial-source/README.md)
- [ecl-tutorial-source/pages/01-ecl-preparing.md](./ecl-tutorial-source/pages/01-ecl-preparing.md)
- [ecl-tutorial-source/pages/02-instructions-and-subroutines.md](./ecl-tutorial-source/pages/02-instructions-and-subroutines.md)
- [ecl-tutorial-source/pages/03-basic-enemy-spawning.md](./ecl-tutorial-source/pages/03-basic-enemy-spawning.md)
- [ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md](./ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md)
- [ecl-tutorial-source/pages/13-ecl-extras-1-the-merlin-library.md](./ecl-tutorial-source/pages/13-ecl-extras-1-the-merlin-library.md)
