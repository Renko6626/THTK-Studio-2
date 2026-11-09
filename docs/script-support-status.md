# 脚本支持现状

更新：2026-11-09。代码已切换至 truth；自动化验证与尚未执行的实机验收分开记录。

| 格式 | 当前实现 |
| --- | --- |
| ECL | thecl 编译/反编译/header；Monaco 语言支持；MCP/skill，保持原路线 |
| MSG stage | trumsg，`.msg` ↔ `.msgspec`，游戏文本 Shift-JIS/GBK/UTF-8，源码 UTF-8 |
| END/staff | trumsg --ending，`.msg` ↔ `.endspec`，独立 map 与模式 |
| STD | trustd，`.std` ↔ `.stdspec`，保留原生 meta、标签、控制流与表达式 |
| ANM | 暂按 thanm 路线，工具链与专用视图尚未完成 |
| DAT | thdat 解包/打包，保持原路线 |

MSG/END/STD 共用 truth 词法高亮，按 kind 查询有效 map，提供指令/枚举补全、悬停、参数提示和编译错误定位。Rust 结构分析提供 script 内标签跳转与源码事件列表；循环、分支后的时间标记未求值，不模拟游戏运行。当前结构提取为基础能力，不是完整 truth 语法树或资源预览。

MCP 的 `run_script`（check/compile/decompile）支持检查未保存文本；`lookup_script_semantics` 支持新文件、源码 pragma 和完整分页。工作区返回三类脚本的独立配置、headless 检查器和资料位置。

菜单生成完整 `.claude/skills/touhou-truth-scripts/` 辅助包，包含 skill、离线编程手册、项目实际配置、完整有效指令表和可编译例子；保留已有 SKILL／AGENTS，显式覆盖仅更新 skill。手册入口见 [MSG／END／STD 编程手册](agent/README.md)。

IDE、MCP 和独立 `thtk-check` CLI 共用 `crates/script-check` 的 truth 调用与检查实现。CLI 无 Tauri／WebView 依赖，支持普通文件、标准输入、项目配置、编码和 map 覆盖；在 Studio 终端通过 `THTK_CHECK` 调用。没有新增脚本 parser、符号索引或文档搜索服务。

## 迁移行为

- `.dmsg/.dstd` 不再具有语言服务或构建入口；旧翻译器、签名补丁与原始导出通道已移除。没有自动转换器，不删除用户文件。
- END 源码由 `.endspec` 扩展名确定模式；二进制文件默认按名称推断，也可在高级表单明确选择。
- 目录优先级：单工具 override > 项目 truthDir > 应用 truth_dir > 随包 truth。没有 thtk 也可使用 MSG/END/STD。
- map 顺序：版本基线 → 项目 `scriptMaps.msg/ending/std` → 源码 pragma。ECL 继续使用 `mapPaths`。
- truth 编译和反编译的工作目录为输入文件所在目录，不继承 `TRUTH_MAP_PATH`。工具失败不会回退旧后端。

目标作品见 [设计中的覆盖基线](superpowers/specs/2026-11-09-truth-msg-std-migration-design.md#迁移覆盖基线)。53 份默认 map 通过真实 CLI 的最小源码编译，TH18 三类真实文件往返字节一致；这不等于所有游戏效果均已验收。Windows 实机安装、编辑器交互和游戏内效果仍需人工确认，详见 [验证记录](../deps/truth-integration-check-2026-11-09.md)。

2026-06/08 的 MSG/STD 文档保留作历史记录，不再指导实现。

## 编辑器 UI 接入（2026-11-09）

脚本菜单按 ECL、MSG/END、STD 分类。MSG 二进制页支持关卡/结局与编码选择，MSG 和 STD 均可打开高级构建表单；原生源码上方提供检查、打包和构建选项。

Monaco 使用调用/原始 opcode/标签着色、带参数占位符的补全、枚举/常量/标签补全、嵌套表达式参数提示及脚本/标签导航。右栏提供 truth 大纲与标签引用。结构提取仍是基础分析，不是完整 AST。

当前缓冲区停顿 800ms 后交由真实 truth 检查，支持未保存内容；临时源码保持原目录 map 语义，检查后清理，不改变原文件。问题同步到问题面板和波浪线，过期响应不覆盖新缓冲区结果。目录不可写或工具不可用时操作栏显示检查失败。
