# THTK-Studio

THTK-Studio 是一个面向东方 Project 脚本与资源魔改工作流的桌面 IDE，目标是围绕 `ECL / ANM / MSG / STD` 构建专用开发环境，而不是做一个通用文本编辑器。

当前仓库基于：

- Tauri v2
- Rust
- Vue 3
- Monaco Editor
- Naive UI

## 项目定位

项目的长期方向以 [project.md](./project.md) 为准：

- 先完成项目工作流闭环
- 再补语言服务
- 再做预览能力
- 最后考虑 AI 集成

## 当前开发进度

项目处于“第一阶段 MVP 工程闭环”的收尾期。ECL 的完整工作流已经闭环，MSG / STD / DAT 已具备基础编译、反编译或容器操作流程，真正的 PTY 终端、进程内 MCP Agent 通道与项目配置对话框也已落地。当前首要缺口是最近项目 / 欢迎流程、项目切换保护，以及尚未实现的 ANM 工具链。

### 已完成

- Tauri + Vue 3 桌面应用骨架
- Rust 侧 `common / modules / config / state` 分层
- 文件树懒加载（只加载当前层级，展开时按需加载子目录）
- 文件树操作：新建、重命名、删除、刷新、多选、剪切/复制/粘贴、拖拽移动、复制路径
- 文件树展开状态持久化（按项目保存，切换项目自动恢复）
- 外部文件变更检测（Rust notify 监听，自动重载或提示用户）
- Monaco Editor 多标签编辑、脏状态跟踪、Ctrl+S 保存
- 当前文件内查找 / 替换
- 工作区和标签页会话恢复、刷新保护、关闭前未保存保护
- VS Code 风格工作台布局（文件管理器 / 菜单栏 / 标签栏 / 编辑区 / 面板 / 右侧边栏）
- Rust 文本读写与 Shift-JIS / UTF-8 处理
- 统一 thecl 请求模型：ECL 编译 / 反编译 / header 生成
- thecl 错误输出解析 → Monaco 诊断标记（波浪线覆盖单词或行尾）
- 诊断路径自动规范化（Rust 端将相对路径解析为绝对路径）
- 应用级配置：thtk 路径、默认游戏版本
- 图形化构建配置弹窗（模式/版本/输出路径/thecl 选项）
- 输出面板（按任务分组）/ 问题面板（点击跳转到源码位置）
- `.ecl` 二进制文件专用工作区视图
- ECL 语言支持：语法高亮、eclmap 语义数据、补全、悬停、转到定义、引用查找、签名帮助、文档符号、静态诊断
- MSG 基础工作流：`.msg` ↔ `.dmsg`、Shift-JIS / UTF-8 桥接、指令名翻译
- STD 基础工作流：`.std` ↔ `.dstd`、指令名翻译、`jmp` 参数顺序适配
- DAT 基础工作流：容器解包与目录打包
- 真正内嵌终端：xterm.js + portable-pty，多会话、流式输出与 resize
- 进程内 MCP Agent 通道：ECL 检查 / 编译 / 反编译 / 语义查询与客户端自动接线
- 项目配置：`.thtk-project.json` 三态读写（不存在 / 有效 / 损坏）、原子保存与项目设置对话框
- 项目级工具链覆盖：项目配置的 `toolchain.thtkDir` 对 ECL / MSG / STD / DAT / MCP 全部生效
- 文件系统安全加固：项目路径守卫、文件名校验、符号链接 / reparse point 防护与严格 CSP

### 已有但尚未闭环

- 工作区视图体系仍只有 `text / binary-script` 两种；MSG / STD 复用文本视图，ANM 尚无专用视图
- MSG / STD 已有基础工具链流程，但没有 Monaco 语言服务、结构化诊断或 MCP 工具
- 项目打开仍是"设根目录 → 取文件树 → 读配置"三步，失败时可能留下半个状态

### 尚未完成

- 最近项目 / 欢迎页
- 项目切换保护（脏标签确认、失效路径处理）
- ANM 工具链、文本编辑层与 sprite / 动画预览
- MSG / STD 语言服务与结构化诊断
- 全局搜索、索引、资源引用分析
- 时间线 / MSG 等领域预览
- 前端 TypeScript 迁移、测试与类型检查门禁

## 阶段进度

按 `project.md` 的阶段划分：

1. **第一阶段 MVP 工程闭环**：接近完成（ECL / MSG / STD / DAT 基础工作流、文件管理、PTY 与项目配置已完成；项目入口与切换保护待收尾）
2. 第二阶段 语言服务 MVP：ECL 基础语言服务已有，MSG / STD / ANM 待实现
3. 第三阶段 可视化预览：未开始
4. 第四阶段 高级工程能力：未开始
5. 第五阶段 AI 集成：Agent 通道与 ECL 辅助包已提前实现，领域能力仍需扩展

## 代码结构

```text
src/
  api/            前端到 Tauri command 的桥接
  components/     编辑器、侧边栏、对话框、工具面板
  composables/    工作台行为、文件树交互、工具链动作
  services/       工作区视图、终端运行时、工具链元数据、ECL 语言服务
  stores/         Pinia 状态管理
  utils/          图标、前端辅助函数

src-tauri/src/
  main.rs         Tauri 桌面入口与 command 注册
  app_state.rs    全局状态（配置、项目根、文件监听、PTY、MCP）
  config.rs       应用配置读写
  common/         文件系统、PTY、项目配置、文件监听、工具链等通用能力
  modules/        ECL / MSG / STD / DAT 工具链与 MCP server
```

## 下一阶段建议

1. 完成欢迎页 / 最近项目与安全的项目切换流程（事务式 `open_project`、脏标签确认）
2. 建立前端测试、lint / typecheck 基线，并从新增领域边界开始使用 TypeScript
3. 清理 Rust 配置层的 `Mutex::lock().unwrap()` 等 panic 风险
4. 实现 ANM 文本工具链，再单独设计 sprite / 动画预览
5. 为 MSG / STD 补结构化诊断与基础语言服务

本轮 MVP 收尾计划见 [docs/superpowers/plans/2026-07-13-mvp-project-workflow-closure.md](./docs/superpowers/plans/2026-07-13-mvp-project-workflow-closure.md)。

## 开发

```bash
npm install
npm run tauri dev    # 完整桌面应用
npm run dev          # 前端单独开发
npm run build        # 前端构建
```

## Linux 服务器开发

本项目原在 Windows 上开发，迁移到 Linux 需注意以下几点。

### 1. 系统依赖（Tauri v2）

Debian / Ubuntu：

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

其他发行版请参考 Tauri 官方先决条件文档（webkit2gtk / gtk3 / librsvg 等）。

还需安装 Rust 工具链（rustup）和 Node.js（建议 LTS）。

#### 无 sudo 时的 conda 替代方案

服务器上没有 root 权限时，可用 conda-forge 提供开发库（gtk3 / webkit2gtk4.1 都有现成包）：

```bash
conda create -n tauri-dev -y -c conda-forge gtk3 webkit2gtk4.1 libsoup pkg-config nsis
```

之后每次编译/跑测试前设置环境（或写进 shell 配置）：

```bash
P=$(conda info --base)/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig
export LD_LIBRARY_PATH=$P/lib    # 测试二进制运行时需要
export PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

注意：headless 服务器上 `tauri dev` 能编译但弹不出窗口（需 `ssh -X` 转发）；
交叉打包 Windows 安装包用 `cargo-xwin`（`npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc`），该路径不依赖 GTK。

### 2. 拉取后初始化

```bash
git clone <repo-url>
cd THTK-Studio
npm install                 # 重装前端依赖（node_modules 未入库）
npm run tauri dev           # 首次会触发 cargo build，耗时较长
```

`node_modules/`、`dist/`、`src-tauri/target/` 均未提交，需在本地重新生成。

### 3. thtk 工具链（重要）

仓库内 `tools/*.exe`（thecl 等）是 **Windows 二进制，在 Linux 上无法运行**。Linux 上需要自行编译或安装 [thtk](https://github.com/thpatch/thtk) 的 Linux 版本，然后在应用设置里把工具链路径（`thecl_path` / `thtk_dir`）指向 Linux 二进制。代码中工具路径是可配置的，无需改源码。
