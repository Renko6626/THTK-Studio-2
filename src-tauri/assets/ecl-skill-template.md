---
name: ecl-modding
description: 在本项目中编写或修改东方 Project ECL 脚本(.decl)时使用 — ECL 语言核心概念、thtk 工作流、何时查 references 与何时调用 thtk-studio MCP 工具
---

# ECL 脚本魔改指南（th{{VERSION}}）

## ECL 是什么

ECL 控制东方 Project 原作的敌机行为与弹幕逻辑。二进制 `.ecl` 文件由 thecl
反编译为文本（本项目约定扩展名 `.decl`），修改后再编译回 `.ecl`。

## 核心概念

- **sub（子程序）**：`void sub_name(...)` 形式的过程，敌机/弹幕逻辑的基本单元。
  关卡主流程通常从 `main` 系列 sub 开始，boss 行为在专门的 sub 里。
- **时间标签**：行首的 `数字:` 表示”等到本 sub 时钟达到该帧再继续”。
  60 = 1 秒（60fps）。这是 ECL 时序的核心——指令按时间标签调度，不是顺序立即执行。
  反编译输出里更常见相对形式 `+30:`（相对上一标签推进 30 帧,行尾注释里是绝对帧数）。
- **难度分支**：`!EN`、`!HL`、`!*` 等行首标记限定后续行只在指定难度生效
  （E=Easy，N=Normal，H=Hard，L=Lunatic，*=全部）。
  实际反编译输出中常见带数字后缀的掩码形式如 `!HL67`、`!E67`——数字是更高难度位(Extra/Overdrive 等)的掩码,移动代码时同样原样保留。
- **变量与寄存器**：
  - 局部变量：`int x = 0;` / `float y = 1.0;`
  - 全局寄存器：整型/浮点寄存器命名随 eclmap 而异（如 `I0`/`F0`，zero318 系 map 则为 `EI0`/`EF0`），以及大量只读系统寄存器
    （玩家坐标、随机数等）。完整列表见 references/{{REGISTERS_FILE}}。
  - `$` 前缀按 int 读，`%` 前缀按 float 读。
- **指令（ins）**：`ins_23(60)` 或经 eclmap 映射后的别名 `wait(60)`。
  同一 opcode 在不同作品中含义可能不同——**不要凭记忆写指令，先查签名**。

## 工作流（重要）

1. 反编译：用 MCP 工具 `decompile_ecl` 把 .ecl 转成 .decl 文本
2. 编辑 .decl（UTF-8；原始游戏文本另行处理，保持既有编码不动）
3. **每次修改后**用 `check_ecl` 验证——返回的诊断与 IDE 问题面板同源
4. 通过后用 `compile_ecl` 产出 .ecl
5. 完成一项任务后用 `report_to_user` 向用户汇报做了什么、验证结果如何

## 何时查什么

| 需求 | 途径 |
|---|---|
| 浏览/检索本作全部指令 | 读 references/{{INSTRUCTIONS_FILE}}（可 grep） |
| 单条指令的精确签名 | MCP 工具 `lookup_ecl_semantics`（name 子串或 opcode） |
| 全局寄存器含义 | references/{{REGISTERS_FILE}} |
| 工具链是否可用 | MCP 工具 `get_workspace_info` |

## 接入 thtk-studio MCP（若你的工具未配置）

终端环境变量 `THTK_MCP_URL` / `THTK_MCP_TOKEN` 是 IDE 提供的 MCP 接入点
（也可读项目根 `.mcp.json`）。若 `/mcp` 或工具列表里没有 thtk-studio：

- **codex**：`codex mcp add thtk-studio --url "$THTK_MCP_URL" --bearer-token-env-var THTK_MCP_TOKEN`
- **opencode**：把 thtk-studio 写入项目根 `opencode.json` 的 `mcp` 段
  （`type: "remote"`，url 用 `$THTK_MCP_URL` 的值，
  `headers.Authorization` 用字面量 `Bearer {env:THTK_MCP_TOKEN}`），
  然后提醒用户重启 opencode
- 任何改动都需要**新会话**才生效；完成后用 `report_to_user` 告知用户

## 注意事项

- 本文件由 THTK-Studio 生成，references/ 会随 eclmap 重新生成而刷新；
  SKILL.md 可手工补充项目特有约定（不会被覆盖）。
- 不要手工调用 thecl 命令行——MCP 工具封装了版本号、eclmap 与编码参数。
- 时间标签与难度标记是行级前缀，移动代码时务必一并移动。
