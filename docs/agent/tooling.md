# 编译与检查脚本

## 在 Studio 中工作

项目配置决定游戏版本、工具路径、编码和项目 map。创建或编辑 `.msgspec`、`.endspec`、`.stdspec` 后使用 Studio 的语法检查／编译操作；END 必须选结局格式。编译错误通常会指到行列，先修正第一个有位置的错误，再重试。

项目使用的默认 map、项目 map、源码 `#pragma mapfile` 会共同决定指令名与签名。改动纯台词、时间或坐标一般直接编辑并检查即可；确实遇到参数不匹配时再查该项目的有效定义。

## 直接调用随包 truth

在仓库根目录执行以下命令。为了使 map 路径在不同当前目录下都明确，先得到绝对根路径：

```sh
REPO_ROOT="$(pwd)"
TRUTH_BIN="$REPO_ROOT/src-tauri/resources/truth"
MAP_DIR="$REPO_ROOT/src-tauri/assets/maps/truth/defaults"

"$TRUTH_BIN/trumsg" compile docs/agent/examples/th18-dialogue.msgspec \
  -g 18 -m "$MAP_DIR/msg-18.msgm" --encoding shift-jis \
  -o /tmp/th18-dialogue.msg

"$TRUTH_BIN/trumsg" compile docs/agent/examples/th18-ending.endspec \
  --ending -g 18 -m "$MAP_DIR/ending-18.endm" --encoding shift-jis \
  -o /tmp/th18-ending.msg

"$TRUTH_BIN/trustd" compile docs/agent/examples/th18-background.stdspec \
  -g 18 -m "$MAP_DIR/std-18.stdm" -o /tmp/th18-background.std
```

Windows 使用对应 `.exe`。TH20 END 要同时更换 `-g 20` 和 `ending-20.endm`；其他例子按 [manifest](examples/manifest.json) 选择格式、版本和编码。STD 不接收 MSG 的 `--encoding` 参数。

反编译：

```sh
"$TRUTH_BIN/trumsg" decompile /tmp/th18-ending.msg \
  --ending -g 18 -m "$MAP_DIR/ending-18.endm" --encoding shift-jis \
  -o /tmp/th18-ending.endspec
"$TRUTH_BIN/trustd" decompile /tmp/th18-background.std \
  -g 18 -m "$MAP_DIR/std-18.stdm" -o /tmp/th18-background.stdspec
```

二进制输入的编码要与生成时一致。原版使用 Shift-JIS；GBK／UTF-8 对应目标游戏的文字补丁。源文件仍保存为 UTF-8。map 和 truth 版本也应一致，避免把同一 opcode 解成另一套参数。

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

## 共享检查器与 MCP 的交付状态

本轮计划增加 `thtk-check` headless CLI，并让 IDE／MCP 复用同一检查实现。具体命令参数和新增 MCP 字段等待实现接入后同步到本节；当前可直接使用上面的真实 truth 命令。现有 MCP 提供 `get_workspace_info`、`run_script`、`lookup_script_semantics`、`report_to_user`，按当前工具 schema 调用。

项目辅助资料沿用 `.claude/skills/touhou-truth-scripts/` 路径，其他 agent 可显式读取该目录的 `SKILL.md`。辅助包更新、签名导出和项目上下文由 Studio 的生成逻辑处理，手册本身可离线阅读。
