# THTK-Studio Docs

这个目录用于放开发辅助文档，而不是用户使用说明。

当前文档分为八类：

- [frontend-guidelines.md](./frontend-guidelines.md)
  - 前端结构、状态管理、组件边界、Monaco 集成建议
- [backend-guidelines.md](./backend-guidelines.md)
  - Rust / Tauri 宿主、command、文件系统与工具链封装建议
- [review-notes.md](./review-notes.md)
  - 当前代码审阅结论、风险点、建议优先修复项
- [thecl-integration.md](./thecl-integration.md)
  - `thecl` 官方用法对应的 IDE 接入设计
- [toolchain-platform.md](./toolchain-platform.md)
  - `thtk` 工具链注册表、通用设置、状态检测和后续扩展约束
- [toolchain-testing-plan.md](./toolchain-testing-plan.md)
  - `thecl` 真实样本回归和静态检查并行的测试计划
- [ecl-highlighting-plan.md](./ecl-highlighting-plan.md)
  - ECL 语法高亮、静态 marker 检查、`eclm` 动态语义层与分阶段实施方案
- [ecl-signature-inference.md](./ecl-signature-inference.md)
  - 基于 `eclm` 和真实 .decl 样本，对 `!ins_signatures` 缩写含义的当前推断

## ECL 教程整理

基于 `docs/ecl-tutorial-source` 的原始教程，整理了三份更适合开发和查阅的中文精简文档：

- [ecl-tutorial-overview-cn.md](./ecl-tutorial-overview-cn.md)
  - ECL 入门、工具链工作流、运行模型和对 IDE 的直接启发
- [ecl-syntax-cheatsheet-cn.md](./ecl-syntax-cheatsheet-cn.md)
  - ECL 常用语法、变量、控制流、栈和预处理速查
- [ecl-patterns-and-organization-cn.md](./ecl-patterns-and-organization-cn.md)
  - 敌机生成、弹幕管理器、坐标移动、多文件组织和 MERLIN 库摘要
- [ecl-tutorial-source/README.md](./ecl-tutorial-source/README.md)
  - Priw8 ECL 教程原始抓取稿与分页来源

另外，根目录下还有一份阶段性状态文档：

- [../editor-shell-status.md](../editor-shell-status.md)
  - 当前 IDE 壳子已经做到哪一步、还缺哪些基础能力

建议维护方式：

- 面向长期架构的约束，优先写进 `frontend-guidelines.md` 或 `backend-guidelines.md`
- 面向当前阶段的阶段性问题，优先写进 `review-notes.md`
- 面向“当前壳子完成度”的判断，优先更新根目录下的 `editor-shell-status.md`
- 面向工具链平台层的共性约束，优先更新 `toolchain-platform.md`
- 面向脚本语言理解和编辑器支持的知识整理，优先更新这些 ECL 中文整理稿和高亮计划
- 功能规划和长期路线仍以根目录下的 [project.md](../project.md) 为准
