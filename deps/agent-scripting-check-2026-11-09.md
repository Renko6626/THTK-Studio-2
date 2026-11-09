# Agent 脚本编程支持实施与验证

日期：2026-11-09。实现分支：`codex/agent-scripting`。固定 truth、编码补丁与现有 53 份 map 内容未变。

## 交付

- `docs/agent/` 提供 MSG、END、STD 的编程手册、共用语法、版本差异、来源和八份完整例子。正文给明确用法；研究过程与未知项集中在来源附录。
- `crates/script-check/` 提供不依赖 Tauri／WebView 的共享调用内核及 `thtk-check`。IDE、MCP 和 CLI 共用版本、编码、map、truth 调用及诊断处理。
- MCP 保留现有工具，补充独立脚本上下文、未保存源码检查、新文件语义查询与完整分页。
- 辅助包导出离线文档、项目实际配置、完整有效指令表和示例。保留用户已有 SKILL／AGENTS；显式覆盖只作用于 skill。
- 构建钩子准备随包 CLI，终端通过 `THTK_CHECK` 获取路径。CLI 可以在 Studio 未启动时使用。

## 实际编程练习

在没有 DISPLAY／WAYLAND_DISPLAY 的环境中，以同一个临时 TH18 项目调用真实检查器和 truth：

| 操作 | 结果 |
| --- | --- |
| 修改 MSG 台词为 GBK 中文并调整阅读等待 | 通过 |
| 从头写一份有文字与等待的 END | 通过 |
| 增加 STD 实例并修改相机循环时长 | 通过 |
| 给 textPause 传入字符串，再根据诊断修复 | 先返回源码错误，定位原文件第 13 行；修复后通过 |
| 未创建文件，以标准输入检查源码，叠加项目 map 和相对 pragma | 通过；未创建原文件，检查临时文件已清理 |

这验证了普通文件编辑加文档、查询和检查的流程；没有建立额外的 agent 运行或评测平台。

## 验证命令与结果

以下命令在实施工作树运行；后端检查使用已有 `tauri-env linux` 环境。

```bash
THTK_TEST_TRUTH_DIR="$PWD/src-tauri/resources/truth" \
  cargo test --manifest-path crates/script-check/Cargo.toml -- --include-ignored
```

通过。包含真实 MSG／END GBK 编解码、STD、map／pragma 顺序、未保存和新文件、原路径诊断、CLI JSON 与退出码。共享 crate 的依赖树没有 Tauri、GTK、WebView、rmcp、tokio 或 truth Rust 库。

```bash
CARGO_TARGET_DIR=/data/sunyunbo/www/THTK-Studio-2/src-tauri/target \
  /data/sunyunbo/bin/tauri-env linux cargo test \
  --manifest-path src-tauri/Cargo.toml --bin thtk-studio
npm run typecheck
npm test
npm run build
```

后端现有全套 189 项通过；TypeScript 检查通过；前端 26 个文件、140 项测试通过；前端构建通过。另分别验证 MCP schema／分页／上下文、辅助包生成与用户文件保护、wire 格式及原有 PTY 环境注入。

```bash
node scripts/check-agent-examples.mjs \
  --checker crates/script-check/target/debug/thtk-check \
  --truth-dir src-tauri/resources/truth
```

八份示例全部通过真实检查器；此前也分别通过原生 truth 编译。

```bash
node scripts/build-script-check.mjs
/data/sunyunbo/bin/tauri-env win node scripts/build-script-check.mjs x86_64-pc-windows-msvc
CARGO_TARGET_DIR=/data/sunyunbo/www/THTK-Studio-2/src-tauri/target \
  /data/sunyunbo/bin/tauri-env win cargo xwin check \
  --manifest-path src-tauri/Cargo.toml --release --target x86_64-pc-windows-msvc
```

host 构建与 Windows 交叉构建通过。Windows 检查器确认为 PE32+ console／Windows CUI；最终暂存目录已恢复 host 程序。host 暂存程序在无图形环境、没有现存应用设置的情况下，通过可执行文件相邻的 truth 目录完成检查，没有创建设置目录。

## 审阅与修正

Sol 子代理分别实施资料、共享内核和 MCP／辅助包；独立文档审阅与集成审阅已完成。文档审阅的两处矩阵措辞已修正。

集成审阅复现了一处退出码问题：缺少配置 map 被当作源码错误。修正 `ab82c8f` 对已经解析的命令行／项目 map 做读取检查，使用现有配置错误通道返回退出码 2，保留上下文和可读错误。源码 pragma 仍由 truth 原生处理，没有新增 parser 或恢复机制。

新增的断言先复现退出码 1／预期 2；修正后只重跑受影响的 CLI 与真实工具用例（1＋3）并通过，重建变更后的 Windows／host 检查器。上述全套后端／前端结果来自该窄修正之前；没有因此重复无关检查。修正差异经定向复核，问题已解决，没有新增阻断项。

## 未执行项

未运行浏览器自动交互、Windows 程序／安装包或目标游戏。编译和检查结果不替代游戏内的资源、演出与时序验收。既有 Monaco sourcemap、构建 chunk 大小及后端未使用 helper 警告未在本任务中扩展修复。
