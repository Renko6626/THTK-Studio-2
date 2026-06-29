# thmsg 基本工作流实现计划

**Spec:** `docs/superpowers/specs/2026-06-07-thmsg-basic-workflow-design.md`
**环境:** 见 CLAUDE.md(conda tauri-dev)。基线 Rust 40 测试。

## thmsg CLI(确认事实)

- `thmsg -d {version} {input.msg}` → **stdout 二进制 dmsg**(SJIS)
- `thmsg -c {version} {input.dmsg} {output.msg}` → 写文件
- 不接受 `-m` mapfile

## 任务拆分

### Task 1 — JSON schema + 种子 + map_parser(Rust TDD)
- 转换旧 `thmsg_ref.json`(35 条)→ `src-tauri/assets/msg-th17.json`(规范 schema)。
  正则拆 `name(args)`;name 形如 `ins_N` 的条目跳过;args 失败用 `params:[]` 兜底。
- 新建 `src-tauri/src/modules/msg/{mod.rs, map_parser.rs}`:`MsgSemanticData` + 反序列化 + 版本回退 th17。
- `parse_msg_semantics(version: &str) -> Result<MsgSemanticData, String>`:读 `assets/msg-th{ver}.json`(include_str! 的多版本 LUT,或运行时按 thtk_dir/assets 读)。
  **决策:include_str! "msg-th17.json"**(种子内嵌,无需运行时找文件;后续版本扩展再改)。
- 模块挂到 `modules/mod.rs`。
- 测试(TDD):反序列化基本、容忍缺 section/description、`parse_msg_semantics("99")` 回退到 th17 内嵌。

### Task 2 — translator(Rust TDD)
- `src-tauri/src/modules/msg/translator.rs`
- API:`pub fn dmsg_to_readable(raw: &str, semantics: &MsgSemanticData, with_comments: bool) -> String` / `pub fn readable_to_dmsg(readable: &str, semantics: &MsgSemanticData) -> String`
- 行级正则:`^(\s*\S+:)?\s*ins_(\d+)\((.*)\)\s*$` / `^(\s*\S+:)?\s*([A-Za-z_]\w*)\((.*)\)\s*(//.*)?$`
- 未知 opcode 保留 `ins_N`;未知 name 保留原文(`compile` 时让 thmsg 自己报错);文本/注释/空行透传;`//` 行尾注释 strip
- 测试:textboxShow ↔ ins_3、带参数 playerShow(0) ↔ ins_1(0)、未知 opcode 保留、注释剥离、时间标签保留(`0:ins_3()` → `0:textboxShow()`)

### Task 3 — compiler + Tauri 命令(Rust)
- `src-tauri/src/modules/msg/compiler.rs`
  - `MsgRequest { mode: Decompile|Compile, version, input_path, output_path }`,`MsgResult`(同 EclResult shape:success/mode/inputPath/outputPath/diagnostics(空)/message)
  - decompile:run `thmsg -d {ver} {in.msg}`,**stdout bytes 写到 temp/{nanoid}.dmsg.raw**,然后 SJIS→UTF-8 解码→`translator::dmsg_to_readable`→写到目标 .dmsg(UTF-8)
  - compile:读目标 .dmsg(UTF-8)→`translator::readable_to_dmsg`→UTF-8→SJIS 编码→temp/{nanoid}.dmsg.raw→run `thmsg -c {ver} {temp} {out.msg}`
  - 复用 `common/cmd_runner.rs`(已支持 SJIS 解码;decompile 这里我们要原始 stdout bytes,需要新的 `cmd_runner::run_tool_bytes` 或直接 std::process::Command,**新加薄包装**避免污染 cmd_runner)
- `src-tauri/src/modules/msg/commands.rs`:`decompile_msg_file(input_path, output_path: Option<String>)` / `compile_msg_file(input_path, output_path: Option<String>)`,从 AppState 取 effective version(复用 mcp::tools::effective_toolchain_config? 不够——那是 ECL 的;msg 用同思路写个 `effective_msg_version(config, project_root) -> String` 在 commands.rs 内部)
- `src-tauri/src/main.rs`:注册两个命令
- 测试:`build_thmsg_args` 单测(参数序列);translator 已单测过;CLI 实跑测试**跳过**(Linux 无 thmsg.exe;Windows 验收)

### Task 4 — 前端集成(最小)
- `src/api/modules/compiler.js` 加 `decompileMsgFile` / `compileMsgFile`
- `src/services/workbench/editorViews.js`:`.dmsg` → `text`;`.msg` 已在(binary-script)?如不在补
- `src/services/toolchains/registry.js`:thmsg descriptor 升级 — `supportsBuildDialog: false` 保留,加 `decompileApi`/`compileApi` 字段或直接由菜单调
- `src/components/Common/MenuBar.vue`:菜单"脚本"新增 `script.decompileMsg` / `script.compileMsg`,从活动编辑器拿文件路径触发(若活动 tab 是 .msg → 解包;.dmsg → 打包;不匹配则禁用),错误/成功推到输出面板
- `src/components/Sidebar/FileTree.vue` 或 `useFileTreeActions.js`:右键 `.msg` / `.dmsg` 加对应菜单项(若现有右键扩展点不支持按扩展名条件显示,**先只做顶部菜单,右键后续**)
- 解包后自动打开生成的 .dmsg

### Task 5 — 验证 + 文档
- 跑 cargo test(应 ~46 个:40 baseline + ~6 个新 msg);npm run build
- 在 `editor-shell-status.md` 加 thmsg 工作流条目;在 `docs/script-support-status.md` 把 thmsg 行从 ❌ 改为 ✅(文本工作流)
- 在 `CLAUDE.md` 的 `modules/` 描述里加 msg

## 注意点

- **不要扩展 ECL 的 AI 辅助包**(spec 明确)
- **不要给 .dmsg 装语言服务**(spec 明确)
- 中间产物清理:成功后删 temp;失败时保留供调试(`cmd_runner` 已有 cleanup 失败容忍)
- 时间标签 `0:` / `60:`/ `+30:` 都要透传,**不可丢**
- 难度标签 `!EN` `!HL67` 等行级前缀透传
- thmsg version 经 `normalize`(`th17` → `17`)与 ECL 同
