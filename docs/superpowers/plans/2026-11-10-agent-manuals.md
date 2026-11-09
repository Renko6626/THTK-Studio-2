# Agent 人工语义手册实施计划

> **For agentic workers:** 使用 superpowers:executing-plans 在当前已隔离的分支工作树内执行。

**Goal:** 将 agent 的 MSG／END／STD 资料整理为可直接阅读的人工手册。

**Architecture:** 维护 Markdown 原稿，Rust 构建时嵌入七份公开手册，生成辅助包时复制。既有 map、编译器、MCP 和跨边界类型保持原样。

**Tech Stack:** Markdown、Rust，沿用现有 Tauri 构建方式。

**Spec:** `docs/superpowers/specs/2026-11-10-agent-manuals-design.md`

## Global Constraints

- 不新增依赖、语义模型或动态加载机制，不扩大逆向范围。
- 游戏版本和二进制文本编码仅供参考；源码 UTF-8。
- 保护已有用户 skill、AGENTS，不主动删除旧生成文件。
- 依用户约定复用现有测试，不为文案新建测试项目；无浏览器交互测试。

## Review Focus

- 编号和参数必须注明适用世代，未知行为不得推断为已知。
- 默认版本不限制人工参考的导出，项目 map 仍可在精确查询中生效。
- 导出文档的所有相对链接可离线访问。
- 新生成器不再依赖解析项目 map 才能复制手册。
- 保留用户文件的既有测试继续有效。

## Task 1：整理人工文档

- [x] 创建 `docs/agent/{msg,ending,std}-instructions.md`，从已有正文、中文注解和固定 map 校对整理，不保留生成脚本。
- [x] 精简 `docs/agent/{README,msg,ending,std}.md`，完整小例子内嵌；来源与原始示例仍留仓库。
- [x] 调整 `src-tauri/assets/truth-skill-template.md` 和 `docs/README.md` 的阅读入口。

## Task 2：调整辅助包导出及验证

- [x] `src-tauri/build.rs` 只嵌入七份公开手册。
- [x] `src-tauri/src/modules/truth/ai_pack.rs` 删除自动指令表渲染；PROJECT 链接人工参考，保留配置参考与用户文件保护。
- [x] 调整该文件已有测试，检查实际导出手册与保留用户文件的行为。
- [x] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --bin thtk-studio ai_pack::tests`（现有 tauri-env 环境）；核对离线链接和指令编号覆盖，运行现有八份真实示例检查，`git diff --check`。

## 实施记录

- 辅助包既有测试先复现自动表仍导出／上下文未链接人工手册，修改后五项通过。
- 七份导出文档离线链接通过，53 个 profile 的默认 opcode 均在对应世代章节有条目。此检查只确认编号覆盖，不替代人工语义校对。
- 八份既有示例及三份内嵌 TH18 示例均通过随包 truth 实际检查。
- 未运行浏览器交互或目标游戏；未改变编译／MCP 接口；未新增永久测试或依赖。
- 旧导出目录中的冗余文件不主动删除；新包不再导出这些文件。
- 独立审阅确认导出和用户文件保护逻辑；发现并修正 TH095 与 TH10 起缓动模式编号相反的问题，按固定 std-95 map 校对后单独说明 TH095 的 mode／type。

## Task 3：将版本范围放入指令条目（用户进一步确认）

- [x] MSG、STD 每行列出明确适用版本，取消基准表与差异叠加。
- [x] END 的同类版本歧义同步拆行；特定作品的已确认细节与其他仅知签名的版本分开。
- [x] 入口和 skill 明确按任务目标选行，不依项目默认版本或同名签名推断语义。
- [x] 对照 53 份固定 map 检查每个版本的编号覆盖、调用名、参数数量与可核对的参数类型；检查离线链接，运行受影响的既有辅助包测试。

本轮只改人工文档与 skill 文案，不新增语义数据结构、运行时逻辑或永久测试。不扩大逆向调查，编号和签名检查不充作运行行为验收。

Task 3 验证记录：逐版本唯一适用行、默认别名、参数数量与显式参数类型对照通过；七份文档的离线链接通过；`cargo test --manifest-path src-tauri/Cargo.toml --bin thtk-studio modules::truth::ai_pack::tests` 通过。独立审阅发现气泡样式 0–3 外观的范围仍偏宽，核对既有 TH18 记录后将其明确限定到 TH16；未扩展逆向研究。该文案修正后定向重跑导出测试与文档检查。
