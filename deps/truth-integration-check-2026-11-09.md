# truth 集成验证记录

日期：2026-11-09。代码已切换 MSG/END/STD；没有提交或发布上游分支。

## 实际实现

- `modules/truth` 负责原生 map、有效版本/项目/pragma 查询、CLI、UTF-8 诊断和基础源码结构。MSG/STD command、Monaco 与 MCP 共用这些服务。
- 旧 `.dmsg/.dstd` 翻译器、thmsg 签名修补、STD 参数交换、原始导出入口及无消费者的旧 map 解析辅助已删除。源码扩展名改为 `.msgspec/.endspec/.stdspec`。未删除用户文件。
- 随包工具固定 zero318/truth c78a541，编码补丁只作用于 MSG/END 二进制字符串；构建清单记录平台与各文件 SHA-256。
- ECL/ANM/DAT 后端保持原路线；共享文件文本读写与 ECL 编码函数保留。

## 已运行检查

| 检查 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm test` | 15 个文件、115 项通过 |
| `npm run build` | 通过；仍有既有大 chunk 提示 |
| `tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml` | 176 passed / 1 ignored；忽略项需要准备平台对应 truth，单独执行 |
| `tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml bundled_tools_share_maps -- --ignored` | 1 项通过；真实 CLI 编译/反编译、GBK、覆盖查询、诊断及 MCP check/lookup |
| 干净固定检出应用 patch，Rust 1.85.1 `test --release --locked --test msg_encoding` | 通过；stage/ending GBK 汉字字节、往返、默认 Shift-JIS 与不可编码字符拒绝 |
| truth `--test integration integration::msg_features` | 10 项通过 |
| truth `--test integration integration::strings` | 16 通过，1 个 ANM 断言失败；未打补丁基线同样失败，见下文 |
| 53 份默认 map 真实 CLI 最小源码编译 | 全部通过；早期 STD 使用对应 meta 与三参数 opcode0 |
| TH18 `st01a.msg`、`e01.msg`、`st01.std` 真实文件 | 新默认 map 反编译后重编译字节一致 |
| 移动工具组到含空格目录，源/输出路径也含空格 | 三类 CLI 往返通过；不依赖 PATH 上的 truth/thtk |
| TH143/165 `tutorial(1)` 与修正后的默认 map | 两作真实编译通过 |
| `git diff --check` | 通过 |

真实游戏样本仅保存在本机临时目录，未放入仓库。原生集成用例使用自建最小源码，覆盖项目 map→源码 pragma 的别名/签名查询、GBK、指定输出路径、诊断行号与禁止覆盖输入；MCP check/lookup 也通过生产服务验证。

Windows 包构建后已恢复 Linux staging，程序 hash 与来源清单一致；最终原生集成测试在恢复后的工具上运行通过。

测试数量减少来自删除旧方言及不再被生产代码使用的辅助模块测试，不代表删掉失败用例来通过检查。

## 审查修正

独立审查发现两处实际语义错误，回归用例先失败再通过：

1. TH10 从 opcode9 起与 TH11 编号不同；中文说明按对应操作对齐，TH19/20 MSG 和 TH20 END 选独立资料。
2. TH143/165 opcode33 来自 truth 的 `th11-scene.msgm`，命名为 `tutorial`，不再沿用 `portraitDarken` 或暗化立绘说明。

## 上游已存在的回归失败

`integration::strings::decompile_string_reg` 属于 ANM：测试预期反编译带 register bit 的字符串报错，固定 zero318 基线实际反编译成功。补丁版本和完全撤销补丁的同提交版本都在相同断言失败。没有修改、忽略或放宽上游用例；本次 ANM 仍走 thtk，不扩大 truth 补丁范围。

## 平台与交付

Linux x86_64、Windows x86_64 的 `truth-core`、`trumsg`、`trustd` 均成功构建，hash 已写入 `manifest.source.json` 的 artifacts。Windows Studio 主程序交叉编译成功。

首次 NSIS 打包在下载 `nsis_tauri_utils.dll` 时遇到代理的 `http response missing version`；改用独立临时缓存补齐官方插件后成功生成 NSIS 安装包。安装脚本确认三个 truth 程序同目录，并包含 LICENSE、NOTICE、crate 许可与 build-info。

产物：`src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/thtk-studio_0.1.0_x64-setup.exe`，7,109,177 字节，SHA-256 `67f83091fbd1b96e8e5e4ed04f61a923c956acb1774307fb83b4cb3a78cc73e2`。此包未签名，尚未在 Windows 实机安装运行。

## 尚未验收与已知辅助限制

- 未做 Windows 实机安装、卸载、编辑器交互或游戏内效果验收；未跑浏览器自动点击/输入。
- MCP 新入口复用现有传输，验证到生产工具服务；未在本轮另起真实客户端跑一遍 HTTP 端到端会话。
- 没有对全部作品使用真实游戏文件验收；版本 map 编译通过与游戏实际效果分开记录。
- `bool` 参数提示存在，但内置 `true/false` 未补进枚举查询表。
- 时序提取按行工作，复杂同一行标签和十六进制时间等语法未完整解析；不可当作完整 AST 或游戏仿真。
- END 选项未按作品过滤；选择不支持的作品时后端明确拒绝。
