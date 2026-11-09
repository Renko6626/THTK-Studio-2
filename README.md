# THTK-Studio

THTK-Studio 是面向东方 Project 脚本与资源魔改的人机协作桌面 IDE，为作者和 coding agent 提供编辑、语言参考、检查、构建与资源理解能力。

当前仓库基于：

- Tauri v2
- Rust
- Vue 3
- Monaco Editor
- Naive UI

## 项目定位

长期方向见 [project.md](./project.md)。语言知识和工具能力同时服务人类编辑器与 agent，AI 接入不再作为所有其他功能完成后的最后阶段。

当前工具路线：MSG/END/STD 完整迁入 **zero318 版 truth**，使用原生源码与 map；ECL/ANM/DAT 暂留 thtk。truth 固定版本随安装包分发，旧 `.dmsg/.dstd` 支持直接废弃，GBK 汉化能力必须保留。

**MSG/END/STD 已切换到 truth 原生源码。** 编译、编辑器与 MCP 使用同一组版本 map 和项目覆盖，GBK 通过最小编码补丁保留。自动化验证与 Windows 实机待验项目见 [集成记录](deps/truth-integration-check-2026-11-09.md)。

- [迁移设计](docs/superpowers/specs/2026-11-09-truth-msg-std-migration-design.md)
- [GBK 编码扩展设计](docs/superpowers/specs/2026-11-09-truth-msg-encoding-design.md)
- [分阶段计划](docs/superpowers/plans/2026-11-09-truth-msg-std-migration.md)
- [依赖与分发约定](deps/README.md)

## 当前开发进度

第一阶段 MVP 工程闭环的功能项已经做完：ECL 完整工作流闭环，MSG / STD / DAT 具备基础编译、反编译或容器操作流程，PTY 终端、进程内 MCP Agent 通道、项目配置对话框、欢迎页与最近项目、事务式项目打开与切换保护均已落地。前端已全量迁至 TypeScript 并建立测试 / 类型检查门禁。剩余缺口是 Windows 实机验收，以及尚未实现的 ANM 工具链。

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
- MSG/END 工作流：trumsg，`.msg` ↔ `.msgspec/.endspec`、独立模式与 map、Shift-JIS/GBK 游戏文本和 UTF-8 源码、结构化诊断
- STD 工作流：trustd，`.std` ↔ `.stdspec`，原生 meta、标签和控制流；truth 三类源码共享高亮，按 map 提供参数/枚举提示与标签跳转
- DAT 基础工作流：容器解包与目录打包
- 真正内嵌终端：xterm.js + portable-pty，多会话、流式输出与 resize
- 进程内 MCP Agent 通道：ECL、MSG/END/STD 检查 / 编译 / 反编译 / 语义查询与客户端自动接线
- 项目配置：`.thtk-project.json` 三态读写（不存在 / 有效 / 损坏）、原子保存与项目设置对话框
- 项目级工具链覆盖：`toolchain.thtkDir` 与 `toolchain.truthDir` 分别覆盖对应工具，`scriptMaps` 设置三类 truth map
- 事务式项目打开：验证与扫描通过后才提交状态，失败不改动当前工作区
- 欢迎页与最近项目（10 项、去重置顶、失效标记与移除）
- 项目切换保护：脏标签三向确认、切换后只清理旧项目的标签、恢复失败落回欢迎页
- 文件系统安全加固：项目路径守卫、文件名校验、符号链接 / reparse point 防护与严格 CSP
- 前端全量 TypeScript：`allowJs` 已关闭，跨 Rust 边界的类型由 `wire_format_tests` 钉住
- 前端测试与类型检查门禁，覆盖 store / composable / service 层

### 已有但尚未闭环

- 工作区视图体系仍只有 `text / binary-script` 两种；MSG / STD 复用文本视图，ANM 尚无专用视图
- truth 时序面板显示源码事件；循环/分支时间未求值，不是游戏运行模拟

### 尚未完成

- 组件层与端到端测试（需要真实 Tauri 窗口，目前靠 Windows 手动验收）
- ANM 工具链、文本编辑层与 sprite / 动画预览
- 全局搜索、索引、资源引用分析
- 完整资源/效果预览；现有基础时间线不等于游戏运行模拟

## 阶段进度

- 已有：基础项目闭环、ECL 语言支持与 MCP、truth MSG/END/STD 流程、终端和 TypeScript 检查。
- 已接入：truth 固定依赖、GBK 补丁、53 份版本 map、编辑器和 agent 同源查询；Windows 安装与游戏效果需实机验收。
- 后续：ANM 工具链与资源预览、跨文件资源关联、更多实际效果反馈。
- Windows 实机验收仍需按实际完成记录，不由文档改动替代。

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

## 下一阶段

先审阅 truth 迁移与编码设计，按 [迁移计划](docs/superpowers/plans/2026-11-09-truth-msg-std-migration.md) 完成固定工具核对、Studio map 解析与接口细化，再实施完整切换。旧实施计划保留作历史记录；冲突处以新设计和用户确认事项为准。

## 开发

```bash
npm install
npm run tauri dev    # 完整桌面应用
npm run dev          # 前端单独开发
npm run build        # 前端构建
npm test             # 前端单元测试（store / composable / service 层）
npm run typecheck    # 全量类型检查（含 .vue）
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

交叉打包 Windows 安装包用 `cargo-xwin`
（`npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc`），
该路径走 WebView2，完全不依赖 GTK。

#### headless 服务器上怎么看界面

`ssh -X` 不是唯一选择，也不是好选择。开一个 VNC 虚拟桌面，把 `tauri dev` 跑在上面，
会话是持久的，断开 SSH 不影响：

```bash
vncserver :2 -localhost yes -geometry 1680x1050 -depth 24
DISPLAY=:2 <上面那套环境> npx tauri dev
```

两个坑（都会表现为「窗口开了但一片空白」，很容易误判成跑不起来）：

1. **conda-forge 的 `webkit2gtk4.1` 2.48.5 渲染不出网页内容**——`WebKitWebProcess`
   会 100% 空转。GTK 外壳（窗口、标题栏）却正常，所以看着像白板。换一个能用的
   webkit 4.1 运行时（比如从发行版的 .deb 里解出 2.50.x，只在运行时用
   `LD_LIBRARY_PATH` 指过去，**不要**和 conda 的 lib 混在一起）。
2. **系统设了 `http_proxy` 却没有 `no_proxy`**——webview 会把
   `http://localhost:1420`（vite dev server）也发给代理，结果白屏。设
   `no_proxy=localhost,127.0.0.1`。

在 zhustation 这台开发机上，以上全部已经配好并封装成 `tauri-env`：

```bash
tauri-env vnc   npx tauri dev     # 窗口出现在 VNC 桌面 :2 上
tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml
tauri-env win   npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc
```

细节见 `/data/sunyunbo/www/tauri-build-handbook.md`。

### 2. 拉取后初始化

```bash
git clone <repo-url>
cd THTK-Studio
npm install                 # 重装前端依赖（node_modules 未入库）
rustup toolchain install 1.85.1 --profile minimal  # 仅供固定 truth 构建
npm run tauri dev           # 首次会触发 cargo build，耗时较长
```

`node_modules/`、`dist/`、`src-tauri/target/` 均未提交，需在本地重新生成。

### 3. 外部工具链

仓库内 `tools/*.exe`（thecl 等）是 **Windows 二进制，在 Linux 上无法运行**。Linux 上需要自行编译或安装 [thtk](https://github.com/thpatch/thtk) 的 Linux 版本，然后在应用设置里把工具链路径（`thecl_path` / `thtk_dir`）指向 Linux 二进制。代码中工具路径是可配置的，无需改源码。

MSG/END/STD 的 truth 工具由开发启动／打包脚本准备，终端用户使用安装包自带版本。构建平台、固定来源与自定义目录说明见 [deps/README.md](deps/README.md)。
