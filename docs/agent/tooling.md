# 编译与检查脚本

## 在 Studio 中工作

项目配置提供默认游戏版本、工具路径、二进制文本编码和项目 map。版本与编码仅供参考，具体操作应按当前任务、文件来源及目标游戏或补丁选择；不要为迁就项目默认值改写脚本或转换编码。未显式指定时，工具使用配置中的默认值。创建或编辑 `.msgspec`、`.endspec`、`.stdspec` 后使用 Studio 的语法检查／编译操作；END 必须选结局格式。编译错误通常会指到行列，先修正第一个有位置的错误，再重试。

项目使用的默认 map、项目 map、源码 `#pragma mapfile` 会共同决定指令名与签名。改动纯台词、时间或坐标一般直接编辑并检查即可；确实遇到参数不匹配时再查该项目的有效定义。

## 在终端中检查

Studio 终端里的 `THTK_CHECK` 指向随包检查器。它也可以在 Studio 关闭后独立运行。先保存文件，再检查一组相关修改：

```sh
"$THTK_CHECK" check scripts/dialogue.msgspec --json
"$THTK_CHECK" check scripts/background.stdspec --json
```

PowerShell 使用：

```powershell
& $env:THTK_CHECK check scripts/dialogue.msgspec --json
```

外部终端可以直接用检查器完整路径代替环境变量；生成包的 `PROJECT.md` 记录当前路径。没有 `--json` 时输出可读诊断。JSON 包含 `context` 和 `result`，前者列出实际版本、工具、编码与命令行 map，后者包含 `success`、诊断和原始编译器信息。检查不保留二进制产物。

检查器从源码目录向上查找 `.thtk-project.json`，也可以显式指定项目：

```sh
"$THTK_CHECK" check scripts/dialogue.msgspec --project . --json
"$THTK_CHECK" check scripts/ending.endspec --kind ending --game 20 --json
```

后缀可推断 kind；`--game` 和 `--encoding` 可覆盖本次检查的版本与二进制编码，`--truth-dir` 指定工具目录。`--map path` 可重复，追加在项目 map 后，源码 pragma 仍在最后生效。相对 pragma 按源码目录解析。

检查未保存内容时，把源码送入标准输入，并使用原文件路径维持相对 map 语义。路径对应的新文件可以尚未创建：

```sh
"$THTK_CHECK" check scripts/new-dialogue.msgspec --stdin --json < draft.msgspec
```

退出码 `0` 表示检查通过，`1` 表示编译器拒绝源码，`2` 表示参数、配置或工具执行失败。MSG／END 原版文本使用 Shift-JIS，汉化游戏按补丁选择 GBK／UTF-8；源文件一直保存为 UTF-8。STD 不使用 MSG 文本编码选项。

## 通过 MCP 编译、反编译与查询

连接 Studio MCP 后，`get_workspace_info` 提供项目根、三类脚本的实际配置、检查器与资料位置。保存好源码后，可用 `run_script` 检查或产出二进制：

```json
{"kind":"msg","operation":"check","input_path":"/project/scripts/dialogue.msgspec"}
```

```json
{"kind":"std","operation":"compile","input_path":"/project/scripts/background.stdspec","output_path":"/project/build/st01.std"}
```

反编译将 `operation` 改为 `decompile`，输入二进制路径；END 必须用 `kind: "ending"`。MSG／END 可以传 `encoding: "gbk"` 等本次操作选项。`source_text` 仅用于检查缓冲区，不要求先保存文件。读取结果中的 `success` 和诊断判断源码是否通过。

需要精确核对项目指令时，使用 `lookup_script_semantics`：

```json
{"kind":"msg","source_path":"/project/scripts/dialogue.msgspec","query":"textPause"}
```

`query` 可以是别名子串或精确 opcode；空字符串用于浏览，`offset`／`limit` 控制分页，默认每页 50 项，按 `nextOffset` 继续读取。结果含有效签名、枚举与 map 来源。可传 `source_text` 查询未保存的 pragma；新文件路径也可以先查询项目定义。生成包的 `msg-instructions.md`、`ending-instructions.md`、`std-instructions.md` 为人工语义参考，按具体目标选择版本章节；项目自定义签名通过查询核对。

任务完成时可用 `report_to_user` 把修改和检查结果放进 Studio 输出面板。

## 直接使用 truth

独立使用 `trumsg`／`trustd` 时，`PROJECT.md` 可提供默认配置参考，检查器 JSON 的 `context` 则列出本次实际使用的配置。按具体目标选择版本、编码和配套 map。例如在源码目录运行：

```sh
trumsg compile dialogue.msgspec -g 18 -m /path/to/msg-18.msgm --encoding shift-jis -o dialogue.msg
trumsg compile ending.endspec --ending -g 18 -m /path/to/ending-18.endm --encoding shift-jis -o ending.msg
trustd compile background.stdspec -g 18 -m /path/to/std-18.stdm -o background.std
```

这里的命令名代表实际工具完整路径；Windows 使用 `.exe`。项目有多份 map 时按原顺序重复 `-m`。把 `compile` 换成 `decompile`，输入二进制并指定对应源码输出路径即可。带相对 pragma 的文件在源码目录运行，以保持目录语义。

## 常见错误

| 反馈 | 处理方式 |
| --- | --- |
| 找不到指令名 | 检查 kind、游戏版本、默认 map 与项目 map；名称可能只存在于某个版本 |
| 参数数量或类型错误 | 先对照版本差异；浮点写 `1.0`，整数写 `1`；TH18 face 的顺序为表情、槽 |
| 字符不能编码 | 选与目标补丁一致的 encoding，或改用该字符集可表达的文字 |
| 找不到 pragma map | Studio 中按源文件目录修正路径；直接调用 truth 时先切到源文件目录 |
| unused script | 检查 `meta.table` 是否引用该脚本，或是否误留旧脚本 |
| 编译通过但图像错误 | 核对 ANM 路径、脚本／sprite 号、平面模式及实例包围区域 |

编译可以确认语法、签名、编码与格式结构；不会加载目标游戏检验资源存在性、气泡排版或 ECL 的实际触发时机。

## 项目辅助包

在 Studio 菜单选择“生成 AI 辅助包”，一次生成 ECL 与 MSG／END／STD 的 skill 和参考资料，无需按格式分别选择。ECL 入口位于 `.claude/skills/ecl-modding/SKILL.md`；MSG／END／STD 入口位于 `.claude/skills/touhou-truth-scripts/SKILL.md`，其他 agent 也可以按这个路径读取。`references/` 导出一份入口、三个格式介绍、三份人工指令参考和项目配置参考，例子内嵌在介绍中；更新辅助包会刷新这些生成内容。已存在的 skill 和项目根 `AGENTS.md` 默认保留，显式选择覆盖只更新 skill。
