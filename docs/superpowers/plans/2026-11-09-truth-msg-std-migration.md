# MSG/END/STD truth 迁移 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Only use superpowers:subagent-driven-development if the user selects delegation. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** MSG/END/STD 使用随包 truth 原生源码流程，保留 GBK，移除旧 thmsg/thstd 方言与运行通道。

**Architecture:** Rust 外部进程封装与格式级服务，前端消费结构化结果。truth 不链接进 Tauri；map 已确定由 Studio 自行解析管理，Task 0 完成数据与接口细化。ECL/ANM/DAT 保留现有路线。

**Tech Stack:** Tauri v2、Rust、Vue 3、TypeScript、Monaco；zero318/truth 固定提交和最小编码补丁，TouhouMaps 固定快照。

**Spec:** [迁移设计](../specs/2026-11-09-truth-msg-std-migration-design.md)、[编码设计](../specs/2026-11-09-truth-msg-encoding-design.md)。

**状态：** 代码切换已实施，固定 Linux/Windows truth 工具已构建；NSIS 安装包已生成并核对资源清单。Windows 实机安装、编辑器交互和游戏内效果仍待人工验收。

## Global Constraints

- truth 固定版本随 Studio 安装包分发，承担对应平台构建与分发。
- 旧 `.dmsg/.dstd` 支持直接废弃，不做自动转换器、不保留 thmsg/thstd 回退。
- GBK 汉化能力必须保留，先单独设计 truth 的最小编码扩展，再完成切换。
- 废弃旧格式指移除产品支持和实现，不是自动删除用户磁盘上的文件。
- ECL/ANM/DAT 后端保持原路线；不新增 mission.msg、完整 LSP、模拟器或依赖更新服务。
- UI 默认截图或人工验收，不自动执行点击流程。新增测试只覆盖具体关键行为，优先复用已有用例。
- 工具与地图来源以 deps/manifest.source.json 为准；不能把 truth 默认 main 或原版 0.5.2 替代选定分支。

## Review Focus

1. GBK 文本经过真实 MSG/END 字符串编码，不是整体转码源文件：Task 1。
2. END 与 stage 文件改名后仍选对模式，opcode 不串表：Task 3/4。
3. enum、padding、字符串属性与多 map 覆盖所得提示和编译一致：Task 2。
4. truth 循环/标签不会被旧时间线平铺成确定的一遍执行：Task 4。
5. 干净安装没有 PATH 上的工具，也能找到同目录 truth-core：Task 1/6。

## 任务依赖

Task 0 → Task 1 与 Task 2 → Task 3 → Task 4/5 → Task 6。每阶段保持现有工作区可构建；最终切换前不发布“已支持 truth”的声明。Task 3–6 合并为完整迁移交付，不把中间双通道状态作为正式产品。

### Task 0：确认设计、核对固定工具与 map 取数方案

**Files:**
- Update: 本计划、两份设计、deps/README.md、deps/manifest.source.json。
- Evidence only: 临时目录中的 truth/maps 固定检出；不把游戏资产提交到仓库。

**Inputs:** 用户确认的工具分工、分发/废弃/编码选择；truth 基线 c78a541d1fb09fcefd448c83b097d4f9b815fce3。
**Outputs:** 审阅通过的配置/源码约定、真实命令样例、map 服务选型及后续任务精确接口。

- [ ] 用户审阅迁移与编码设计，确认推荐的扩展名、名称保留、资源布局和配置字段。
- [x] 在独立临时检出原样构建基线：`cargo +1.85.1 build --release --locked --bin truth-core --bin trumsg --bin trustd` 成功；1.98.1/1.92.0 的旧语法兼容失败已定位，未升级依赖树或修改上游源码。
- [ ] 运行 trumsg/trustd 的 help 与版本命令，核对 ending、map、输出路径和 stderr 形状；验证缺 truth-core 的行为。
- [ ] 选项目现有样本或授权本机样本：一份 STD、一份 stage MSG、一份 END。记录目标版本、地图、解包/回编命令及关键内容，不宣称未运行的结果。
- [x] 确认 map 取数路线：用户选择 Studio 自己处理 map，不给 truth 加查询命令，不新增额外分支或 truth API 辅助程序。
- [ ] 细化原生 map 解析、版本基线和源码 pragma 路径规则；用 enum/padding 签名、无参指令、一次覆盖核对。说明数据来源，避免在 Studio 重建编译器类型检查。
- [ ] 将选择写回设计和 Task 2/4/5，补齐跨任务的确切类型/函数。不创建通用插件平台，也不在 Tauri 主程序直接加入 truth git 依赖。

**完成证据：** 实际工具命令记录、三类样本状态、已细化的 Studio map 接口；map 查询路径不修改 truth。

已完成的基线证据见 [核对记录](../../../deps/truth-baseline-check-2026-11-09.md)：真实 help/version 已运行，TH18 stage/ending/STD 原样往返字节一致。局部编辑与其他目标版本尚未验证；不勾选整个 Task 0。

### Task 1：GBK 补丁与随包工具构建

**Files:**
- Create: `deps/truth/patches/0001-msg-binary-encoding.patch`（实现后生成）。
- Create: `scripts/build-truth.mjs`（固定源码、应用补丁、构建与复制产物）。
- Modify: `deps/manifest.source.json`、`deps/README.md`、`.gitignore`。
- Upstream checkout ownership: `src/cli_def.rs`、`src/context/` 中必要上下文、`src/llir/lower.rs`、`src/llir/raise/early.rs` 及对应现有集成用例。

**Inputs:** 编码设计确认；固定基线。
**Outputs:** 含编码选项的 truth 工具组、可复现补丁、平台来源/校验记录。

- [ ] 按编码设计，在 truth 现有测试体系中补一组真实 GBK/Shift-JIS 行为断言，首先确认缺失功能导致失败；不要在 Studio 复制编解码算法。
- [ ] 实现 trumsg `--encoding` 的请求级传递，仅影响 MSG/END；默认保持 Shift-JIS，显式支持 GBK 和既有 UTF-8 配置。
- [ ] 验证两个方向和 stage/ending 模式，不可编码字符必须报错；运行相关字符串/MSG/END 回归，记录精确命令。
- [ ] 生成可应用到固定提交的补丁；在干净检出复现构建一次。产物记录基线、补丁、目标平台、hash 与许可。
- [ ] 构建脚本按实际平台生成同目录 truth-core/trumsg/trustd，平台产物放被忽略的 staging 目录；未构建的平台不填假 hash。

**不包含：** 自动在线安装、上游提交、全平台 CI。Windows 交叉构建命令须从实际可运行的构建流程确认；不假定 Tauri 的交叉参数自动适用于独立 truth 工程。

### Task 2：统一 map 与有效语义

**Files:**
- Create: `src-tauri/assets/maps/truth/` 中版本入口、整理后的 MSG/END/STD 表和来源记录。
- Modify: `src-tauri/src/common/game_map.rs`、`src-tauri/src/modules/msg/map_parser.rs`、`src-tauri/src/modules/thstd/map_parser.rs`。
- Create: `src-tauri/src/modules/truth/maps.rs`，Studio 自行解析原生 map；不直接破坏 ECL 的 map_parser。
- Types: `src/types/` 内新增领域语义类型，与 `src-tauri/src/wire_format_tests.rs` 对齐。

**Inputs:** Task 0 确认的语义获取方案、truth 内置版本定义、固定 map 源。
**Outputs:** 给指定 game/kind/maps/source pragma 解析的最终指令表；包含参数、枚举、文档、来源和 raw signature。

**接口细化（已接入）：**
- `load_truth_maps(game: u32, kind: TruthScriptKind, paths: &[PathBuf]) -> Result<TruthDefinitions, String>`：读取显式有序列表；支持 gamemap 相对路径、名称/签名独立覆盖、枚举与 intrinsic；不隐式加载内置表或源码 pragma。已通过 `get_truth_map_definitions` Tauri 命令和前端 typed API 暴露。
- `parse_truth_signature(signature: &str) -> Result<Vec<TruthParameter>, String>`：扫描类型字符和括号属性，padding 不出现在用户参数中；字符串属性保留在其参数上，不拆成多个参数。
- `resolve_truth_definitions(game: u32, kind: TruthScriptKind, map_paths: &[PathBuf]) -> Result<TruthDefinitions, String>`：按已解析的有序路径加载版本资料及覆盖；源码 pragma 的路径列表由同一请求准备过程追加，调用方不各自解释覆盖顺序。
- `TruthDefinitions` 包含 game、kind、instruction 列表、枚举及实际来源；`TruthParameter` 保留类型、可选枚举名、编码属性。中文参数名/单位/说明按 game/kind/opcode 附加，独立于编码规则。
- 解析实现先覆盖选定 MSG/END/STD map 实际用到的语法，编译器负责完整语法与编码校验；不开发第二套类型推导或二进制编码器。


- [ ] 按现有支持作品整理覆盖矩阵，分别列 stage/ending/std；排除不存在该脚本种类的作品，补齐社区 gamemap 空缺，不能偷换成“全部版本同一表”。
- [ ] 同一个实际指令以 opcode/游戏/脚本种类关联说明，按设计保留当前公开名称；不复制旧 thmsg 签名覆盖来定义 truth。
- [x] 覆盖具体行为：`SU(enum="InterpMode")fff` 得到 5 个参数，`b---` 不把 padding 当参数，字符串属性不增加参数数量，零参数与未知签名可区分。
- [ ] 验证后加载覆盖与源码 pragma 的优先级及路径基准；结果与同配置 truth 实际调用吻合。每种独立失败风险选一个用例，不穷举属性组合。
- [ ] 记录来源与本项目修订；独立研究资料转为用户参考，不在正常提示中复制研究过程。

**2026-11-09 实施进度：** 显式 map 读取模块、签名解析、Tauri/TypeScript 契约和 27 份原始 map 来源快照已落地；尚未配置产品默认版本基线，也未注入 truth 内置签名或源码 pragma。5 个新行为用例经历失败→通过，补 1 个项目要求的 wire contract。完整 Rust 测试 277 passed / 2 ignored，typecheck 通过；真实 truth 对 5 参数相机调用编译成功，对 4 参数调用报 expects 5 / got 4。此记录不代表 Task 2 或整体迁移完成。

### Task 3：配置、原生编译通道与诊断

**Files:**
- Modify: `src-tauri/src/config.rs`、`common/project_config.rs`、`common/toolchain.rs`、`common/dependencies.rs`、`common/game_version.rs`。
- Create: `src-tauri/src/modules/truth/{mod,compiler,diagnostics}.rs`，在 modules/mod.rs 注册。
- Modify: `modules/msg/{commands,compiler,mod}.rs`、`modules/thstd/{commands,compiler}.rs`、`src-tauri/src/main.rs`。
- Modify: `src/types/{project,toolchain,dependencies}.ts`、`src/api/modules/compiler.ts` 和 wire_format_tests。

**Inputs:** 已构建工具组、有效语义/配置选择。
**Outputs:** 原有领域 commands 改为调用 trumsg/trustd，返回结构化结果；新设置兼容读取旧项目 JSON。

- [ ] 按设计增加 truth_dir / truthDir / scriptMaps，保留 ECL mapPaths；复用原配置校验和保存，不引入新状态数据库。
- [ ] 更新工具描述及版本探测；用 trumsg/trustd ID，旧 override 不充当新工具路径。覆盖缺省字段、显式路径和随包路径优先级。
- [ ] 编译与反编译直接使用 truth 源码；实现新扩展名推导、ending 模式、每次操作编码和明确工作目录/环境。
- [ ] 进程放后台；捕获实际退出码、stdout/stderr。保留原始错误输出，解析真实 codespan 主位置；不假设支持 JSON stderr。
- [ ] 对真实样本做“解包→局部编辑→回编”检查，确认 stage/ending 模式及 GBK；相关单测仅覆盖路径/参数/诊断等独立逻辑。

### Task 4：工作台与语言支持切换

**Files:**
- Modify: `src/services/toolchains/{registry,msgMetadata,readiness,gameVersions}.ts`。
- Modify: `src/components/Dialogs/{ToolchainSettingsDialog,ProjectSettingsDialog}.vue`、`forms/MsgBuildForm.vue`、构建表单相关组件。
- Modify: `src/composables/useToolchainActions.ts`、`src/components/Common/MenuBar.vue`、`src/stores/editor.ts`、`src/components/Editor/BinaryScriptView.vue`。
- Modify: `src/services/languages/dispatch.ts`、`languages/msg/`、`languages/std/`、`src/components/Sidebar/TimelinePanel.vue`。
- Create: `src-tauri/src/modules/truth/analysis.rs`（仅按确认的分析边界实现）；Monaco providers 消费其结果。
- Reuse tests: `tests/unit/{msgScriptKind,dialectHighlighting,stdRegister,stdJumpNavigation,stdTimeline,editorModels,toolchainReadiness}.spec.ts`。

**Outputs:** 新源码能打开、高亮、补全、查参数、定位编译错误和标签；MSG/END/STD 构建不露出旧格式选项。

- [ ] 全部文件识别入口使用 `.msgspec/.endspec/.stdspec`；END 源文件即使改名也按扩展名保持模式，二进制保留显式选择。
- [ ] 菜单、构建表单、设置、依赖卡片展示真实工具；移除旧 withComments 和导出原始 dmsg/dstd 控件。
- [ ] truth 词法高亮共用，语义按种类分开；增加基于有效 map 的参数/枚举提示，复用 UI 模式，不引入新前端 parser 依赖。
- [ ] 替换旧偏移导航与行式时间线：符号标签按 script 作用域解析；循环显示结构，未知时间标记未求值，不能虚构实际运行时序。
- [ ] 迁移受影响的已有行为用例，旧格式用例随实现删除；不保留只证明旧字符串不再存在的测试。
- [ ] 截图或人工确认新文件、构建表单和问题跳转入口；不跑自动按钮/表单交互流程。

### Task 5：agent 通道与 skill

**Files:**
- Modify: `src-tauri/src/modules/mcp/{tools,server}.rs`。
- Create: `src-tauri/assets/truth-skill-template.md` 与对应辅助包生成模块（位置在 Task 0 接口回填时确认）。
- Modify: 现有辅助包入口，保持 ECL 包有效。

**Inputs:** Task 2 的有效语义查询、Task 3 的领域服务。
**Outputs:** MCP 与 UI 同源；agent 得到 truth 原生写法与明确的检查工作流。

- [ ] 新增 MSG/STD 检查、编译、反编译和语义查询，END 通过 kind 区分；工具说明写清版本、UTF-8 源码与游戏编码。
- [ ] 检查编译到临时产物，沿用现有 ECL check 的简单模式；不建立恢复队列或编辑缓冲区同步协议。
- [ ] skill 包含 map/枚举查询、语言子集、GBK 参数、每次改动后的检查及效果验证边界；不再教 agent 调 thmsg 或写旧方言。
- [ ] 用少量真实 MCP 调用验证返回值与同配置 UI 服务一致；复用 Rust 服务测试，不机械复制所有编译场景。

### Task 6：删除旧通道、打包与集中验收

**Files:**
- Remove: `modules/msg/translator.rs`、`modules/msg/signatures.rs`、`modules/msg/check.rs`（替换完成后），`modules/thstd/translator.rs`。
- Remove: export_raw_dmsg/export_raw_dstd 的命令、前端 API、菜单及废弃 fixture/test；仅删除已无消费者的旧 map。
- Modify: `src-tauri/tauri.conf.json`、构建 staging、对应平台打包说明、README/project/docs 现状。

- [ ] 清理旧模块声明、旧工具 ID、旧参数适配和导出调用；保留公共编码/文件模块仍被其他格式使用的部分。
- [ ] 安装包附带工具组和地图/许可，实际安装位置下可运行；移动安装目录或含空格路径的常规场景用一次代表性检查，不新增大测试框架。
- [ ] 验证没有外部 truth/thtk 的环境能执行 MSG/END/STD；ECL/DAT 按原配置继续工作。没有配置 thtk 不得阻断 truth 功能。
- [ ] 集中执行 `npm test`、`npm run typecheck`、`npm run build`、`tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml`、`git diff --check`。必要前置环境沿用 README；局部修正只重跑受影响项。
- [ ] 验证三类代表性二进制和 GBK 文本、入口、跳转、meta 信息。真实往返不强求不同编译器无条件字节一致，差异应有解释。
- [ ] Windows 安装包人工验收与游戏运行单独记录；未进行的部分明确标注。无须全作品或全故障矩阵。
- [ ] 更新 README、script-support-status、依赖来源与阶段状态；只有旧通道已移除、GBK 可用、随包运行通过时才标记完成。

## 执行记录（2026-11-09）

前述任务清单保留设计时的验收细节；以下为当前执行账目，不以文档勾选替代验证。

- Task 0：固定 truth CLI、普通脚本覆盖与 Studio 原生 map 接口已确定。基础核对见 deps/truth-baseline-check-2026-11-09.md。
- Task 1：MSG/END 编码补丁、固定 Rust 1.85.1 开发构建脚本、Linux/Windows CLI、随包许可与 build-info 已落地。没有维护 inspector API 或 truth 分支。
- Task 2：27 份原始社区 map；53 份版本基线；签名属性/padding/枚举/gamemap/alias；项目覆盖与源码 pragma 同源查询已实现。
- Task 3：truth_dir、truthDir、scriptMaps；trumsg/trustd 工具 ID；原生编译与 UTF-8 诊断；后台进程与输出路径已切换。
- Task 4：文件识别、共享高亮、独立语义补全/参数提示、标签跳转、结构时序列表、设置和菜单已切换。没有浏览器交互验收。
- Task 5：MCP run_script / lookup_script_semantics 与 truth skill 生成入口已接通；ECL 工具继续保留。
- Task 6：旧翻译器、签名修补、原始导出和旧方言测试已移除；集中验证及包状态见 [集成记录](../../../deps/truth-integration-check-2026-11-09.md)。

实施选择：使用 Node 构建脚本替代最初拟定的 shell 脚本，Windows 开发机可复用现有 Node；部署机制不变。结构时序只声明基础源码分析，不实现完整解析器/模拟器。保留 Windows 人工与游戏效果验收边界，不扩大为全作品 UI 自动化。

最终独立审查发现 TH10 中文说明编号错位、TH143/165 opcode33 名称错误，按固定来源修正并使用真实查询回归。小项暂缓：bool 内置常量未加入枚举查询表、复杂同一行/十六进制时序未完整分析、END 选项未按作品过滤（后端会拒绝不支持的组合）。不因此新增另一套状态或语言解析基础设施。

最终验证：前端 115 项、typecheck 与 build 通过；Rust 176 项通过，另 1 项真实工具集成用例显式执行通过。Windows NSIS 已生成；TH18 三类真实往返字节一致。上游 ANM 单项既有失败及人工验收边界见集成记录。
