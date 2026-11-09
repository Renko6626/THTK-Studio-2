# 工具链平台边界

更新：2026-11-09。MSG/END/STD 已使用 truth；ECL/ANM/DAT 保持 thtk。

| 工具 ID | 程序 | 路径来源 |
| --- | --- | --- |
| thecl | thecl | 现有 thtk 配置 |
| trumsg | trumsg + 同目录 truth-core | 显式 truth 覆盖或随包资源 |
| trustd | trustd + 同目录 truth-core | 显式 truth 覆盖或随包资源 |
| thanm | thanm | 现有 thtk 配置 |
| thdat | thdat | 现有 thtk 配置 |

Rust `common/toolchain.rs` 管理路径、状态和版本。truth 目录优先级为单工具 override > 项目 `toolchain.truthDir` > 应用 `truth_dir` > 随包资源；thtk 保持原有目录设置。truth 使用 `--version`，支持范围来自固定版本的产品基线；thtk 继续使用 `-V` 和 usage 探测。更换手动 truth 时由用户确保版本与扩展兼容。

`modules/truth/compiler.rs` 在后台执行外部进程，设置明确 argv/cwd/map/编码，捕获退出状态与 UTF-8 输出；MSG/STD 领域 command 是薄包装。可定位的 codespan 诊断转换为问题面板行列，其他错误保留原文。MCP 复用此服务。无多后端插件框架，也不回退 thmsg/thstd。

`modules/truth/maps.rs` 解析原生 map 的名称、签名属性、枚举和 intrinsic；`catalog.rs` 选择并释放版本基线，叠加项目 map；`commands.rs` 继续叠加源码 pragma，并提供编辑器和 MCP 查询。编译器采用相同文件顺序。`analysis.rs` 只做源码结构提取，不承担真值编译或游戏仿真。

truth 通过 Tauri `bundle.resources` 分发三个同目录程序、构建记录与许可。构建方法、固定来源和验证状态见 [依赖说明](../deps/README.md)。
