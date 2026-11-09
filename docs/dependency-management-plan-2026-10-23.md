# Windows-first 依赖管理实施计划

> **2026-11-09 状态更新：历史记录。** MSG/END/STD 已决定迁入随包 truth，旧方言直接废弃，GBK 必须保留。后续以 [新迁移设计](superpowers/specs/2026-11-09-truth-msg-std-migration-design.md) 为准；本文中旧后端与依赖选择不再构成实施授权。现有 ECL map 基线事实仍有效，thtk 在线安装/更新继续暂缓。


**日期：** 2026-10-23
**状态：** M0/M1 已实现，规则一致性收尾于 2026-10-25；M2/M3 暂缓（用户决定暂不推进 thtk 版本管理）
**范围：** Windows 优先；Linux/macOS 暂保留手动路径

## Context

当前 thtk 与 mapfile 都靠用户手动放进文件夹：`thtk_dir` / `tool_overrides` / `eclmap_path` 全是指向本机路径的设置。实际使用中有三个问题：

1. 官方 thtk release 12（2019）只支持到 TH17，现代作品需要 master/nightly；用户不知道该用哪个。
2. mapfile 的命名、版本和来源不统一，ECL 编辑器与 thecl 的默认 map 解析还不一致。
3. 首次配置门槛高，而 thtk 与 zero318/TouhouMaps 的许可证其实允许更省心的分发/下载。

目标：**Windows 上让 IDE 负责 thtk 与 map 的安装/更新，Linux/macOS 先保留手动路径**。不做静默升级，所有下载都由用户显式触发，并保留手动覆盖。

## 范围与非目标

**范围**

- Windows x64：托管 thtk nightly（官方 Windows 构建），自动发现/安装/更新/回滚。
- ECL mapfile：内置一份 permissive 基线 + 可更新快照；统一解析顺序。
- 依赖状态 UI：来源、版本、支持的 TH 范围、SHA-256、许可证、错误提示。
- 离线导入与手动路径始终可用。

**非目标（本阶段不做）**

- Linux/macOS 的官方二进制托管（先保留手动/自建）。
- thtk 源码构建、自动编译。
- MSG/STD/END 外部 map 的在线更新（它们已内嵌在 `src-tauri/assets/maps`）。
- Priw8/eclmap 的再分发（无 LICENSE，需先拿许可）。
- 项目级依赖 pin 的完整 UI（Phase 3 再做）。

## 事实基线

- thtk 许可是 BSD 式宽松许可（`COPYING`），不是 GPL；随包/下载分发需保留版权与免责声明。
- 官方 stable 12 只到 TH17；官方 nightly 的 README 表覆盖到 TH20：
  `https://thcrap.thpatch.net/thtk-nightly/thtk-win32-x64.zip`
- nightly zip 约 1.2 MB，含五个工具 + `COPYING`/`NEWS`/`README`；没有签名或 checksum。
- `thdat.exe` 依赖 `VCOMP140.DLL`，nightly/stable zip 都没带；需要系统 VC++ 运行库或额外处理。
- zero318/TouhouMaps 是 **Unlicense**；固定 commit 可取 tarball。它的 `any.eclm` 对 th13+ 不可靠，必须按版本显式选文件。
- TouhouMaps 的 ECL 覆盖并不完整：有 th06–12、th15、th18、th185、th19、th20，缺 th128/th13/th14/th143/th16/th165/th17。
- Priw8/eclmap 无 LICENSE，不能随包/镜像。
- 旧 THTK-Studio v1 是 MIT，自带 th06–20 的 `eclm_files`；th19/20 明确来自 zero318，其余 provenance 需要审计后才能作为默认包。

## 架构

### 目录布局

生产环境使用 `ProjectDirs::data_dir()`（不是 config 目录）：

```text
<data_dir>/
  deps/
    manifest.json                  # 安装状态 + 已选版本 + 来源
    thtk/
      nightly-2026-10-08/
        bin/                       # 五个 exe + thtk.dll
        COPYING.txt ...
        manifest.json              # 来源 URL、SHA-256、安装时间、支持的版本
    maps/
      bundled/                     # 从二进制内嵌释放的基线
      snapshots/<sha>/             # 下载的 TouhouMaps 快照
```

临时下载放 `ProjectDirs::cache_dir()`，安装采用「临时目录 → 校验 → 原子 rename」，失败不破坏旧版本。

### 依赖状态

新增 `src-tauri/src/common/dependencies.rs`：

```rust
pub struct DependencyStatus {
    pub thtk: Option<InstalledThtk>,
    pub maps: MapStatus,
    pub source: DependencySource, // manual | managed
    pub warnings: Vec<String>,
}
```

- `DependencyManager` 接受一个 base dir 参数，生产用 data dir，测试用 temp dir。
- 状态文件 `deps/manifest.json` 不进入 `AppConfig`，避免改动现有 wire format。
- `AppConfig.thtk_dir` / `eclmap_path` 继续表示**手动来源**；sidecar 记录 `source = manual | managed`。
- 解析优先级：
  1. 显式 `tool_overrides` / `thecl_path`
  2. 项目级 `toolchain.thtkDir`
  3. 手动 `thtk_dir`
  4. IDE 管理的 thtk（M2，未实现）

  当前不从 PATH 自动发现工具；显式路径失效时不自动换用其他来源。
- map 优先级：
  1. 项目 `mapPaths`
  2. 全局 `eclmap_path`
  3. IDE 管理的 map（bundled / snapshot）
  4. 有效 `thtk_dir/maps/` 推导（项目目录覆盖全局目录，不混合搜索）

  项目根目录及其 `maps/` 不自动参与查找，需要通过项目 `mapPaths` 显式指定。显式项目 map 或全局 map 加载失败时，编辑器显示错误，不回退换源；多份显式 map 必须全部加载成功才发布词表。

### 安装清单

`deps/manifest.json` + 仓库内 `deps/manifest.source.json`（可提交、可 review）：

```json
{
  "thtk": {
    "channel": "nightly",
    "url": "https://thcrap.thpatch.net/thtk-nightly/thtk-win32-x64.zip",
    "sha256": "...",
    "published": "2026-10-08",
    "supportedVersions": [6, 7, 8, 9, 95, 10, 11, 12, 128, 13, 14, 15, 16, 17, 18, 185, 19, 20]
  },
  "maps": {
    "source": "zero318/TouhouMaps",
    "commit": "0673f4221f5e975842162e07cd7b765fc7570ebd",
    "url": "https://github.com/zero318/TouhouMaps/archive/<sha>.tar.gz",
    "sha256": "..."
  }
}
```

- 仓库内清单是维护者更新的「已知可用」版本；应用启动时可选择从 `main` 拉取最新清单，失败则用内嵌基线。
- 更新脚本：`scripts/update-dependency-manifest.sh`，下载 zip/tarball、计算 SHA-256、写入清单并打印 diff。
- 不直接把 1.2 MB 的 thtk zip 提交进仓库；清单只存 URL + hash。

### Windows thtk 安装流程

1. 用户点「安装 / 更新 thtk」；先展示来源、版本、大小、SHA-256、许可证。
2. 下载到 cache，限制大小/超时，校验 SHA-256。
3. 解压到临时目录，做 zip-slip 防护（拒绝绝对路径和 `..`）。
4. 校验五个 exe 存在；逐个运行 `-V`（带超时），解析支持的版本。
5. 检查 `VCOMP140.DLL`：
   - 优先看 managed 目录 / `System32`；
   - 缺失时安装仍可完成，但把 `thdat` 标成不可用，并提示安装 VC++ 运行库；
   - 后续可选：从官方 VC++ redistributable 附带该 DLL（先做许可确认）。
6. 原子 rename 到 `deps/thtk/nightly-<date>/`，更新状态文件，保留上一版本用于回滚。
7. 触发工具链状态刷新；若项目正在使用手动路径，不自动切换。

### Windows-first 的取舍

- 官方 nightly 无签名：采用 **manifest 固定 SHA-256 + HTTPS + 显式确认**；不静默升级。
- 不做首次启动自动下载；在工具链设置/欢迎页提供入口。
- 网络失败、代理、镜像：提供「离线导入 zip」和可配置镜像 URL；不依赖 Tauri HTTP 插件，直接在 Rust 侧下载。
- 新依赖：`reqwest`（rustls，含代理/重定向）、`zip`、`sha2`（若已有传递依赖则提为直接依赖）。新增前先跑 `cargo tree` 确认体积与许可。

### map 处理

- Phase 1 先内置 zero318 基线（Unlicense）：
  - 从固定 commit 取文件，统一命名成 `th{version}.eclm`。
  - th15/18/185/19/20 使用 `_thtk.eclm` 变体（th15 只有 `th15_thtk.eclm`）；其余用普通 `.eclm`。
  - 带 `PROVENANCE.json` 记录源文件、commit、许可证。
  - 缺失版本（th128/13/14/143/16/165/17）不假装覆盖，UI 标「需手动提供」。
- 首次使用时把内嵌基线释放到 `deps/maps/bundled/`，让 thecl 和编辑器都能拿到真实文件路径。
- 更新时下载 TouhouMaps 固定 commit tarball 到 `deps/maps/snapshots/<sha>/`，保留旧快照。
- 统一解析：
  - Rust `resolve_default_maps` 从 `{version}.eclmap` 改为同时尝试 `th{version}.eclm`、`{version}.eclm`、`{version}.eclmap`，并加入 managed map 目录。
  - 前端 `semantic-loader` 增加 managed map 候选；保持项目 `mapPaths` > 全局 `eclmap_path` > managed > thtk_dir 推导。
- 旧 THTK-Studio v1 的 th13–17 map 作为 Phase 3 的审计项：确认 provenance 后再决定是否纳入内置；未确认前只作为用户手动路径。

## 任务拆分

### Milestone 0：地基（不改下载）

- [x] 修正 `common/toolchain.rs::not_configured_message` 的 “GPL” 文案，改为 BSD 式许可并指向 `COPYING`。
- [x] 只读依赖卡片展示许可证说明与 map 来源 commit。完整 `COPYING` 查看随安装功能留待 M2，未计入本阶段完成范围。
- [x] 新增 `src-tauri/src/common/dependencies.rs`：状态模型、路径解析（manifest 读写延后到 M2）。
- [x] 在 `main.rs` 注册 `get_dependency_status`。
- [x] 统一 map 解析：
  - [x] `src-tauri/src/modules/ecl/commands.rs::resolve_default_maps` 支持 `.eclm` 并接入 managed map 目录。
  - [x] `src/services/languages/ecl/semantic-loader.ts` 增加 managed map 候选。
- [x] 前端新增只读「依赖」卡片：显示 thtk 是否配置、来源、版本、map 覆盖范围。
- [x] 测试：路径优先级、`.eclm` 解析（manifest 反序列化随 M2 补）。

### Milestone 1：内置 ECL map 基线

- [x] 新增 `src-tauri/assets/maps/ecl/`，放入 zero318 固定 commit 的基线文件与 `PROVENANCE.json`。
- [x] `dependencies::ensure_bundled_maps()`：首次使用释放到 `deps/maps/bundled/`。
- [x] `get_dependency_status` 返回每个版本的覆盖情况与来源。
- [x] 前端依赖卡片显示覆盖版本，缺失版本提示「手动指定 mapPaths」。
- [x] 测试：释放后的文件名、版本解析、缺失版本不误报。

### Milestone 2：Windows thtk 安装/更新（暂缓）

- [ ] 添加 `reqwest`（rustls）/`zip`/`sha2` 依赖；确认 `cargo tree` 与许可。
- [ ] 实现下载、SHA-256 校验、zip-slip 防护、原子安装、回滚。
- [ ] 运行 `-V` 验证五个工具；记录 supportedVersions。
- [ ] `VCOMP140.DLL` 检测与错误提示；thdat 不可用时不让整个安装失败。
- [ ] `install_thtk` / `remove_thtk` / `import_thtk_zip` 命令 + 进度事件。
- [ ] `toolchain::resolve_tool_path` 在手动路径为空时回落到 managed thtk。
- [ ] 前端「安装 / 更新 / 移除 / 导入 zip」UI、进度条、来源与 hash 展示。
- [ ] 维护脚本 `scripts/update-dependency-manifest.sh`。
- [ ] 测试：本地 fixture zip 安装、hash 不匹配、zip-slip、回滚、`-V` 失败。

### Milestone 3：更新、离线与项目 pin（暂缓）

- [ ] map 快照更新/回滚 UI。
- [ ] 代理/镜像设置与离线导入。
- [ ] `.thtk-project.json` 增加可选 `toolchain.thtkVersion` / channel；同步 `KNOWN_TOOLCHAIN_KEYS`、TS 类型、wire tests。
- [ ] 项目级 pin 优先于全局托管版本。
- [ ] 审计旧 MIT map，决定 th13–17 的最终来源。

## 关键文件

**后端**

- 新增：`src-tauri/src/common/dependencies.rs`
- 修改：`src-tauri/src/common/mod.rs`、`config.rs`、`common/toolchain.rs`、`modules/ecl/commands.rs`、`main.rs`
- 数据：`src-tauri/assets/maps/ecl/`、`deps/manifest.source.json`
- 依赖：`src-tauri/Cargo.toml`

**前端**

- 新增：`src/types/dependencies.ts`、`src/api/modules/dependencies.ts`、`src/stores/dependencies.ts`
- 新增/修改：`src/components/Dialogs/ToolchainSettingsDialog.vue` 或独立 `DependencySettingsDialog.vue`
- 修改：`src/services/languages/ecl/semantic-loader.ts`、`src/api/index.ts`

**测试**

- `src-tauri/src/common/dependencies.rs` 单测
- 前端 `tests/unit/dependencies.spec.ts`
- 手动 Windows 验收清单

## 实施记录（2026-10-24）

M0/M1 已落地：

- `src-tauri/src/common/dependencies.rs`：只读状态、bundled map 释放、路径解析。
- 内置 15 份 zero318 ECL map（th06–12、th095/th103/th125、th15、th18、th185、th19、th20），
  缺失 `[128, 13, 14, 143, 16, 165, 17]`，不放假数据。
- `resolve_default_maps` 与前端 semantic-loader 已统一 `.eclm` / `.eclmap` 与补零命名，
  顺序为显式 mapPaths > eclmap_path > managed > thtkDir/maps。
- 设置对话框新增只读依赖卡片；thtk 的 GPL 文案已改为 BSD 式许可说明。

与计划的偏差：

- 未新增 `src/stores/dependencies.ts`，依赖卡片使用对话框局部 `ref`。
- `deps/manifest.json` 的读写延后到 M2；M0/M1 不需要安装状态。
- 依赖卡片目前只展示许可证文案与来源 commit，完整 `COPYING` 查看留待 M2。

验证：

- `cargo test`：271 passed / 2 ignored
- `npm test`：164 passed
- `npm run typecheck` / `npm run build`：通过

## M0/M1 收尾（2026-10-25）

- 保留既有工具路径优先级：单工具覆盖 > 项目 thtkDir > 全局 thtk_dir；不新增 PATH 回退。
- 编辑器取消项目根目录及其 maps/ 的隐式搜索；默认候选目录与后端一致。
- 项目 thtkDir 非空时，不再额外搜索全局 thtk_dir/maps。
- 显式 map 任一加载失败时返回空词表和带路径的错误，通过现有工作台提示显示；不发布部分词表，不回退到其他来源。
- 保留已有工作区中的内置 15 份 map、来源记录、只读状态卡片和后端解析实现。
- M2/M3 暂不推进，不新增安装、下载、更新、版本固定、清单或依赖包。

本次验证：

- `npm test`：167 passed（含显式 map 失效、部分失效、默认目录和项目覆盖回归）。
- `npm run typecheck`、`npm run build`：通过；构建有大 chunk 提示。
- `tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml`：271 passed / 2 ignored；编译有 3 条未使用函数警告。
- 用 Python SHA-256 核对 `PROVENANCE.json`：15 份本地 map 全部匹配；未重新下载上游文件。
- `git diff --check`：通过。
- 未运行 Windows 实机验收或界面交互；thtk 安装与版本管理保持未实现。

下列内容仍需在重启 M2 前重新决策，原方案仅作为讨论记录：

- 滚动 nightly URL 与固定 SHA-256 无法保证历史包可获取，先确定是否提供固定版本产物。
- `-V` 探测工具版本；支持的游戏版本从 usage 获取，不能混为一次探测。
- 文件完整性与逐工具可用性分别判断；thdat 缺运行库不应与“五个工具必须全部运行成功”同时作为安装要求。
- 下载可放 cache；需要 rename 切换时，解压临时目录应位于目标目录所在卷。
- 离线导入在 M2/M3 的归属重复，后续按实际交付范围调整。

## 验收

当前阶段验证内置 map 释放、默认命名与路径优先级、显式配置失败提示，以及前端类型和构建。下述 Windows 安装验收属于尚未实现的 M2/M3，不代表 M0/M1 已完成它们。


**自动化**

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test` / `npm run typecheck` / `npm run build`
- 本地 fixture zip 的安装/回滚测试不依赖网络。

**Windows 手动**

1. 干净用户目录启动，未配置 thtk；依赖卡片显示未安装。
2. 安装 nightly；五个工具 `-V` 成功；来源、hash、许可证可见。
3. 没有 VC++ 运行库时，`thdat` 给出明确提示，其余工具正常。
4. 用托管 map 编译一个 th18 ECL；不手动放任何 map 文件。
5. 删除/回滚 thtk；手动路径仍可作为回退。
6. 断网安装失败，旧版本不被破坏；离线 zip 导入成功。
7. 更新 manifest 后，更新按钮能识别新版本并保留旧版本。

## 风险与待决

- nightly 无签名：SHA-256 固定只能防 manifest 之后的篡改，不能防上游投毒；是否改为「我们自己托管已知可用 zip」需要后续决定。
- VCOMP140 的再分发许可需要确认；在此之前只做检测和提示。
- th13–17 的 map 来源是最大数据缺口；不审计就不内置。
- Linux/macOS 无官方 nightly，本计划明确不做自动安装。
- 网络受限环境需要镜像/离线导入；首次实现至少要有离线导入。
