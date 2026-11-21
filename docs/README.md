# THTK-Studio 文档

[项目路线图](../project.md)记录长期方向和后续工作，[项目首页](../README.md)记录当前可用功能。

## ECL

- [TH16–18 映射研究状态](ecl/ecl-mapping-research-status.md)汇总原版 ECL 可用线，链接各作指令表、可靠度 CSV 和后续核查清单。
- 编写脚本可查[入门与工作流](ecl/ecl-tutorial-overview-cn.md)、[语法速查](ecl/ecl-syntax-cheatsheet-cn.md)、[弹幕与脚本组织](ecl/ecl-patterns-and-organization-cn.md)以及 [Priw8 教程原稿](ecl/ecl-tutorial-source/README.md)。
- 编辑器侧的研究见[语法高亮与静态检查](ecl/ecl-highlighting-plan.md)和 [ECLM 签名推断](ecl/ecl-signature-inference.md)。

## 架构与工具链

- [前端开发约定](architecture/frontend-guidelines.md)与[后端开发约定](architecture/backend-guidelines.md)涵盖 Vue、Monaco、Rust 和 Tauri 的职责。
- 各格式的实现进度见[脚本支持现状](toolchains/script-support-status.md)。工具的选择、注册和调用见[工具链生态](toolchains/toolchain-ecosystem.md)、[平台约定](toolchains/toolchain-platform.md)和 [thecl 集成](toolchains/thecl-integration.md)。
- MSG、END、STD 的当前接口见 [truth 迁移设计](superpowers/specs/2026-11-09-truth-msg-std-migration-design.md)和 [GBK 编码扩展](superpowers/specs/2026-11-09-truth-msg-encoding-design.md)。固定版本、构建与分发见[依赖说明](../deps/README.md)。
- 引擎逆向在独立的 renkolab 仓库维护。SHT 的格式和工具研究在 `engine/sht/`，MSG 在 `engine/msg/`，各子系统的研究入口在对应目录的 `OVERVIEW.md`。

## 设计与历史记录

[设计规格](superpowers/specs/)与[实施计划](superpowers/plans/)按日期保存。[阶段记录](history/)收录早期的[编辑器完成度](history/editor-shell-status.md)、[项目审阅](history/review-notes.md)、[文件树审计](history/file-tree-audit-2026-06-07.md)和[依赖管理计划](history/dependency-management-plan-2026-10-23.md)等资料。
