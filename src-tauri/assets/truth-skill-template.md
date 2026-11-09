---
name: touhou-truth-scripts
description: 编写或修改东方 MSG 对话、END 结局和 STD 背景脚本时使用，提供 truth 语法、完整项目指令表、可编译示例与检查命令。
---

# 编写 MSG、END、STD

从 [编程入口](references/README.md) 和 [当前项目](references/PROJECT.md) 了解目标版本、编码和工具位置。按任务阅读 [MSG](references/msg.md)、[END](references/ending.md) 或 [STD](references/std.md)，共用语法见 [truth-syntax](references/truth-syntax.md)。[示例清单](references/examples/manifest.json) 提供可直接改写的源码。

源码是 UTF-8 的 truth 原生文本：关卡对话 `.msgspec`，结局／staff `.endspec`，背景 `.stdspec`。用普通文件工具阅读和编辑。项目指令表包含完整名称、参数和枚举；文件的 `#pragma mapfile` 可以进一步覆盖定义，需要精确项目签名时再查询 MCP。

例如查询新文件或未保存源码中的文本指令：

```json
{"kind":"msg","source_path":"/path/to/project/dialog.msgspec","source_text":"#pragma mapfile \"custom.msgm\"\n","query":"text","offset":0,"limit":50}
```

这是 `lookup_script_semantics` 的参数。返回 `total` 和 `nextOffset`；继续用 `nextOffset` 可遍历完整指令，枚举每页完整返回。

完成一组相关编辑后检查，再按需要编译。终端的 `thtk-check` 用法见 [tooling](references/tooling.md)；检查器位置见 PROJECT.md，也可由 `THTK_CHECK` 指定。Studio MCP 检查磁盘文件：

```json
{"kind":"msg","operation":"check","input_path":"/path/to/project/dialog.msgspec"}
```

检查未保存内容时加 `source_text`，路径继续用于解析相对 pragma。只有 check 接受该参数；检查不会改写源文件。结果的 `success` 表示源码是否通过，失败时阅读 `diagnostics` 和 `message` 修正。`compile` 生成二进制，`decompile` 生成原生源码；二者可指定 `output_path`。MSG/END 的 `encoding` 指二进制文本编码，默认沿用项目设置；源码始终保持 UTF-8。

保留目标版本和项目 map。STD 使用符号标签，由编译器重算跳转偏移。编译通过后，画面、节奏和游戏侧效果仍由游戏运行确认。

`references/`（包含 examples、PROJECT.md 和 instructions）是生成内容，重新生成辅助包时刷新。此 SKILL.md 默认保留手改内容；项目根 AGENTS.md 始终保留。
