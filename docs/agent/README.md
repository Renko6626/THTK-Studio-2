# MSG、END、STD 编程

这三类脚本使用 truth 原生源码。按任务选一份介绍，再用对应的人工指令参考查询编号、作用和参数。

| 任务 | 编程介绍 | 指令参考 |
| --- | --- | --- |
| 关卡台词、立绘、气泡和 ECL 对话配合 | [MSG](msg.md) | [MSG 指令](msg-instructions.md) |
| 结局文字、CG、音乐和 staff | [END](ending.md) | [END 指令](ending-instructions.md) |
| 背景模型、相机、雾和循环 | [STD](std.md) | [STD 指令](std-instructions.md) |

介绍包含文件结构、基本语法和完整小例子。指令参考人工维护，每条直接列出适用版本，按目标版本选行；项目自定义 map 的精确名称和签名可用现有 MCP 查询。

## 版本、编码与检查

项目的 `references/PROJECT.md` 记录生成时的默认配置和工具位置，版本与二进制文本编码仅供参考。按当前任务、文件来源及目标游戏或补丁选择，不要为迁就默认值改写脚本或转换编码。源码始终 UTF-8；MSG／END 二进制可按目标选择 Shift-JIS、GBK 或 UTF-8，切换编码不会给游戏增加字库或解码能力。STD 不使用 MSG 文本编码选项。

保存一组修改后检查：

```sh
"$THTK_CHECK" check scripts/dialogue.msgspec --json
"$THTK_CHECK" check scripts/ending.endspec --game 20 --encoding gbk --json
"$THTK_CHECK" check scripts/background.stdspec --json
```

PowerShell 用 `& $env:THTK_CHECK check ...`。`--game`、`--encoding` 指定本次检查目标；不指定时工具使用配置默认值。检查器从源码目录向上查找 `.thtk-project.json`，也可传 `--project DIR`、`--truth-dir DIR`；`--map FILE` 可重复，追加到项目 map 后。相对 pragma 按源码目录解析。`--stdin` 可检查未保存内容，同时保留原文件路径作为相对路径基准。

JSON 的 `context` 是本次实际配置，`result.success` 和诊断说明源码是否通过。退出码 0 为通过，1 为源码错误，2 为参数、配置或工具执行失败。检查不保留二进制产物。检查器可以在 Studio 关闭后独立运行。

Studio MCP 的 `run_script` 支持 `check`、`compile`、`decompile`；`kind` 用 `msg`、`ending` 或 `std`。仅 check 接受 `source_text`，未保存文件的路径仍用于解析相对 pragma。`lookup_script_semantics` 可查询项目 map 与文件 pragma 下的签名，`query` 为名称子串或 opcode，分页按 `nextOffset` 继续。MCP 查询与执行使用当前工具配置，目标版本与该配置不一致时先对齐配置，或在 CLI 显式指定目标，避免用错误版本检查后修改脚本。

直接编译的例子：

```sh
trumsg compile dialogue.msgspec -g 18 -m /path/to/msg-18.msgm --encoding shift-jis -o dialogue.msg
trumsg compile ending.endspec --ending -g 18 -m /path/to/ending-18.endm --encoding shift-jis -o ending.msg
trustd compile background.stdspec -g 18 -m /path/to/std-18.stdm -o background.std
```

命令使用实际工具路径；Windows 带 `.exe`。多份 map 按顺序重复 `-m`，源码 pragma 最后覆盖；带相对 pragma 的文件先切到源码目录再调用 truth。Studio 中也可执行检查、编译和反编译；END 必须选结局类型。编译通过确认语法、签名、编码与结构，资源和演出效果仍由目标游戏运行确认。
