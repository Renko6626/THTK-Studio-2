# 工具链生态与 map 边界

更新：2026-11-09。本文替代 2026-08-05 的“thtk 唯一后端、truth 仅作参考”决策。新路线已实现，验证状态见依赖目录中的集成记录。

## 后端分工

MSG/END 使用 zero318/truth 的 `trumsg`，STD 使用 `trustd`，保存 truth 原生源码；ECL/ANM/DAT 保留 thecl/thanm/thdat。truth 随应用分发。旧 `.dmsg/.dstd` 直接废弃，不保留旧后端、翻译方言或转换器；用户文件不自动删除。GBK 编码支持是切换前置条件。

truth 是脚本编译器，目标不局限于 MSG/STD。其 ANM 脚本语言也有价值；本次暂不迁移 ANM 是范围选择。ECL 仍存在返回类型、inline 等能力差距，保留 thecl。

## 依赖基线

- truth：[zero318/truth](https://github.com/zero318/truth)，跟踪分支 `format_improvements_test`，固定基线 `c78a541d1fb09fcefd448c83b097d4f9b815fce3`。
- maps：[zero318/TouhouMaps](https://github.com/zero318/TouhouMaps)，基线 `0673f4221f5e975842162e07cd7b765fc7570ebd`。
- 产物包含 MSG/END 编码补丁；二进制与补丁 SHA-256 随包记录，不能仅靠上游版本字符串识别。
- `trumsg`、`trustd` 是启动包装程序，需要同目录 `truth-core`。不能仅随包复制两个包装程序。
- 分支名用于追踪上游，构建使用固定提交；不能安装默认 main 或以 `0.5.1-pre1` 版本字符串判定分支能力。

来源记录见 [依赖清单](../deps/manifest.source.json)，交付约定见 [deps/README.md](../deps/README.md)。

## 统一的是什么

统一目标作品、脚本种类、名称、参数说明、枚举和项目选择结果。后端 map 语法保留各自约定，不要求一份文件直接供所有编译器使用。

| 格式 | 编译器输入 | Studio 需要负责的部分 |
| --- | --- | --- |
| ECL | thtk 兼容 eclmap；使用适合当前后端的 `_thtk` 变体 | 继续现有配置；普通 map 与 `_thtk` 不是可互换文件 |
| ANM | thanm 支持的名称、变量映射 | 未来接入时从丰富资料生成兼容视图；本次不实现 |
| MSG/END/STD | truth 扩展 map 与当前版本内置定义 | 正确解析属性、枚举、padding、intrinsic，构建与提示使用一致定义 |
| DAT | 无指令 map | 容器操作 |

truth 的参数签名包含 `S(enum="...")`、`b---`、字符串编码属性等；不能逐字符当作用户参数。MSG 的 `m` 也不能按 ECL 子程序引用解释。有效定义包括 truth 内置版本规则和外部覆盖；不能只读取名字再沿用 thmsg 的签名表。

zero318 的 `any.msgm` / `any.stdm` 留空了若干中间作品，不能直接作为产品完整支持列表。用固定来源的 truth 内置定义、社区表和本项目研究补齐目标版本，按脚本种类记录覆盖。END 与关卡 MSG 的 opcode 空间分开；mission.msg 是另一种格式，本次不扩大支持范围。

ECL `_thtk` map 包含标为 ZECL 的扩展，不可把扩展运行时能力自动当作原作能力。此项记录为后续 ECL 工作，不借本次迁移重构。

## 源语言约定

truth 源码直接保留 `meta`、`script`、时间标签、标签/goto、可用的结构化控制流、常量和枚举。编译器决定各脚本种类允许的语言子集；共享解析器不等于 MSG 支持 ECL 的运行时变量。

不把 truth 输出翻译回旧 Studio 方言。使用 `.msgspec/.endspec/.stdspec`；具体字段与覆盖规则见迁移设计。

## 维护边界

社区资料与自有逆向成果都可以作为正式语义来源。维护者处理版本和依据，用户得到简明定义。只因编解码器不支持具体目标操作时才处理对应缺口；目前明确需要的下游扩展是 MSG/END 编码选择，不接管整套 truth 或 thtk 开发。

旧文档中的“th14–20 共用 thstd 格式表，因此全部指令语义相同”“同族 map 均可直接互换”等推断不再作为实现依据。以选定 truth 分支的有效定义和目标游戏事实为准。
