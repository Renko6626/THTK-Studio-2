# thmsg 基本工作流 + dmsg 翻译层

日期:2026-06-07 状态:已批准范围,待落细节

## 目标

让 .msg 文件能在 IDE 内"解包→读名→编辑→打包"闭环,**不做 IDE 集成的高级语言服务**(无补全/悬停/跳转/MCP 工具/AI 辅助包扩展/语法高亮——这些是"花活",留给后续)。

## 在范围

1. **JSON 语义数据**(规格)
   - schema 与 `EclMapSemanticData` 同形(opcode/name/params/section/description),按 `{tool}-{version}.json` 切分
   - 种子:`src-tauri/assets/msg-th17.json`,从旧仓库 `thmsg_ref.json` 转换得到(扁平 `{opcode:[sig,desc]}` → 规范 schema)
   - 找不到对应版本时,回退到 th17 不阻塞工作流

2. **Rust `modules/msg/`**(后端骨架,套 ECL 模块形状)
   - `compiler.rs`:调 thmsg -d/-c,返回 EclResult-like 结构(共享 `MsgResult`)
   - `translator.rs`:`.dmsg`(thmsg 原始,`ins_N`)↔ 可读 `.dmsg`(`textboxShow`) 双向翻译
   - `map_parser.rs`:JSON 反序列化,返回 `MsgSemanticData`(类比 `EclMapSemanticData`)
   - `commands.rs`:Tauri 命令 `decompile_msg_file` / `compile_msg_file`
   - **不做 error_parser**——thmsg stderr 整体放进 message,无结构化诊断、无 Monaco 波浪线

3. **文件命名与编码**(对齐 ECL 心智模型)
   - `.msg` = 二进制,thtk 标准
   - `.dmsg` = 用户可见可编辑的**可读源码**(UTF-8,Monaco 文本视图)
   - thmsg 原始 SJIS 输出走 `std::env::temp_dir()` 中转,翻译后立刻写入目标 `.dmsg`(UTF-8)
   - 编译前在 temp_dir 临时转回 SJIS 喂给 thmsg
   - 复用现有 `save_file(is_source=true/false)` 编码区分

4. **前端最小集成**
   - `editorViews.js` 加 `.dmsg` → Monaco 文本视图(纯编辑,无语言服务)
   - `editorViews.js` 加 `.msg` → 现有 `binary-script` 视图(显示反编译入口)
   - `services/toolchains/registry.js` 的 thmsg descriptor 从 stub 升为可用(executor 实现,但 `supportsBuildDialog: false` 保留——走菜单/右键,不开 BuildDialog)
   - 顶部菜单"脚本"加 `script.decompileMsg` / `script.compileMsg`,与 ECL 三项平级
   - Output/Problems 面板复用 publishToolResult 显示结果

5. **版本解析复用 ECL effective_toolchain_config 模式**
   - 优先 `.thtk-project.json` 的 gameVersion;回退 app config default_game_version;最终 fallback th17

## 不在范围(明确"不做花活")

- Monaco 的 .dmsg 语法高亮、补全、悬停、跳转、签名帮助、文档符号
- MCP 工具 `check_msg` / `compile_msg` / `decompile_msg` / `lookup_msg_semantics`
- AI 辅助包(`generate_ai_assist_pack`)扩展到 msg references
- thmsg stderr → 结构化诊断 → Monaco markers
- BuildConfigDialog 集成
- 多 .dmsg 文件批量处理 UI、汉化辅助工具
- 跨版本 opcode 差异提示

## JSON Schema(锁定)

```jsonc
// src-tauri/assets/msg-th{ver}.json
{
  "tool": "thmsg",
  "version": "17",
  "instructions": [
    {
      "opcode": 3,
      "name": "textboxShow",
      "params": [],
      "section": "textbox",          // 可选;无则省略
      "description": "显示对话框。"
    },
    {
      "opcode": 1,
      "name": "playerShow",
      "params": [{ "name": "who", "type": "int" }],
      "section": "portrait",
      "description": "显示玩家的角色立绘。"
    }
  ]
}
```

Rust 结构(对应 EclMapInstructionSpec 命名):

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgInstructionParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgInstructionSpec {
    pub opcode: u32,
    pub name: String,
    #[serde(default)]
    pub params: Vec<MsgInstructionParameter>,
    pub section: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgSemanticData {
    pub tool: String,
    pub version: String,
    pub instructions: Vec<MsgInstructionSpec>,
}
```

## Translator 行为(锁定)

**dmsg 原始 → 可读**(line-based,thmsg 输出格式稳定):

输入行示例:
```
0:ins_3()
0:ins_1(0)
60:ins_30(1)
```

规则:
- `时间:ins_<opcode>(<args>)` → `时间:<name>(<args>)`,name 查 `instructions` by opcode
- 未知 opcode:**保留原样**(`ins_999()`),不阻塞工作流
- 注释行 / 空行 / 段落标记(`!difficulty` 等)原样透传
- 行尾追加注释 `// <description>`(可选,通过开关控制,默认开)
- 文本字符串(`T="..."` / 引号包裹)原样透传

**可读 → dmsg 原始**(反向):
- 反查表 name → opcode,替换为 `ins_N`
- 行尾的 `// ...` 注释 strip 掉
- 其他原样保留

不实现复杂解析(无 AST、无参数类型校验)——纯行级文本变换。

## 命令清单

新增 Tauri 命令:
- `decompile_msg_file(path: &str, output_path: Option<String>) -> MsgResult`
  - thmsg -d → temp/.raw.dmsg → translator → output(默认同目录同名 .dmsg)
- `compile_msg_file(path: &str, output_path: Option<String>) -> MsgResult`
  - translator → temp/.raw.dmsg(SJIS) → thmsg -c → output(默认同目录同名 .msg)
- `get_msg_semantics(version: &str) -> MsgSemanticData`(可选,前端如果想列指令)
  - 找不到对应版本回退 th17

## 测试

Rust(TDD 各自):
- `translator`:dmsg→readable 包含 textboxShow / playerShow 翻译;round-trip readable→dmsg→readable 恒等;未知 opcode 保留 ins_N;注释剥离
- `map_parser`:JSON 反序列化基本;缺 section/description/params 默认值容忍
- `compiler`:套 ECL 现有 cmd_runner mock 模式,验证参数构建(`thmsg -d {ver} {in}` / `thmsg -c {ver} {in} {out}`)——具体参数语法以旧 wrapper 与 thmsg --help 为准

前端:`npm run build` + 手动验收(在 Windows 拿一个真 .msg 试)

## 落地里程碑(便于排 task)

1. JSON schema + 种子 msg-th17.json + map_parser(+测试)
2. translator(+往返测试)
3. compiler 调 thmsg + cmd 命令(+测试)
4. 前端 editorViews + registry executor + 菜单接线
5. 手动验收清单(在 status doc 里追加)

## 后续(明确不做但记上,future spec)

- msg 语义查询 MCP 工具 + AI 辅助包扩展(让 agent 也能改 msg)
- msg 静态诊断(thmsg stderr 解析)
- thstd 走同模板(更简单,无 dmsg 翻译层——thstd 直出文本)
- thanm 文本层 + sprite 预览(独立 spec)
