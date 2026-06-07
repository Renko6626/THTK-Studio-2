# ECL 常见模式与工程组织（中文精简版）

这份文档侧重教程里更“项目化”的部分：敌机生成、弹幕管理器、坐标与移动、多文件组织，以及这些内容对 IDE 的启发。

## 1. 敌机生成的基本模式

ECL 里的敌机逻辑通常不是单条指令堆砌，而是围绕“生成对象并初始化行为”展开。

常见步骤：

1. 生成敌机
2. 设置碰撞、判定、生命、标志位等属性
3. 绑定动画资源
4. 进入子程序或状态逻辑

`main` 往往承担这类入口职责，因此项目浏览时：

- `main` 可以视为脚本入口摘要
- 其它子程序更像被调度的行为模块

## 2. ANM 资源与脚本绑定

教程说明了 ECL 与 ANM 之间的天然关联：

- `anim { ... }` 用于声明动画资源
- 不同槽位和脚本号决定展示与行为表现
- 部分对象生成指令会同时依赖脚本逻辑和动画资源

这意味着 IDE 后续不应该把 `.decl` 和 `.anm` 视为完全独立的世界。

更合理的长期方向是：

- 在脚本视图中识别引用的 ANM 资源
- 在资源视图中回看被哪些脚本使用
- 至少支持路径跳转或引用提示

## 3. 弹幕管理器（Bullet Managers）

教程把射弹逻辑归纳为“弹幕管理器”概念，这很重要。

它的意义在于：

- 发射行为通常不是一个简单 `shoot()` 就结束
- 不同模式决定扇形、环形、随机散布等发射结构
- 弹幕参数常常成组出现，并受到管理器状态影响

教程中提到的模式包括：

- 扇形模式
- 环形模式
- 多种随机模式

对 IDE 的启发：

- 将来如果做帮助文档或悬浮提示，弹幕相关指令应展示“模式含义”
- 参数提示不应只显示原始数字，更应尽量解释它们对应的发射行为

## 4. 坐标、位置与移动

教程给出了比较明确的屏幕/战场坐标模型：

- 原点和屏幕几何并不是普通 UI 坐标
- `x` 大致在 `-192..192`
- `y` 大致在 `0..448`
- 顶部中间附近可视为主要参考原点

此外还强调：

- 有绝对位置和相对位置
- 有不同移动和插值方式
- 某些边界或速度效果与引擎约束有关

这对 IDE 的后续价值很高：

- 可以做位置参数悬浮解释
- 更进一步甚至可以做简单轨迹/位置预览
- 但在没有真实语义支撑前，不要伪造“正确预览”

## 5. 全局定义与可读性

教程建议用 `global NAME = value;` 管理常量，而不是到处写魔法数字。

典型用途：

- 难度相关参数
- 子弹类型
- 瞄准和行为标记
- 公共配置值

这对 IDE 来说非常适合做第一批“符号级支持”：

- 全局常量索引
- 跳转到定义
- 悬浮显示值
- 查找常量引用

## 6. 多文件工程组织

教程给出的多文件组织思路很接近真正工程开发：

- `#include`：文本级包含
- `ecli`：外部脚本入口声明
- `thecl -h`：生成头文件
- include guard：防止重复包含
- `#set / #ifset`：条件化开关

实际区别可以简化理解为：

- `#include` 更像“把文件内容并进来”
- `ecli` 更像“告诉编译器还存在别的脚本入口”

因此 IDE 后续应区分两种关系：

- 文本包含关系
- 脚本依赖/入口关系

不要只做一个“文件引用”树就算完事。

## 7. MERLIN 库与工程化写法

MERLIN 库在教程中的价值主要有两点：

- 它代表了更具名、更工程化的写法
- 它说明社区实践已经不满足于裸写 magic numbers

对 THTK-Studio 的启发：

- IDE 应支持库式写法，而不是假设所有脚本都只有原始指令
- 未来做补全、定义跳转、符号索引时，应优先兼容这类常量与封装层

## 8. 对 IDE 的直接落地方向

基于这些模式，后续最适合做的不是“花哨 UI”，而是这些真正有价值的能力：

1. 大纲优先展示 `main`、子程序、`global` 定义
2. 识别 `#include`、`ecli`、`anim` 等关键组织节点
3. 在问题面板和编辑器里保留脚本与资源路径上下文
4. 给构建前校验补上 include/map/资源路径存在性检查
5. 为将来的 ANM、MSG、STD 接入预留跨文件引用模型

## 9. 原始来源

主要来源页：

- [ecl-tutorial-source/pages/03-basic-enemy-spawning.md](./ecl-tutorial-source/pages/03-basic-enemy-spawning.md)
- [ecl-tutorial-source/pages/04-bullet-managers-shooting-danmaku.md](./ecl-tutorial-source/pages/04-bullet-managers-shooting-danmaku.md)
- [ecl-tutorial-source/pages/10-ecl-enemy-positioning.md](./ecl-tutorial-source/pages/10-ecl-enemy-positioning.md)
- [ecl-tutorial-source/pages/11-ecl-global-definitions.md](./ecl-tutorial-source/pages/11-ecl-global-definitions.md)
- [ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md](./ecl-tutorial-source/pages/12-ecl-combining-multiple-ecl-files.md)
- [ecl-tutorial-source/pages/13-ecl-extras-1-the-merlin-library.md](./ecl-tutorial-source/pages/13-ecl-extras-1-the-merlin-library.md)
