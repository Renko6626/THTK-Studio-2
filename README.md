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

项目处于"第一阶段 MVP 工程闭环"的收尾期。thecl 的完整工作流已经闭环（编辑 → 编译 → 诊断 → Monaco 波浪线 → 问题面板 → 点击跳转）。下一步应扩展更多工具链。

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

### 已有但尚未闭环

- 工作区视图体系有 `text / binary-script` 两种，未扩展到 `msg / std / anm`
- 右侧边栏和底部面板已成形，大纲/检查面板仍是占位
- "终端"本质仍是命令执行面板，不是 PTY 终端

### 尚未完成

- 项目配置文件格式（`.thtk-project.json`）
- 最近项目 / 欢迎页
- MSG / STD / ANM 工具链 UI 和专用工作区视图
- 真正内嵌终端（PTY/ConPTY + xterm.js + 流式输出）
- 全局搜索、索引、资源引用分析
- ANM / 时间线 / MSG 预览
- AI 辅助

## 阶段进度

按 `project.md` 的阶段划分：

1. **第一阶段 MVP 工程闭环**：接近完成（thecl 闭环 ✅，文件管理 ✅，懒加载 ✅，文件监听 ✅）
2. 第二阶段 语言服务 MVP：ECL 基础语言服务已有，其他未开始
3. 第三阶段 可视化预览：未开始
4. 第四阶段 高级工程能力：未开始
5. 第五阶段 AI 集成：未开始

## 代码结构

```text
src/
  api/            前端到 Tauri command 的桥接（6 个模块）
  components/     编辑器、侧边栏、对话框、工具面板（20 个 SFC）
  composables/    工作台行为、文件树交互、工具链动作（10 个 hook）
  services/       工作区视图、工具链元数据、ECL 语言服务（22 个模块）
  stores/         Pinia 状态管理（9 个 store）
  utils/          图标、前端辅助函数

src-tauri/src/
  main.rs         Tauri 入口与 command 注册（20 个命令）
  app_state.rs    全局状态（ConfigManager + project root + file watcher）
  config.rs       应用配置读写
  common/         文件系统、命令执行、文件监听、工具链等通用能力（7 个模块）
  modules/ecl/    ECL/thecl 工具链封装、错误解析、eclmap 解析
```

## 下一阶段建议

1. 扩展 `msg / std / anm` 工具链到统一注册表
2. 明确项目配置文件格式
3. 欢迎页 / 最近项目
4. Rust 端错误处理清理（消除 unwrap panic 风险）
5. 真正 PTY 终端

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
