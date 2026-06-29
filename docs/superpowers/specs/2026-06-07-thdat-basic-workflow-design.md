# thdat 基本工作流(解包 + 打包)

日期:2026-06-07 状态:已批准范围

## 目标

让 `.dat` 容器能在 IDE 内"解包到目录"和"从目录打包",形成与 thmsg/thstd 平级但更扁平(无脚本语义层)的工作流。

## 与前面工具的差异

| 维度 | thmsg / thstd | **thdat** |
|---|---|---|
| 处理对象 | 单个脚本文件 | **容器**(里面是任意多个文件) |
| 语义翻译 | 必需(map_parser + translator) | **无**(打包器,不解析内容) |
| 编码 | SJIS / UTF-8 | **N/A**(文件按字节透传) |
| 版本检测 | 用户配置/项目配置 | 解包 `-xd` 自动检测;**打包**必须显式版本 |
| 工作量 | ~thecl 60% | **~thmsg 60%**(更扁平,但要 directory/file picker UI) |

## CLI 参考(thtk 上游)

- 解包:`thdat -xd archive.dat -C target_dir` (自动检测版本)
- 打包:`thdat -c<version> archive.dat -C source_dir file1 file2 ...`(版本紧贴 flag,如 `-c17`)
- Exit 0 = 成功,1 = 失败

注:`-c` 模式要列出文件名,直接传目录不行——我们的实现要在 Rust 侧列目录、把所有文件名作为参数(注意 Windows 命令行长度上限 32K,th 系列 .dat 几百个文件可能逼近;如撞到上限退化为 ThtkResult err 让用户分批,本期不实现 chunking)。

## 在范围

1. **Rust `modules/thdat/`**:
   - `compiler.rs`:`ThdatRequest { mode: Extract|Pack, version, archive_path, target_dir, files: Option<Vec<String>> }`,`ThdatResult`(沿用 `EclResult`-like shape)
   - `commands.rs`:`extract_dat_file(archive_path, target_dir)` / `pack_dat_file(source_dir, archive_path)`
   - **无** map_parser、translator(thdat 无语义)
   - 模块挂到 `modules/mod.rs`

2. **前端最小集成**:
   - `src/api/modules/compiler.js` 加 `extractDatFile` / `packDatFile`
   - 菜单"脚本 → 解包 .dat / 打包目录为 .dat"(或者放到一个新的"工具"二级菜单,跟脚本分开)。简化:**复用"脚本"菜单**,加个 divider 隔开。
   - 解包流程:
     - 优先取活动 tab(若是 .dat),不弹文件选;否则弹 `@tauri-apps/plugin-dialog` `open({ filters: [{name:'Touhou Archive', extensions:['dat']}] })`
     - 弹 `open({ directory: true, defaultPath: ${dat_parent}/${dat_stem}/ })` 选目标目录
     - 调 extract_dat_file
     - 完成后刷新文件树
   - 打包流程:
     - 弹 `open({ directory: true })` 选源目录
     - 弹 `save({ filters:[{name:'Touhou Archive', extensions:['dat']}] })` 选目标 .dat
     - 版本取 `effective_version`(同 thmsg/thstd 模式)
     - 调 pack_dat_file
   - 结果同样推到输出面板

3. **版本解析**:解包用 `d`(自动);打包用 `effective_thdat_version`(同 thmsg/thstd 的 effective_*_version helper)

## 不在范围

- **list 模式**(已知限制:用户解包后用文件树看)
- **glob 过滤选择性提取**(基本工作流是整体提取/打包)
- **多 .dat 批处理 UI**
- **流式进度**(同步等结果,后续可优化)
- **chunking 突破命令行长度上限**(撞到再做)
- MCP 工具、AI 辅助包扩展、BuildDialog、registry executor 接线(沿用 stub 形态)

## 关键实现细节

### Rust 侧 ThdatRequest 设计

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ThdatMode { Extract, Pack }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThdatRequest {
    pub mode: ThdatMode,
    pub version: String,            // Extract 时被忽略(用 d);Pack 时必填
    pub archive_path: String,       // .dat 路径
    pub target_dir: String,         // Extract:目标目录;Pack:源目录
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThdatResult {
    pub success: bool,
    pub tool: String,          // "thdat"
    pub mode: String,
    pub archive_path: String,
    pub target_dir: String,
    pub message: String,
    pub diagnostics: Vec<crate::modules::ecl::error_parser::Diagnostic>, // 总是空,frontend 兼容
    pub file_count: Option<usize>, // 提取后/打包前的文件计数,用于 UI 报告
}
```

### Extract 实现要点
- 创建 `target_dir` 若不存在(`fs::create_dir_all`)
- 调 `thdat -xd {archive} -C {target_dir}`(`-x` 与 `d` 紧贴)
- stderr 解码用 `cmd_runner::run_tool`(SJIS 解码后字符串,可能含日文错误消息)
- 解包后扫 `target_dir` 算 file_count(给 UI 用)
- 失败时不清理 target_dir(用户可调试)

### Pack 实现要点
- 列 `target_dir` 一层(`fs::read_dir`),收集相对文件名(不递归,thdat 不支持嵌套目录;若 thdat 实际支持递归再说,本期一层)
- 构造命令:`thdat -c{normalize_version} {archive_path} -C {target_dir} file1 file2 ...`
- 命令行长度估计(总参数字节数);若 > 28000 → 返回失败"too many files, batching not implemented"
- 调用,成功后 stat archive_path 拿大小用于 message
- file_count = 列目录的数量

### 文件选择 UX(前端)
- 用 `@tauri-apps/plugin-dialog`(已经在依赖里)的 `open`/`save`
- 解包默认目标:同目录同名(`/path/th17.dat` → `/path/th17/`),用户可改
- 打包默认源:无默认,弹空 picker
- 取消任何 picker → 静默取消(不发卡片)

## 测试

Rust(`modules/thdat/compiler.rs`):
- `build_thdat_extract_args` 构造测试(`["-xd", archive, "-C", target]`)
- `build_thdat_pack_args` 构造测试(`["-c17", archive, "-C", target, "f1", "f2"]`)
- `normalize_thdat_version` 同 thmsg
- `pack_fails_when_file_args_too_long` 构造一个超长名单 → 函数 returns Err 不调用 thdat
- `run_returns_failure_when_thdat_path_not_configured`

CLI 实跑测试在 Linux 不做(同 thmsg/thstd 原因),Windows 验收覆盖。

## 任务拆分(3 个)

### Task 1 — Backend(TDD)
- `modules/thdat/{mod,compiler,commands}.rs`
- main.rs 注册 `extract_dat_file` / `pack_dat_file`
- 5 个测试

### Task 2 — 前端 + 验证
- api 包装 + 菜单 + dialog plumbing + result cards + 刷新文件树
- npm run build + cargo test 全套通过

### Task 3 — 文档
- `script-support-status.md` thdat 行(从 ❌ 改为 ✅,标注"无脚本语义,纯容器管理")
- `CLAUDE.md` Backend structure 加 `modules/thdat/`(注明无 map_parser/translator)
- `editor-shell-status.md` §2.10 加 thdat 工作流,验收清单 22-25

## 验收(Windows 单独跑)

22. 菜单"脚本 → 解包 .dat":选 .dat 文件 + 目标目录 → 输出面板"解包 .dat 完成"卡片显示 file_count;目录里出现解出来的 .ecl/.anm 等
23. 编辑解出的某个 .ecl(或不改),菜单"打包目录为 .dat":选源目录 + 目标 .dat → 输出面板"打包 .dat 完成"卡片
24. 给定一个非法目录(无文件)打包 → 失败卡显示 thdat stderr
25. thtk_dir 未配置:触发立刻失败 "thdat path is not configured"
