# 阶段总结:thmsg / thstd / thdat 工作流落地

日期:2026-06-07
范围:`7232513..9314fb3`,17 个提交,从 ECL-only 扩展为四工具链 IDE

## 这轮做了什么

把 IDE 从"thecl 一等公民,其余 stub"升级到**四工具链皆可用**(thecl/thmsg/thstd/thdat),并修了 binary-script 视图把非 ECL 二进制误送 Monaco 的真问题。

## 工具链现状(完整表)

| 工具 | 后端模块 | 测试数 | 翻译/语义层 | 前端入口 | 备注 |
|---|---|---|---|---|---|
| **thecl** | `modules/ecl/` | 8(map_parser/ai_pack/...) | eclmap → Monaco 全套语言服务 + MCP 工具 + AI 辅助包 | BuildDialog + 菜单 + binary-script 视图 + 文件树右键 | 一等公民,设计标杆 |
| **thmsg** | `modules/msg/` | 19(map_parser 3 + translator 9 + compiler 7) | `assets/msg-th17.json` 33 条 → 行级 ins_N↔name + SJIS 桥接 | 菜单(activeTab) + binary-script 视图按钮 | 基本工作流 |
| **thstd** | `modules/thstd/` | 16(map_parser 3 + translator 9 + compiler 4) | `assets/std-th17.json` 19 条 + **opcode 1 (jmp) 参数交换** + UTF-8 全程 | 菜单(activeTab) + binary-script 视图按钮 | 基本工作流;`@label` 预处理暂缓 |
| **thdat** | `modules/thdat/` | 6(compiler 6) | **无**(纯容器管理) | 菜单 + native pickers + binary-script 视图按钮 | 基本工作流;不递归子目录 |
| thanm | — | — | — | binary-script 视图按钮(disabled) | **未实现**(下次单独 spec) |

总:**81 个 Rust 单测,全绿;`npm run build` 通过**。

## 统一约定(四工具落地后浮现)

### 文件命名

| 工具 | 二进制 | 可读源码 |
|---|---|---|
| thecl | `.ecl` | `.decl` |
| thmsg | `.msg` | `.dmsg` |
| thstd | `.std` | `.dstd` |
| thdat | `.dat` | (容器,无源码扩展) |

**"d 前缀 = decompiled = 用户可见源码"** 的心智模型,跨工具一致。

### Rust 模块形状

每个工具都是 `modules/{tool}/`,包含:
- `compiler.rs` — CLI 进程封装,返回 `*Result` 结构(success/mode/inputPath/outputPath/diagnostics(空,frontend 兼容)/message)
- `commands.rs` — Tauri 命令 + `effective_*_version` 取自 project_config / app config

可选(只有需要的工具才加):
- `map_parser.rs` — JSON 语义数据反序列化,种子 `include_str!` 内嵌
- `translator.rs` — 行级 `ins_N` ↔ name 双向变换

### 编码与缓冲区策略

| 工具 | 编码 | thtk -d 输出方式 |
|---|---|---|
| thecl | UTF-8 (.decl) / 用户控 SJIS 标志 | 写文件 |
| thmsg | **SJIS ↔ UTF-8 桥接**(.dmsg 落盘为 UTF-8) | stdout(需 bytes 捕获) |
| thstd | **全程 UTF-8** | 写文件 |
| thdat | N/A(字节透传) | 写目录 |

### 配置与版本

- 应用级:`AppConfig.default_game_version`、`thtk_dir`、`thecl_path` 等
- 项目级:`.thtk-project.json` 的 `gameVersion`(优先)
- 解析逻辑:每个工具一个 `effective_*_version`,**逻辑相同但不抽象**(早期抽象代价高于收益,3 个 copy)
- thdat extract 用 `d` 自动检测,**绕开版本依赖**

### 前端模式

- 菜单:`MenuBar.vue` 的"脚本"段每个工具两条(解包/打包),根据 activeTab 后缀启用
- 视图:`BinaryScriptView.vue` 用 `TOOL_DESCRIPTORS` 表分发,按 `tab.extension` 显示文案 + 操作
- 结果:统一推到输出面板的 `publishToolResult` 卡片

## 这轮顺手修的真问题

`stores/editor.js` 的 `isBinaryEclFile` 写死 `extension === 'ecl'`——`.msg`/`.std`/`.dat`/`.anm` 全部走 Monaco text,导致 `.msg`/`.std` 显示 SJIS 乱码、`.dat` 可能 100MB 卡死。改成 `BINARY_SCRIPT_EXTENSIONS` 集合 + 视图按 extension 分发(commit `9314fb3`)。

`.anm` 也进 binary-script 视图但按钮 disabled——thanm 工具链做完之前,提示"待实现"比让用户用 Monaco 拆字节友好得多。

## "不做花活"原则的兑现

四个工具都明确**没做**(被明确从范围中切除):
- Monaco 语法高亮 / 补全 / 悬停 / 跳转 / 签名帮助 / 文档符号(thmsg/thstd/thdat 全部)
- 结构化诊断 + Monaco 波浪线(thmsg/thstd/thdat)
- MCP 工具(check_msg/lookup_*_semantics 等)
- AI 辅助包扩展(generate_ai_assist_pack 仍只覆盖 ECL)
- BuildConfigDialog 集成(thmsg/thstd/thdat 都走菜单 + native pickers)

这些是**有意识的范围切割**——把工作量留给"用户痛点最高"的功能,而不是机械地把 ECL 全套照葫芦画瓢。

## 暂缓事项(用户决策)

- **thstd `@label` → 字节 offset 预处理**:旧 PyQt 版有 `_preprocess_jmp_labels`,用户决定封装到下一版 thtk 工具,IDE 不实现
- **thanm 工具链**:文本层 + sprite 预览,下次单独 brainstorm(预览层另开 spec)
- **thdat list 浏览 UI**:解出来用文件树看就行,marginal value

## 后续工作清单(优先级排序)

1. **thanm 文本层**(类比 thmsg/thstd 模式,~thmsg 80% 工作量,有 anmm map 文件)
2. **thanm sprite 预览**(领域价值最大,但工作量与 thecl 同级,**单独 spec**)
3. msg/std MCP 工具(让 agent 能改对话/std,~3 小时机械活)
4. msg/std AI 辅助包扩展(`generate_ai_assist_pack` 泛化产出 references,让 SKILL.md 涵盖 msg/std)
5. msg/std 结构化诊断(thmsg/thstd stderr 解析,Monaco markers)
6. msg/std 语言服务(Monarch 高亮 + 补全/悬停,~thecl 30% 工作量)
7. thdat batching(超过 28KB 命令行长度时分批打包)
8. 旧一次性命令面板代码彻底清理(`api/modules/terminal.js`、`run_shell_command` Rust 端)

## 验收清单(Windows 单独跑)

集中在 `editor-shell-status.md` §9,条目 14-25 共 12 条覆盖 thmsg/thstd/thdat 三个工具链 + 各自的菜单触发、auto-open、错误路径、未配置失败、版本回退等。

## Spec 与 Plan 文档索引

本轮产出的 spec/plan:

- `docs/superpowers/specs/2026-06-07-thmsg-basic-workflow-design.md`
- `docs/superpowers/plans/2026-06-07-thmsg-basic-workflow.md`
- `docs/superpowers/specs/2026-06-07-thstd-basic-workflow-design.md`
- `docs/superpowers/specs/2026-06-07-thdat-basic-workflow-design.md`
- `docs/script-support-status.md` ←总览状态表,以后改这个
