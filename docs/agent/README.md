# MSG、END、STD 编程手册

这里讲怎样用 Studio 当前的 truth 工具链编写脚本。源文件统一保存为 UTF-8：关卡对话用 `.msgspec`，结局和 staff 用 `.endspec`，背景用 `.stdspec`。编译后的关卡对话和结局都可以叫 `.msg`，两者使用独立指令集。

先选自己的任务：

| 要改什么 | 从哪里开始 |
| --- | --- |
| 台词、立绘、气泡、boss 登场时机 | [MSG](msg.md) |
| 结局文字、CG、staff roll | [END](ending.md) |
| 背景对象、相机、雾、循环 | [STD](std.md) |
| truth 写法、时间标签、常量、跳转 | [共用语法](truth-syntax.md) |
| 把一个版本的脚本移到另一作 | [版本差异](version-differences.md)、[支持矩阵](support-matrix.md) |
| 编译、反编译、Studio 与 agent 检查 | [工具使用](tooling.md) |
| 查出处或尚未解释的指令 | [来源与附录](sources.md) |

## 从一个完整例子开始

[examples/manifest.json](examples/manifest.json) 为每个例子指定了格式、游戏版本和编码。以下八个例子已使用随包 truth 和对应版本的默认 map 编译通过：

| 例子 | 内容 |
| --- | --- |
| [TH06 对话](examples/th06-dialogue.msgspec) | 入口、ANM 立绘、指定两行文本、等待、ECL 放行 |
| [TH10 对话](examples/th10-dialogue.msgspec) | 旧现代指令编号、文本框、等待和结束 |
| [TH18 对话](examples/th18-dialogue.msgspec) | 立绘槽、说话者、表情、气泡和两行分页 |
| [TH18 结局](examples/th18-ending.endspec) | ANM 源与图片槽、音乐、文字翻页和黑幕 |
| [TH20 结局](examples/th20-ending.endspec) | 双颜色参数和注音 |
| [TH06 背景](examples/th06-background.stdspec) | 对象、实例、相机关键帧与末尾哨兵 |
| [TH08 背景](examples/th08-background.stdspec) | 早期 meta、旧插值写法与符号跳转 |
| [TH18 背景](examples/th18-background.stdspec) | ANM 路径、现代插值、循环与 ECL 中断 |

例子中的 ANM 脚本号、路径和音乐名展示资源引用的写法。将它们接入项目时，换成项目实际资源；这里没有附带游戏资源。编译验收确认语法、签名和文件结构，画面、文字排版和与 ECL 的时序仍由目标游戏决定。这八份例子尚未做游戏效果验收。

## 阅读已有工程

先保留反编译所得的 `meta`、资源引用和入口号，在需要的位置修改指令。正文使用默认 map 的短名，如 `textAdd`、`posTime`；`text_dialogue`、`camera_position_interp` 等长名也可能出现在旧文件中。它们是 map 中同一 opcode 的别名。项目 map 可以重新命名或重新定义签名，迁移时要同时带上 map。

详细签名以有效 map 为准；正文负责说明常用参数和行为。Studio 源码仓库的默认 map 位于 `src-tauri/assets/maps/truth/defaults/`；生成到 mod 项目的辅助包会在项目上下文中列出实际 map 路径。默认集合覆盖 19 个 MSG、12 个 END、22 个 STD profile。`mission.msg`、TH06–09 旧结局及 TH06NC 不在这份手册的支持集合中。
