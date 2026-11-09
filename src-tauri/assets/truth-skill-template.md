---
name: touhou-truth-scripts
description: 修改东方 MSG 对话、END 结局或 STD 背景脚本时使用，包含 truth 原生语法、项目 map 查询、编译检查和游戏文本编码。
---

# MSG、END、STD 编辑

使用当前项目的游戏版本和 map。源码格式为 UTF-8 的 truth 原生文本：关卡对话 `.msgspec`，结局/staff `.endspec`，背景 `.stdspec`。旧 `.dmsg/.dstd` 已废弃。

1. `get_workspace_info` 查看目标项目和工具状态。
2. 二进制用 `run_script`，operation=decompile，kind=msg/ending/std，得到原生源码。
3. 写指令前用 `lookup_script_semantics`，传源码路径与 kind，查询名字、参数、枚举。项目覆盖和源码 pragma 都影响结果。
4. 在 `meta`、`script`、时间标签、常量、标签/goto 及当前脚本可用的结构化控制流中局部修改。不要给 MSG 编造 ECL 的运行时变量或函数。
5. 保存后用 `run_script`，operation=check。通过后用 operation=compile 生成二进制。
6. 汇报修改与检查结果，运行效果是否已经在游戏中确认另行说明。

MSG/END 的 encoding 是二进制游戏文本编码：默认 shift-jis；已适配的汉化游戏可用 gbk。解包和打包可以分别指定。源码始终 UTF-8，不要整体转码源文件。

`#pragma mapfile` 使用原生 truth 规则，Studio 的工作目录为源码所在目录。保留已生成的持久 map 引用。STD 使用符号标签，插入指令后由编译器重算偏移，不手算旧 thstd 的字节偏移。

使用 IDE 的检查工具，不直接绕过项目设置调用 thmsg/thstd。编辑器时序面板只显示源码结构，循环或分支中的时间未求值不表示游戏不会执行。
