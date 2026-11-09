# MSG／STD agent 编程支持实施计划

> **For agentic workers:** 用户已指定使用 Sol 子代理执行。主 agent 负责划分任务、接口协调与集成；子代理仅执行获派范围，不自行扩大架构或派发更多代理。

**Goal:** 让通用 agent 能依靠可信文档、普通文件工具、有效语义查询和真实检查，自然阅读、编写和修改现有支持范围的 MSG／END／STD。

**Architecture:** 保留外部 truth 编译器。建议将已有调用和检查逻辑抽到不依赖 Tauri 的小型 Rust 库，由 IDE、MCP 和独立 CLI 共用；领域知识以本地 Markdown、现有 map 和短说明交付。不给 agent 另造源码定位与编辑系统。

**Tech Stack:** 现有 Rust、Tauri v2、Vue 3／TypeScript、rmcp，以及固定 truth CLI。共享库只使用现有依赖族，不新增 parser、数据库或服务框架。

**Spec:** [设计与来源核对](../specs/2026-11-09-agent-scripting-design.md)。用户已确认执行；文档与 skill 采用面向编程者的直接写法，取证过程集中在来源笔记。

## 全局约束

- truth 固定 `c78a541d1fb09fcefd448c83b097d4f9b815fce3`，保留现有 GBK 补丁；本计划不升级或扩展 truth。
- 源码 `.msgspec`／`.endspec`／`.stdspec` 始终 UTF-8；不恢复 `.dmsg/.dstd`、thmsg 或 thstd。
- 使用现有 53 份版本／格式 map；默认 map → 项目 `scriptMaps` → 源码 pragma，工作目录为源码目录。
- ECL／ANM／DAT 行为、现有 MCP 接线、PTY 会话模型保持原路线。
- renkolab 仅作只读资料来源；不接触其未提交工作，不运行新逆向，不将游戏二进制加入仓库。
- 文档、样式和低风险接线不自动新增永久测试；关键新行为优先在共享库一层验证，前端复用已有测试。
- 验证按完整改动集中执行；UI 至多截图，不进行浏览器自动交互，不新增大范围 CI。
- 文档先教用法和例子；只有影响代码写法的限制就地说明。研究不确定性集中到来源／附录，不能让防御性表述主导教程或 skill，也不编造未知行为。

## 重点核对

1. 旧 thmsg／thstd 写法不可混入 truth 示例；MSG 与 END 编号空间不能混用。由任务 2 的真实示例检查覆盖。
2. 项目覆盖、相对 pragma、二进制编码在 IDE／CLI／MCP 一致。由任务 3 的真实工具用例覆盖，任务 5 只检查适配参数。
3. 新文件和未保存文本可检查／查询，磁盘原文不变，错误路径指向原文件。复用并迁移现有缓冲区检查用例。
4. 原始错误没有可解析位置时仍可见；不要用虚构的行号代替缺失位置。由诊断用例覆盖。
5. 更新辅助包保留用户 skill／规则，安装后的 CLI 不依赖开发目录或图形环境。分别由任务 6、7 验证。

## 文件职责

| 文件／目录 | 职责 |
| --- | --- |
| `docs/agent/`（新） | 面向脚本作者与 agent 的规范文档、来源和示例 |
| `crates/script-check/`（新，建议边界） | Tauri 无关的上下文、进程执行、检查与 CLI |
| `src-tauri/src/modules/truth/{compiler,check,catalog}.rs` | 保留现有接口，适配共享实现 |
| `src-tauri/src/modules/truth/{commands,maps}.rs` | 有效 map 查询与现有短说明；补齐新文件上下文和说明边界 |
| `src-tauri/src/modules/truth/ai_pack.rs` | 导出文档、指令表、示例和项目入口 |
| `src-tauri/src/modules/mcp/{server,tools}.rs` | MCP 参数与结果适配 |
| `src-tauri/assets/truth-skill-template.md` | 简短任务入口与工具选择规则 |
| `src/types/truth.ts`、`src/api/modules/truthMaps.ts`、`MenuBar.vue` | 生成包结果类型及已有菜单反馈 |
| `scripts/build-script-check.mjs`（新） | 构建、暂存 headless 程序 |

任务按 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 顺序实施。每项结束保留可审阅改动与证据，不要求为每次文案修正单独提交。

## 任务 1：形成可维护的来源与覆盖清单

**文件：**新建 `docs/agent/{README,sources,support-matrix}.md`；修改 `docs/README.md`。

**输入／输出：**使用设计文档中的固定资料与现有 `bundled_maps.rs`；产出后续手册必须遵守的版本、来源和已知限制清单。

- [ ] 核对 THBWiki 修订 `975551`，不使用搜索缓存中的 `896324`；以 MSG、END、STD 各节为入口。
- [ ] 阅读 renkolab 的 MSG 总览、Wiki 母稿及 TH16／18 一手记录，保留章节／版本／证据位置。主目录 HEAD 与工作树状态写入来源记录，不改动该仓库。
- [ ] 固定 ExpHP/thpages 的 `msg.ts`、`end.ts`、`std.ts`（提交 `effc37824ff4373b8251bedcc516b08bd9d452d7`）和 Priw8 三份 STD Markdown（提交 `1c0276927ec203f7adfafca01ec157dba52c35a7`）。记录作者、链接、适用范围；自行组织说明，避免整页搬运。
- [ ] 将资料分为“编译器语法／格式”“有效签名／枚举”“游戏行为”。相互引用的社区资料不重复计算为独立证据。
- [ ] 覆盖清单列全：MSG `6,7,8,9,10,11,12,128,13,14,143,15,16,165,17,18,185,19,20`；END `10,11,12,128,13,14,15,16,17,18,19,20`；STD `6,7,8,9,95,10,103,11,12,125,128,13,14,143,15,16,165,17,18,185,19,20`。每类分别写出行为资料覆盖和未知项。
- [ ] 列出当前冲突而不自动改工具：TH16 op27 的旧待验记录与 TH18 的已验行为、TH18 END op6 负数处理与社区说法、旧记录中的签名是否仍对应当前固定分支。先对当前源码复核，只有仍成立的冲突进入成稿。

**验收：**覆盖组合与现有 53 份 map 一致；TH19／20 不被标为已有 TH16／18 同等逆向证据。无产品代码或新增测试。

## 任务 2：编写能支持实际编程的领域手册

**文件：**新建 `docs/agent/{truth-syntax,msg,ending,std,version-differences,tooling}.md`、`docs/agent/examples/manifest.json` 和下述示例。

**输入／输出：**任务 1 的来源清单、固定 truth 源码／测试、现有 map；输出可离线阅读的手册和真实可编译的示例。文档不依赖运行时解析 Markdown。

- [ ] `truth-syntax.md`：文件结构、meta、script、常量与常量表达式、字符串、数值／颜色、时间标签、标签与跳转、当前格式允许的结构化控制流、原始 opcode／伪参数、map pragma。按 kind 说明限制；不把通用 truth 语法一概承诺给 MSG／STD。
- [ ] `msg.md`：入口表与 script 的关系、等待与脚本时钟、说话者、立绘／表情、文本分页与注音、BGM、ECL 握手、结束流程、编码。面向修改任务解释参数和行为，不堆引擎字段偏移。
- [ ] `ending.md`：独立 opcode 集、文本行与翻页、ANM 源与显示槽、等待、音乐、画面效果、staff 切换及已知版本差异。
- [ ] `std.md`：不同世代 meta、对象／quads／实例、ANM 脚本引用、相机位置／朝向／up、FOV、雾与颜色、插值、循环、ECL 中断。以 truth 字段和符号跳转重写旧教程，不引入手算偏移。
- [ ] `version-differences.md`：MSG 的早期格式、TH11 插位、TH143／165 分支、TH18／19／20 改动；END 的 TH20 差异；STD 的 TH06、TH07–09、TH095 起及后续增量。按实际 profile 区分，不按数字大小猜世代。
- [ ] 在手册中给出可搜索的指令名称、opcode、参数含义和所属版本。签名完整表由辅助包从 map 导出；手册只在解释行为所需的位置展示调用写法。
- [ ] 正文采用明确、可操作的写法；已知的常用行为直接说明。研究分级、历史疑问和取证过程放在 sources／附录，不逐条要求 agent 重新诊断，不把查询工具变成每次编辑的前置仪式。
- [ ] 创建 `th06-dialogue.msgspec`、`th10-dialogue.msgspec`、`th18-dialogue.msgspec`、`th18-ending.endspec`、`th20-ending.endspec`、`th06-background.stdspec`、`th08-background.stdspec`、`th18-background.stdspec`。使用自写最小内容，覆盖入口、等待、实例和符号跳转等实际结构；不复制游戏资源。
- [ ] `manifest.json` 每条固定 `path, kind, gameVersion, encoding`，一个示例只声明实际检验过的目标；STD 不附加 MSG 编码语义。
- [ ] 手册示例引用这些真实文件，标明编译验收与游戏效果验证的区别。任务 8 通过共享检查器集中检查；撰写期间只做必要的 truth 调用确认语法。

**验收：**agent 能找到格式字段、参数意义、典型写法及版本限制。没有“请自行浏览外部网页才知道怎么写”的必要步骤。未知行为允许保留，不能伪造支持或扩大逆向工作。

## 任务 3：抽出共享的 truth 调用与检查内核

**文件：**新建 `crates/script-check/Cargo.toml`、`crates/script-check/Cargo.lock`、`crates/script-check/src/{lib,types,context,runner,diagnostics,catalog}.rs` 和生成的 `crates/script-check/src/bundled_maps.rs`；修改 `src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`scripts/import-truth-maps.py`、`src-tauri/src/modules/truth/{compiler,check,catalog,bundled_maps}.rs`。只有确实需要的现有版本解析辅助代码才移动，不迁移整个配置系统。

**建议接口：**

```rust
pub enum ScriptKind { Msg, Ending, Std }
pub enum ScriptOperation { Check, Compile, Decompile }
pub fn resolve_context(input: &ContextInput) -> Result<ScriptContext, CheckError>;
pub fn run(context: &ScriptContext, request: &ScriptRequest)
    -> Result<ScriptResult, CheckError>;
```

`ContextInput` 字段为 `kind, input_path, project_root, explicit_game_version, explicit_encoding, explicit_truth_dir, explicit_map_paths, app, project, bundled_truth_dir`。应用快照只有 `default_game_version, truth_dir, tool_overrides`；项目快照只有 `game_version, encoding, script_maps, truth_dir`。这些结构没有 AppState、ConfigManager 或窗口句柄；完整 IDE 配置类型保持原处，适配器只取所需字段。

`ScriptContext` 包含 `kind, game_version, encoding, tool_path, map_paths`。`map_paths` 是默认＋项目 map；源码 pragma 继续由 truth 原生加载。查询端返回的 `sourcePaths` 才包括源码 pragma，不混淆两种列表。

`ScriptRequest` 包含 `operation, input_path, output_path, source_text`。`source_text: None` 表示磁盘文件；`Some("")` 是空缓冲区。源码文本只允许用于 Check，避免增加隐式保存／编译规则。

`ScriptResult` 保持现有 `success, tool, mode, scriptKind, inputPath, outputPath, message, diagnostics` 的 JSON 形状。共享库自己的 kind／诊断类型通过薄适配转为现有类型，避免让它依赖 rmcp 或 ECL 模块。

- [ ] 将已有 truth 进程参数、输出收集、诊断解析和检查临时文件逻辑迁入共享库；编译／反编译也复用这段执行逻辑，不留下第二份 runner。
- [ ] 默认 map 内容仍保存在 `src-tauri/assets/maps/truth/defaults/`；调整现有生成脚本输出共享 registry，后端 catalogue 转发共享实现。不要维护两份 profile 列表或改变已发布 map 内容。
- [ ] 固定优先级：显式调用参数 > 项目设置 > 应用设置 > 既有默认值。保留单工具 override 优先于工具目录、源码 cwd、原生 pragma 顺序及移除 `TRUTH_MAP_PATH` 的既有行为。
- [ ] 项目文件读取继续使用已有字段；共享快照只包含 `gameVersion, encoding, scriptMaps, toolchain.truthDir` 和相关应用工具设置。CLI 与后端复用同一解析／优先级逻辑，不复制各自的版本或 map 选择算法。读取失败明确返回，不改写配置或建设恢复流程。
- [ ] IDE 的 `check_truth_source`、编译命令及 MCP runner 变成适配器。缓冲区同目录临时源码、原路径诊断、清理、原文件不变等现有行为保留。
- [ ] 诊断保留原始输出；没有位置的错误留在 `message`，不编造位置。仅补齐对当前真实输出确有需要的解析，不另造完整 diagnostic 协议。
- [ ] 迁移已有真实工具用例到共享库，覆盖 GBK、项目 map＋相对 pragma、未保存错误定位和原文件不变；增加一个新文件缓冲区用例、一个无位置错误用例。预期位置／结果独立写出，不复制实现。

**验收命令：**`cargo test --manifest-path crates/script-check/Cargo.toml`，随后运行受影响的后端 truth 测试。真实工具用例需要 prepared host truth，必须显式执行；仅编译或跳过用例不算验证通过。执行时 truth 路径由准备好的资源提供。

## 任务 4：提供真正 headless 的 thtk-check

**文件：**新建 `crates/script-check/src/{main,cli}.rs`、`crates/script-check/tests/cli.rs`；更新 `docs/agent/tooling.md`。

**CLI 契约：**

```text
thtk-check check SOURCE [--project ROOT] [--kind msg|ending|std]
    [--game VERSION] [--encoding ENCODING] [--truth-dir DIR]
    [--map PATH ...] [--stdin] [--json]
```

- `SOURCE` 按调用目录解析，决定源码 cwd；省略 kind 时只按三种源码后缀推断。
- `--project` 显式指定时读取该根的 `.thtk-project.json`；省略时从 SOURCE 所在目录向上查找最近的项目文件，不扫描整个目录树。
- 应用设置从现有 settings.json 位置只读获取相关字段；未安装 Studio 时允许显式参数＋随程序交付的工具工作，不创建配置。项目缺版本时沿用应用默认；最终采用值必须出现在 JSON 中。
- `--map` 追加调用级覆盖，位于项目 map 之后、源码 pragma 之前；不是另建 map 配置文件。
- `--stdin` 从标准输入读 UTF-8 源码，允许 SOURCE 尚未创建；与 IDE 缓冲区走同一检查函数。
- JSON stdout 固定一个 `{context, result}` 对象；日志写 stderr。`context` 使用任务 3 的字段及 camelCase。纯文本默认输出可读诊断。
- 退出码：`0` 检查成功（可含 warning）；`1` truth 拒绝源码；`2` 参数、配置、读写或启动工具失败。失败也保留可读信息，JSON 模式不混入日志。

- [ ] 使用小型显式参数解析，不为有限选项新增 CLI 框架；实现 `--help` 和上述契约。
- [ ] CLI 与 IDE 构造同一个 `ContextInput`，不依赖 MCP token 或运行中的 Studio。
- [ ] 复用一个有效／无效真实源码，验证 stdout JSON 和退出码；再验证 `--stdin` 新文件。不在 CLI 层复制共享库全部场景。
- [ ] 在没有 DISPLAY 的环境运行实际 CLI；检查依赖树没有 Tauri／WebView／truth Rust 库。

**验收命令：**`cargo test --manifest-path crates/script-check/Cargo.toml --test cli`；`env -u DISPLAY -u WAYLAND_DISPLAY <thtk-check路径> check <有效源码> --project <临时项目> --json`。Windows 控制台程序保留 stdout／stderr，不使用 GUI subsystem。

## 任务 5：补齐必要的 MCP 上下文与查询

**文件：**修改 `src-tauri/src/modules/mcp/{server,tools}.rs`、`src-tauri/src/modules/truth/{commands,maps}.rs`；按实际短说明修正 `src-tauri/assets/maps/*.zh.json`。不修改无关 MCP 工具。

**接口增量：**

| 现有工具 | 增量 |
| --- | --- |
| `get_workspace_info` | 新增 `scriptProfiles`，每项返回 kind、有效游戏版本／编码／项目 map 和可用状态；新增 `checkerPath`、`referenceRoot`，未准备时为 null。保留原有字段 |
| `run_script` | 新增可选 `source_text`，只允许 check；统一调用共享库。编译／反编译和已有必填参数保持兼容 |
| `lookup_script_semantics` | 保留 kind／source_path／query；新增可选 `source_text`、`offset`、`limit`，返回 `total`、`nextOffset`。默认 limit 50，允许 1–200；保留原 definitions 字段 |

`source_path` 仍提供相对 pragma 的解析目录。提供 source_text 时无需磁盘文件；不提供且文件不存在时以空源码查询项目定义，返回明确的上下文说明；其他读盘错误正常报告。没有源码 pragma 时不假装已解析某个已有文件。

- [ ] 工作区 profile 不借用 ECL 的 mapPaths；分别按 msg／ending／std 解析。没有 END 的版本返回该 kind 不可用说明，不连带隐藏其他 kind。
- [ ] 查询先过滤、再计数、再分页；指令按 opcode 排序。query 保持原来的名称子串／精确 opcode 规则，枚举保留现有完整返回。
- [ ] 返回实际 map 来源，指令附可读参考文件路径。先比较项目有效签名与版本基线；签名被改写时不沿用原版行为保证，短说明标出原版适用范围。别名改名但签名未变时保留原版解释及来源。
- [ ] 按任务 1 的核对结果补全短说明，尤其 TH19／20；未知项照实显示，不从别的作品按 opcode 硬套。
- [ ] 复用现有后端测试设施：验证三种 kind 的上下文分离、新文件 pragma 查询、遍历分页不丢指令、自定义签名不会冒充原版行为。共享库已覆盖的编译场景不在 MCP 再做一套。
- [ ] 更新 MCP description 和 skill 的调用示例；编译失败仍通过 `success=false` 和诊断表达，不把 HTTP／MCP 调用成功当作源码成功。

**验收命令：**`cargo test --manifest-path src-tauri/Cargo.toml modules::mcp` 和受影响的 `modules::truth` 测试；MCP schema 实际生成后核对新增参数与默认值。不新增文档检索、文件操作或源码分析工具。

## 任务 6：生成真正可用的 agent 辅助包

**文件：**修改 `src-tauri/assets/truth-skill-template.md`、`src-tauri/src/modules/truth/ai_pack.rs`、`src/types/truth.ts`、`src/api/modules/truthMaps.ts`、`src/components/Common/MenuBar.vue`、`src-tauri/src/wire_format_tests.rs`。

**产物：**

```text
AGENTS.md                               # 仅缺失时创建简短入口
.claude/skills/touhou-truth-scripts/
  SKILL.md                              # 保留手改；显式更新才覆盖
  references/
    README.md、PROJECT.md               # 索引、实际版本／编码／工具位置
    truth-syntax.md、msg.md、ending.md、std.md
    version-differences.md、support-matrix.md、sources.md、tooling.md
    instructions/msg-thVERSION.md       # 按本项目三个可用 kind 导出
    instructions/ending-thVERSION.md
    instructions/std-thVERSION.md
    examples/                           # 自写源码与 manifest
```

项目指令表是“基线＋项目 map”的快照；文件级 pragma 可能改变定义，精确查询使用 MCP。通过普通文件读取访问所有文档，不要求特定客户端能自动发现此目录。

**接口：**`generate_truth_assist_pack(force: bool) -> TruthAssistPackResult`，返回 `skillPath, skillWritten, skillExisted, referenceFiles, projectRulesPath, projectRulesWritten, suggestedRulesText`。后端 core `generate(project_root, context, force)` 不依赖 Tauri，命令负责取 AppState。force 只覆盖 skill，不覆盖已有 AGENTS.md。

- [ ] 将任务 2 的文档／示例作为构建时静态资源导出，规范母本只有 `docs/agent/` 一处；整个母本目录对应包内 `references/`，使文档间及 examples 的相对链接保持有效。不维护多份手写副本，不在运行时联网。
- [ ] 复用现有有效 map 解析器渲染三类完整指令表，包含 opcode、别名、参数、枚举、短说明和来源。没有 END 的项目不生成虚假的 END 表。
- [ ] 明确 references／examples／PROJECT.md 为生成文件。资料不足时提供有效签名与未知说明，不因缺中文行为解释而宣称整个工具不可用。
- [ ] skill 指导自然工作流：先读入口与相关章节，普通工具编辑，必要时查有效定义，CLI 或 MCP 检查；允许一次完成一组相关编辑后检查，不规定每改一行都调用工具。
- [ ] 根 AGENTS.md 缺失时写入口与必要项目规则；存在时不改写，返回建议加入的短段落。当前 Studio 开发仓库的 AGENTS.md 不充当用户 mod 项目模板。
- [ ] 复用 ECL 已有“保留／显式覆盖”交互模式，修改 truth 的菜单结果反馈；不顺带重构 ECL 生成器。跨边界返回值同步 TS 类型与 wire-format 检查。
- [ ] 给生成器保留一个初次生成用例和一个重复生成用例，验证用户 skill／规则不变而生成参考刷新。通过实际生成文件内容及有效 map 结果断言，不用源码字符串测试模板实现。

**验收：**辅助包脱离本机 renkolab 和网络仍可使用；参考链接不泄露成必须存在的 `/data/...` 绝对路径。运行受影响 ai_pack／wire-format 测试和 `npm run typecheck`。模板文字本身不单独新增测试。

## 任务 7：随包交付与终端发现

**文件：**新建 `scripts/build-script-check.mjs`；修改 `src-tauri/tauri.conf.json`、`.gitignore`、`src-tauri/src/common/{toolchain,pty}.rs`、`deps/README.md`；必要时更新实际交付来源记录，不伪造未构建平台的 hash。

- [ ] 使用项目现有 Rust 工具链构建独立 crate；host／显式 target 参数遵循 `build-truth.mjs` 的现有调用习惯。CLI 产物暂存 `src-tauri/resources/script-check/`，加入 resources 映射；将其构建接到 beforeDev／beforeBuild。忽略 crate target 和暂存产物，提交 lockfile，不引入新的顶层 Cargo workspace。
- [ ] 保留 truth 三程序同目录要求；检查器通过现有目录覆盖或相邻资源目录 `truth/` 找工具。开发态、安装态使用各自明确资源根，不嵌入开发机绝对路径。
- [ ] 增加 `toolchain::bundled_checker_path()`，仅负责定位；工作区信息与辅助包共用它。PTY 注入 `THTK_CHECK` 为检查器绝对路径，沿用已有环境变量注入，不修改用户 PATH 或全局 shell 配置。
- [ ] 文档给出 POSIX `"$THTK_CHECK" check ...` 和 PowerShell `& $env:THTK_CHECK check ...` 的例子；外部终端可直接使用检查器完整路径。
- [ ] 检查暂存／安装资源布局中的实际 CLI，可以关闭 IDE 后运行；Windows 做已有发布目标的编译和资源布局检查，Windows 实跑若无环境则明确留待验。不重建跨平台发布矩阵。

**验收：**host 产物通过任务 4 的真实 headless 命令；Windows 控制台构建通过且打包资源有对应程序。无 Windows 环境时不声称安装／控制台实跑成功。纯计划阶段不执行这些构建。

## 任务 8：集中验证与实际编程验收

**文件：**新建 `scripts/check-agent-examples.mjs`、`deps/agent-scripting-check-2026-11-09.md`；更新 `docs/script-support-status.md`、`deps/README.md` 和本计划执行记录。验证记录放开发资料目录，不混入面向编程者的辅助包。

- [ ] 示例脚本只读取 manifest，调用实际 thtk-check 并检查退出码与 JSON；不在脚本里复制编译或 map 逻辑，不写独立测试框架。
- [ ] 运行共享库／CLI 测试、受影响 Rust 测试、已有前端检查与构建；必要完整命令见下。只在新失败或改动后补跑受影响检查。
- [ ] 用一份临时项目做四个实际任务：修改 MSG 多段对话及节奏；编写最小 END 演出；修改 STD 对象实例／相机循环；引入并修正一个真实参数错误。以文档、普通文件编辑和 CLI／MCP 完成，不依赖源码定位器。
- [ ] 额外选一个项目 map 别名＋源码 pragma 的任务确认静态参考与有效查询的区别；复用任务 3／5 的夹具，避免再建一套工程。
- [ ] 记录每个任务读了哪些资料、实际命令、诊断、最终产物及未确认的游戏效果。允许用户后续实跑反馈；不把人工游戏验收改成自动化工程。
- [ ] 汇总仍未知的语义或不支持的格式，只将确实阻碍任务的项列为后续决策；不在结束前无限扩充工具和验证。

**最终集中检查：**

```bash
cargo test --manifest-path crates/script-check/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml modules::truth
cargo test --manifest-path src-tauri/Cargo.toml modules::mcp
cargo test --manifest-path src-tauri/Cargo.toml wire_format_tests
npm run typecheck
npm test
npm run build
node scripts/check-agent-examples.mjs --checker <已构建的thtk-check路径>
git diff --check
```

需要 prepared host truth 的既有 ignored 测试按实际保留的测试名显式运行并记录；迁移后不要求同时保留重复用例。不把 ignored／未运行的测试写成通过。

## 本轮计划交付记录

- 已阅读本仓库现有 truth 编译、缓冲区检查、map 查询、MCP、辅助包与打包接线。
- 已核对指定资料的实际入口、THBWiki 当前 revision、社区固定提交及 renkolab 的版本边界。
- 本轮仅写设计和实施计划；未新增检查器、MCP 或产品文档，未运行产品构建／测试。
- 下一步先审阅本文的共享库边界、CLI 契约、MCP 增量及辅助包保护规则；确认后再实施。默认建议主 agent 顺序执行，按阶段提交可审阅结果。
