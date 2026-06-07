# ECL 语法速查（中文精简版）

这份文档把原始教程里最常用的 ECL 语法点压缩成一份速查表，方便后续做 IDE 提示、静态检查和文档悬浮。

## 1. 子程序与基本结构

ECL 以子程序为主要组织单位。

常见形式：

```c
void MainBossSpell();

void GirlA01(var A)
{
    enemy_set_hitbox(24.0f, 24.0f);
}
```

常见要点：

- 可以有前向声明
- 参数可以带类型或 `var`
- 子程序内部可声明局部变量
- `main` 通常具有特殊入口意义

## 2. 变量类别

教程里重点提到 4 类变量：

- 子程序参数
- 子程序局部变量
- 全局变量
- 特殊返回值/栈相关变量

注意点：

- 没有真正通用的字符串变量模型
- 某些类型转换需要显式写法，如 `_S`、`_f`
- 返回值往往通过特定临时位置或约定变量传递

## 3. 表达式与运算符

ECL 支持常见表达式和运算符，但不能把它完全当成 C。

重点：

- 存在常规算术和比较表达式
- 运算符优先级大体类似 C-like 语言
- `+`、`-` 在某些写法上对空格和歧义更敏感
- 某些结果依赖显式类型转换

因此做静态分析时，建议先做保守解析，不要过度假设完整 C 兼容。

## 4. 栈语法

教程单独强调了 ECL 的栈用法，这是普通脚本语言里不常见的一点。

常见形式：

```c
123;
[-1]
[-1.0f]
```

可理解为：

- 裸表达式可能把值压入栈
- 通过 `[-1]` 之类的写法读取栈顶附近内容
- 浮点和整数访问形式不同

这部分在反编译结果里可能显得比较奇怪，因此 IDE 后续：

- 不要把这些写法误判成数组
- 语法高亮和格式化都要单独照顾

## 5. 条件判断

ECL 支持常见条件语句：

- 三元表达式
- `if / else`
- `switch`
- 难度条件标签

特别是难度标签很有 ECL 特征：

- `!E`
- `!N`
- `!H`
- `!L`
- `!*`
- 单指令形式如 `!E:`
- 组合形式或难度分发表达

这意味着 IDE 后续需要：

- 把难度标签当一等语法元素处理
- 问题定位时不要把它们误报为非法标记

## 6. 循环

教程涵盖了几种常见循环：

- `while`
- `times`
- `break`
- `continue`

循环体也有自己的作用域和控制流语义。

对静态检查来说，第一阶段只需要：

- 识别循环结构边界
- 检查明显不平衡的大括号
- 允许 `break / continue` 在循环体内出现

## 7. 返回与调用

ECL 中调用和返回不能简单套入普通同步函数模型。

常见特征：

- `@SubName(...)` 调用语法
- 存在普通返回与更强制的退出语义
- 某些调用具有异步行为
- 主栈和额外调用栈的概念比较重要

这会影响 IDE 未来的大纲、跳转和调用关系图。

## 8. 全局定义

教程中有明确的全局定义写法：

```c
global NAME = value;
```

用途通常是：

- 替代魔法数字
- 统一管理 flag、aim、bullet type 等常量
- 提升脚本可读性

IDE 后续应优先支持：

- 全局常量索引
- 定义跳转
- 悬浮显示常量值

## 9. 预处理与组织指令

教程涉及的预处理式指令包括：

- `#eclmap`
- `#include`
- `#nowarn`
- `#message`
- `#set`
- `#ifset`

这些不是普通注释，而会影响构建和组织方式。

后续静态检查至少应识别：

- `#include` 路径是否存在
- `#eclmap` 是否可解析
- 条件块是否基本配对

## 10. 对 IDE 的语法支持建议

基于教程内容，第一阶段最值得支持的语法对象是：

1. `sub` / 前向声明 / `main`
2. 难度标签
3. `global` 定义
4. `#include` / `#eclmap` / `ecli`
5. 栈访问语法
6. `times / while / if / switch`

不建议一开始就宣称“完整 ECL 语义解析”。更稳的做法是先做：

- 词法级高亮
- 结构级索引
- 轻量静态规则
- 真实 `thecl` 结果兜底

## 11. 原始来源

主要来源页：

- [ecl-tutorial-source/pages/02-instructions-and-subroutines.md](./ecl-tutorial-source/pages/02-instructions-and-subroutines.md)
- [ecl-tutorial-source/pages/05-ecl-variables.md](./ecl-tutorial-source/pages/05-ecl-variables.md)
- [ecl-tutorial-source/pages/06-expressions-return-types.md](./ecl-tutorial-source/pages/06-expressions-return-types.md)
- [ecl-tutorial-source/pages/07-ecl-the-stack.md](./ecl-tutorial-source/pages/07-ecl-the-stack.md)
- [ecl-tutorial-source/pages/08-ecl-conditional-statements.md](./ecl-tutorial-source/pages/08-ecl-conditional-statements.md)
- [ecl-tutorial-source/pages/09-ecl-loops.md](./ecl-tutorial-source/pages/09-ecl-loops.md)
- [ecl-tutorial-source/pages/11-ecl-global-definitions.md](./ecl-tutorial-source/pages/11-ecl-global-definitions.md)
- [ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md](./ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md)
