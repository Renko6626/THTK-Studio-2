---
name: touhou-truth-scripts
description: 编写或修改东方 MSG 对话、END 结局和 STD 背景脚本时使用，提供编程介绍、人工指令语义参考和检查方式。
---

# 编写 MSG、END、STD

按任务阅读一份介绍和对应的指令参考：

| 格式 | 编程介绍 | 指令参考 |
| --- | --- | --- |
| 关卡对话 MSG | [MSG](references/msg.md) | [MSG 指令](references/msg-instructions.md) |
| 结局／staff END | [END](references/ending.md) | [END 指令](references/ending-instructions.md) |
| 背景 STD | [STD](references/std.md) | [STD 指令](references/std-instructions.md) |

介绍包含语法和完整小例子；指令参考人工维护，每行明确适用版本、编号、作用、参数类型和含义。先确定当前任务目标，再选明确列出的版本行；未列版本不继承同名指令的行为，签名相同也不证明语义相同。无需先查询工具才能写每条指令。项目自定义 map 与手册不一致时，再用 `lookup_script_semantics` 核对；查询新文件或未保存源码可传 `source_path`、`source_text`，分页按 `nextOffset` 继续。

[PROJECT.md](references/PROJECT.md) 只是生成时默认配置参考。版本和 MSG／END 二进制文本编码按当前任务、文件来源及目标游戏或补丁选择，不要为迁就默认值改写脚本或转换编码。源码始终 UTF-8：`.msgspec`、`.endspec`、`.stdspec`，使用普通文件工具读写。

Studio 新建终端自动设置 `THTK_CHECK`；外部终端先复制 [PROJECT.md](references/PROJECT.md) 中“外部终端首次使用”的完整命令，为当前会话设置检查器路径并用 `--help` 确认可执行。

完成一组编辑后检查：`"$THTK_CHECK" check /path/to/script.msgspec --json`，需要时显式传 `--game` 和 `--encoding`。检查器可在 Studio 关闭后独立运行；完整命令及 map 用法见 [入口](references/README.md#版本编码与检查)。

Studio MCP 也可使用 `run_script`：`kind` 为 `msg`／`ending`／`std`，`operation` 为 `check`／`compile`／`decompile`，`input_path` 为文件路径。仅 check 接受 `source_text` 检查未保存内容，路径仍用于解析相对 pragma。读取 `success`、`diagnostics` 和 `message` 判断结果。MCP 使用当前工具配置，检查前确认它与本次目标一致；CLI 可显式指定版本和编码。

检查通过后按需要编译。编译确认语法、签名、编码和结构，资源与游戏内效果仍由目标游戏确认。任务完成可用 `report_to_user` 汇报。

辅助包更新会刷新导出的 `references/`，默认保留手改的 SKILL.md 和项目根 AGENTS.md。自定义约定放在这两个文件中。
