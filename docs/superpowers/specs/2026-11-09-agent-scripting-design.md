# MSG／STD agent 编程支持设计

日期：2026-11-09。状态：已按用户确认路线实施，Sol 子代理完成实现与独立审阅。验证及未执行项见 [实施记录](../../../deps/agent-scripting-check-2026-11-09.md)。

## 1. 目标与边界

让通用编程 agent 能自然地阅读、编写、修改和检查 MSG／END／STD。agent 自行使用文件工具搜索、阅读和编辑源码；Studio 提供可信的领域知识、有效项目语义和真实工具链反馈。覆盖现有支持矩阵中的脚本，不限定为修改台词等少数预设任务。

本次交付三部分：可随项目使用的文档与 skill；IDE 和 headless 共用的轻量检查能力；补齐现有 MCP 的必要上下文与查询能力。

不建设完整 parser／AST 服务、LSP、符号索引、引用定位器、专用编辑 DSL、文档搜索服务或 agent loop。不模拟游戏执行，不把编译成功表述为游戏效果正确。不扩展 ECL／ANM／DAT 后端。维持现有 MSG、END、STD 版本集合；mission.msg、TH06–09 END、TH06NC 等原有排除项明确列出，不静默新增或承诺支持。

## 2. 方案与取舍

| 方案 | 取舍 | 结论 |
| --- | --- | --- |
| 完整源码分析服务，复用 truth AST | 能提供导航，但增加编译器接口、结构化导出和维护责任，agent 已能读文本与 grep | 经讨论撤回 |
| 只维护文档，agent 直接拼 truth 命令 | 开发少，但 IDE 和终端容易使用不同版本、map、编码及诊断格式 | 不足以满足目标 |
| 文档、现有语义查询、共享的 truth 调用与检查代码 | 复用编译器验证，CLI 无需 GUI；新增边界只容纳已有工具链职责 | 用户确认，已实施 |

共享库及 CLI 的组织随实施计划确认；进一步扩展架构仍按项目约定单独决策。

## 3. 已核对的资料

### 3.1 THBWiki

- 用户指定其本人编写的页面。实际入口是[《脚本对照表/其他》](https://thwiki.cc/脚本对照表/其他)，MSG 和 STD 均在此页；`脚本对照表/MSG` 当前不存在。
- 2026-11-09 通过 MediaWiki API 核对当前修订：`975551`，时间 `2026-09-15T16:12:33Z`。[固定修订链接](https://thwiki.cc/index.php?title=脚本对照表/其他&oldid=975551)。
- 当前修订已有 MSG 二进制格式、游戏内运行逻辑、关卡指令、结局／staff 指令和各作差异。网页检索返回的旧缓存 `896324` 缺少这些新内容，不用它代表当前文章。
- 这是游戏格式与行为资料；旧 thmsg／thstd 源码写法需要改写为当前 truth 写法，不能原样放进 agent 的可执行示例。

### 3.2 renkolab

只读资料根：`/data/sunyunbo/www/renkolab`；核对时 HEAD 为 `c1a0f1672143ee565f662dcacd2031c90a8579e3`，分支 `feat/stg-agent-proto`。该工作树有其他未提交工作，本任务不修改、切换、清理或提交它。

主要资料：

- `engine/msg/OVERVIEW.md`、`engine/msg/WIKI-DRAFT.md`。
- `engine/msg/th16/01-architecture-overview.md`、`02-msg-vm-opcodes.md`、`03-dialogue-lifecycle.md`、`04-ending-staff-msg-instruction-set.md`。
- `engine/msg/th18/01-stage-msg.md`、`02-ending-msg.md`。
- `engine/std/OVERVIEW.md`：主要是 TH06／TH06NC 的版本比较，不能代替现代 STD 的全面行为资料。

已有记录能补充对话等待、ECL 握手、分页与注音、ANM 中断协议，以及 END 的淡入淡出等行为。记录中的待验证项、版本限定及与社区资料的冲突必须保留。`WIKI-DRAFT.md` 中 TH10 起的概括不能推广到 TH06–09。

### 3.3 社区资料

- [ExpHP/thpages MSG 表](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/msg.ts)。
- [ExpHP/thpages END 表](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/end.ts)。
- [ExpHP/thpages STD 表](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/std.ts)。
- [Priw8 STD 指令表](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-ins-table.md)、[适用范围与来源](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-ins.md)、[STD 教程](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-tutorial.md)。

thpages 的版本映射、参数名与行为说明有价值，MSG 表当前列到 TH185，END 映射列到 TH18；不能作为 TH19／20 的完整依据。Priw8 的 STD 表明确以 TH13 起为目标，且声明参考 THBWiki；这些来源并非完全独立的多份验证。旧教程里的手算跳转偏移、旧文本格式和命令不沿用。

### 3.4 当前工具链

固定 zero318/truth `c78a541d1fb09fcefd448c83b097d4f9b815fce3` 与现有编码补丁。源码位于 `deps/.cache/truth`，重点核对 `doc/syntax.md`、`src/formats/{msg,std}.rs`、`src/core_mapfiles/` 和 `tests/integration/`。

实际可接受语法、meta 约束、签名和编码行为由固定工具及有效 map 决定。游戏行为依赖对应游戏的证据。不同来源冲突时分别记录“工具如何编码”和“游戏如何解释”，不擅自改 map 或 truth 补丁来掩盖冲突。

## 4. 文档与规则

规范资料维护在 `docs/agent/`。包括共用语法、MSG、END、STD、版本差异、工具使用、来源、可编译示例。详细指令签名从有效 map 生成；人维护行为解释和示例，不手抄第二套签名。

写作面向正在编程的人：先给明确用法、参数和可运行例子，再解释必要细节。研究过程、证据分级和历史疑问放在来源／研究笔记，不把取证口吻搬进教程与 skill。只有会改变代码写法、版本选择或结果判断的限制才在正文对应位置说明；未知指令集中列入参考附录。对已有依据的常用行为直接说明，不反复要求 agent 重新验证，也不强制每次写指令前都查询工具。目标是让 agent 顺畅使用现有生态，同时不编造未明行为。

现有中文说明 JSON 继续承担 UI／MCP 的短说明；详细 Markdown 承担参数含义、行为、限制及例子。不为两者建设通用内容数据库或 Markdown 解析服务。

项目辅助包继续使用 `.claude/skills/touhou-truth-scripts/`，保留历史路径。其他 agent 可通过项目规则中的明确文件路径读取，不声称它们会自动发现 Claude 的 skill 目录。资料离线可读，不依赖 renkolab 绝对路径、在线 Wiki 或 GitHub。

生成规则：skill、项目规则仅在缺失时创建；已有文件保留，只有用户显式选择更新才覆盖 skill。生成的 references、示例和项目上下文可以刷新，并明确标记为生成内容。项目根 `AGENTS.md` 已存在时不自动拼接或重写，返回一段可手动加入的入口说明。

2026-11-10 更新：辅助包改为直接导出人工语义手册，不再从 map 生成项目指令表；资料范围和版本／编码参考规则见 [人工手册设计](2026-11-10-agent-manuals-design.md)。本节原有自动指令表与示例导出安排由该设计替代。编译与查询仍使用实际 map。

## 5. 检查服务

推荐新增 `crates/script-check/`，提供普通 Rust 库和 `thtk-check` CLI。它只负责项目检查所需上下文、truth 进程调用、临时文件和诊断；不链接 truth Rust 库，不依赖 Tauri／WebView，不实现脚本 parser。

IDE 和 MCP 在进程内调用共享 Rust 库；CLI 调用同一个库。三条路径最终运行同一组外部 `trumsg`／`trustd`。无需给每次 IDE 自动检查额外启动一个 `thtk-check` 进程。

保持源码 UTF-8；MSG／END 二进制编码使用现有 Shift-JIS／GBK／UTF-8 选择。默认 map、项目 map、源码 pragma 的编译顺序不变。检查未保存文本时保留原目录语义，不改原文件、不保留产物。

原有 `ScriptResult` 和前端诊断字段尽量不变。CLI 额外返回本次实际上下文，原始编译器输出必须可读；不能因为结构化诊断没有匹配到位置就吞掉错误。

## 6. MCP

保留 `get_workspace_info`、`run_script`、`lookup_script_semantics`、`report_to_user`，不增加文件 CRUD、grep、符号导航或文档搜索工具。

- 工作区信息补充三类脚本的有效配置、检查器路径与生成资料位置。
- `run_script(check)` 和 IDE 缓冲区检查共用内核；允许可选源码文本，支持尚未落盘的新文件。
- 语义查询支持源码文本、尚未创建的源码路径、完整分页，以及参数对应的现有枚举；不再静默截取前 50 项。
- 查询返回签名和说明的适用边界：项目重新定义 opcode 签名时，不能继续把原版行为当作已验证说明。

## 7. 验收与结束条件

文档覆盖现有 19 个 MSG、12 个 END、22 个 STD 组合，并区分编解码支持、已知行为和未知行为。沿用迁移设计的 53 份 map 基线，不要求重新对每作进行逆向或游戏实跑。

代表性有效示例通过真实检查器；错误示例能返回准确位置。CLI 在 IDE 未启动、没有图形会话的环境可运行。IDE 原有自动检查保持可用。少量真实编程任务验证“读资料—改文本—检查—修正”的流程，不建设 agent 自动评测平台或浏览器交互测试。

文档不足以确定的游戏行为列入已知限制。若发现确实阻碍目标任务的工具能力缺口，单独说明影响与改动方案；不把所有未知项自动升级为新逆向或编译器开发任务。

实施顺序和具体接口见 [实施计划](../plans/2026-11-09-agent-scripting.md)。
