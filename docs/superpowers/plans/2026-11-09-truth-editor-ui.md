# truth 编辑器接入计划

设计：../specs/2026-11-09-truth-editor-ui-design.md。用户已确认缓冲区自动检查及分类构建入口。

1. Rust `truth/check.rs`：同目录临时源码、临时二进制、清理与诊断路径映射；Tauri/TS 接口。真实 CLI 测试未保存错误、相对 map、原文件不变及无残留。
2. types/registry/buildDialog/actions：增加 STD 表单及输出路径；二进制页 MSG 类型/编码/高级选项，STD 高级选项；源码操作栏共用 actions。
3. truth Monaco providers：调用着色、片段补全、嵌套/字符串参数识别、符号/引用；复用 Rust analysis，右栏按语言展示。
4. Monaco 诊断：停顿 800ms 检查当前缓冲区，版本校验丢弃旧结果；问题面板与标记、手动检查、错误状态；不自动保存原文件。
5. 运行相关既有测试、typecheck/build/Rust 检查。新增测试对应真实关键行为；不做浏览器自动交互，不扩展 Windows 全量打包。

## 实施结果

- 已实现 check_truth_source；800ms 停顿自动检查、手动检查、旧响应丢弃、关闭时清理问题、状态栏失败信息。
- MSG 二进制页 kind/encoding/高级选项、STD 表单/输出路径、分类菜单、源码检查/打包/高级选项均接入既有 actions。
- 增强调用与标签着色、参数片段补全、枚举/布尔/常量/标签补全、嵌套参数识别、DocumentSymbol、标签定义和引用；右栏展示脚本/常量/标签与标签引用。仍沿用基础结构提取，不声称完整 AST。
- 验证：npm run typecheck、npm test（120 passed）、npm run build、tauri-env linux cargo test（176 passed / 2 ignored）、显式 buffer_check_preserves 真实 CLI 用例、补充 truth_outline_preserves wire/结构用例、git diff --check 均通过。新增 wire 用例在全量测试之后单独通过。
- 真实检查用例验证相对 map、GBK 未保存缓冲区、错误行号、原文件不变、临时文件无残留；前端用例验证停顿、过期结果、关闭清理和 STD 自定义输出路径/错误呈现。
- 未做浏览器交互、Windows 实机验收或本轮 Windows 安装包重建；上一阶段安装包不包含本轮改动。
