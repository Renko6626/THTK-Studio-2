# 后端开发建议

本文档面向 `src-tauri/src/` 下的 Rust / Tauri 宿主代码。

目标是让 Rust 侧继续承担“系统层、工具链层、性能敏感层”，不要被前端拖成一堆零散 command。

---

## 1. 后端职责边界

Rust 侧应该负责：

- 文件系统访问
- 工作区扫描
- 外部工具调用
- 结构化结果返回
- 配置读写
- 长任务管理
- 后续的索引、解析、缓存、诊断

Rust 侧不应该只是：

- 前端随调随写的“工具函数堆”

建议方向：

- command 是对外接口
- `common/`、`modules/`、`domain/` 才是实际逻辑承载层

---

## 2. command 设计建议

当前 command 已经具备基础分层，但后续要继续控制接口质量。

建议：

- command 命名保持清晰和稳定
- 返回结构化结果
- 错误信息明确
- 避免直接把字符串拼接逻辑暴露给前端
- 涉及长时间任务时，优先考虑异步或后台任务模型

对于工具链相关 command，建议统一返回：

- `success`
- `stdout`
- `stderr`
- `exit_code`
- `cwd`
- 结构化错误位置

不要让前端去猜命令输出格式。

---

## 3. 文件系统模块建议

[`src-tauri/src/common/fs_ops.rs`](../src-tauri/src/common/fs_ops.rs) 当前已经承载：

- 创建目录
- 创建文件
- 重命名
- 复制
- 删除

这是合理起点，但后续建议继续补：

- 更统一的错误类型
- 覆盖更完整的路径校验
- 文件冲突策略的统一封装
- 文件监听能力

建议：

- 复制 / 移动 / 删除 等操作逐步从“单 command 单函数”演进为更稳定的文件服务层
- 对路径、冲突、权限、大小写差异处理建立统一策略

---

## 4. 工具链封装建议

当前项目的长期价值不在“能调用一次命令”，而在“把 Touhou 工具链变成稳定内部接口”。

建议：

- `thecl / thdat / thanm / thmsg / thstd` 后续都做成统一封装接口
- 区分：
  - 原始命令执行层
  - 参数构建层
  - 结果解析层
  - 面向前端的结构化返回层

目标：

- 前端只知道“编译 ECL”“反编译 MSG”
- 前端不应该直接知道完整命令拼接细节

---

## 5. 当前后端值得注意的风险

### 5.1 `main.rs` 容易继续堆大

当前 `main.rs` 已经承担 command 注册与应用装配。

建议：

- 继续让 `main.rs` 只做入口与注册
- 新功能优先加到 `common/`、`modules/` 或未来的服务层
- 不要把功能实现直接写回 `main.rs`

### 5.2 终端目前还不是真 PTY

当前终端链路本质上还是“一次性子进程执行”。

这对命令面板足够，但对真正终端不够。

后续如果做真终端，建议：

- 独立 PTY 模块
- Windows 走 ConPTY
- 用事件流推前端
- 不要在现有一次性 command 上继续打补丁

### 5.3 后续需要统一错误模型

现在不少 Rust command 仍然是 `Result<_, String>`。

这是 MVP 阶段能接受的写法，但后续建议：

- 引入更清晰的错误分类
- 至少在内部层统一错误类型
- command 层再决定如何映射为前端可用结果

---

## 6. 后端下一步建议

已完成项：
- ~~文件变更监听~~ ✅（`common/file_watcher.rs`，基于 notify + notify-debouncer-mini，500ms 防抖，通过 Tauri emit 推送前端）
- ~~构建输出 / 错误定位接口~~ ✅（`compiler.rs` 诊断路径自动规范化为绝对路径）
- ~~文件树懒加载~~ ✅（`fs_utils.rs` 改为浅层扫描 + `get_dir_children` 按需加载）

当前建议优先顺序：

1. 消除 `config.rs` 中的 unwrap/expect 调用（3 处 Mutex + 1 处 ProjectDirs）
2. 引入 `tracing` 结构化日志替代 println
3. thmsg/thstd/thanm 工具链封装（复用 thecl 的 ToolOperationResult 模式）
4. 终端 PTY 架构
5. 索引与诊断服务层

### 当前 common/ 模块清单

| 模块 | 职责 |
|------|------|
| cmd_runner.rs | 外部工具进程执行（Windows 隐藏控制台 + Shift-JIS 解码） |
| file_watcher.rs | 文件系统变更监听（notify + debounce + Tauri 事件） |
| fs_utils.rs | 文件树浅层扫描 + 按需加载子目录 + 文件分类 |
| fs_ops.rs | 文件 CRUD 操作（创建/重命名/删除/复制） |
| toolchain.rs | 工具路径解析 + 版本检测（5 个工具） |
| terminal.rs | Shell 命令执行（cmd/PowerShell） |
| system_clipboard.rs | Windows 系统文件剪贴板（via PowerShell） |

---

## 7. 后端编码约束

建议继续遵守：

- 重逻辑尽量留在 Rust
- command 只是边界层
- 错误显式处理
- 长任务不要阻塞 UI
- 返回结构化数据，不让前端做二次猜测
