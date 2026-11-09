# 工具链依赖

更新：2026-11-09。MSG/END/STD 已接入固定版本 truth；ECL/ANM/DAT 保持 thtk。运行时通过外部 CLI 调用，不链接 truth Rust 库。

## 固定来源

- truth：zero318/truth `c78a541d1fb09fcefd448c83b097d4f9b815fce3`，Apache-2.0。
- TouhouMaps：`0673f4221f5e975842162e07cd7b765fc7570ebd`，Unlicense。
- [最小编码补丁](truth/patches/0001-msg-binary-encoding.patch)：MSG/END 增加 `--encoding shift-jis|gbk|utf-8`。源码、map 和诊断保持 UTF-8，其他格式不改编码。补丁随仓库保存，不维护查询指令或额外 truth 分支。
- [来源清单](manifest.source.json) 由开发构建脚本读取。实际二进制 SHA-256、平台与补丁 hash 生成在随包 `truth/build-info.json`。

## 开发与打包

```bash
rustup toolchain install 1.85.1 --profile minimal
node scripts/build-truth.mjs
npm run tauri dev
```

truth 固定使用 Rust 1.85.1；Studio 本体继续使用现有 Rust 工具链。首次构建需要网络、Git 与对应平台的 Rust/linker 环境，终端用户无需安装这些工具。Tauri 的 beforeDev/beforeBuild 会准备 truth；`npm run build` 单独只构建前端。

脚本在 `deps/.cache/truth` 获取固定源码、应用补丁并构建，把 `truth-core`、`trumsg`、`trustd` 放到 `src-tauri/resources/truth/`。两处均为忽略的生成目录。三个程序必须同目录。Tauri 把这个目录分发到应用资源 `truth/`，同时携带许可、crate 来源和构建记录。

Linux 交叉构建 Windows 工具需要 cargo-xwin 与目标工具链：

```bash
rustup target add --toolchain 1.85.1 x86_64-pc-windows-msvc
tauri-env win node scripts/build-truth.mjs x86_64-pc-windows-msvc
TAURI_ENV_TARGET_TRIPLE=x86_64-pc-windows-msvc tauri-env win npm run tauri -- build --runner cargo-xwin --target x86_64-pc-windows-msvc --config '{"bundle":{"targets":["nsis"]}}'
```

当前 Linux CLI 2.9.6 的 `--bundles` 参数不接受 nsis，使用配置覆盖。Windows 原生开发机可直接 `npm run tauri build`。切回 Linux 开发前重新运行无 target 参数的 `node scripts/build-truth.mjs`。

不加入在线更新、下载管理或回滚服务。thtk 继续手动配置，不因 truth 随包而自动安装。

## map 维护

原始社区快照在 `src-tauri/assets/maps/truth/zero318/`；产品基线在同级 `defaults/`。53 份按游戏与脚本类型区分的 map 来自固定 truth 内置签名、社区名称/枚举及 Studio 已有名称，来源和内容 hash 见各自 `PROVENANCE.json`。

```bash
python3 scripts/import-truth-maps.py deps/.cache/truth
```

这是维护时的显式生成步骤。运行时不解析 Rust 源码，也不运行此脚本。基线 map 内嵌进 Studio，首次使用释放到应用数据目录。默认 map、项目 `scriptMaps`、源码 `#pragma mapfile` 按顺序覆盖；编译器和 Studio 查询使用同一组文件。

升级时同时审阅基线、编码补丁与生成 map，验证代表性编解码及编码行为，再构建目标平台产物。完整边界与验证证据见 [迁移设计](../docs/superpowers/specs/2026-11-09-truth-msg-std-migration-design.md)、[实施计划](../docs/superpowers/plans/2026-11-09-truth-msg-std-migration.md) 和 [验证记录](truth-integration-check-2026-11-09.md)。
