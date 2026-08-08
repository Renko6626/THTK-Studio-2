# MSG / STD 可读性生态 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**日期：** 2026-08-05

**Goal:** 把 MSG / STD 的可读性从"自创 JSON + 静默回退"升级为"生态格式 mapfile + 显式版本分档 + IDE 层跳转导航"，并把 `.dmsg` / `.dstd` 是 IDE 方言这件事显式化。

**Architecture:** thtk 作唯一编解码后端；语义数据改用 eclmap 家族的 `.msgm` / `.stdm`（含 gamemap 间接），直接复用既有的 eclmap 解析器；跳转标签不写入文件，改由 Monaco 提供导航。决策依据见 `docs/toolchain-ecosystem.md`。

**Tech Stack:** Rust（无新依赖，复用 `modules/ecl/map_parser` 与 `regex`）、Vue 3 + TypeScript、Monaco、vitest。

## Global Constraints

- 前端 `allowJs: false`，禁止新增 `.js`；新增 SFC 必须带 `lang="ts"`。
- 新增跨边界结构用 `#[serde(rename_all = "camelCase")]`，并在 `src-tauri/src/wire_format_tests.rs` 钉住键集。
- 分层规则（`AGENTS.md`）：mapfile 解析、版本分档属 Rust 层；导航与展示属前端。
- 每个 Task 结束前四道门禁必须绿：`npm test` / `npm run typecheck` / `npm run build` / `cargo test --manifest-path src-tauri/Cargo.toml`（Linux 上跑 cargo 需先 export conda `tauri-dev` 环境）。
- **绝不静默回退**：版本查不到数据必须让用户看见，不得像现在的 `let _ = version;` 那样蒙对。

## 事实依据（实施前必读）

完整证据链见 `docs/toolchain-ecosystem.md`，此处只列直接影响实现的：

1. **thstd 的 th14–th20 共用 `formats_v2`，无 per-version 分支** → th17 的 STD 数据对 th20 正确。
2. **thmsg 的签名表按版本组分开**：`th185_msg_fmts[]` 新增 id 37/38/39，`th19_msg_fmts[]` 新增 id 42–47、50–56。我们现有的 33 条（opcode 0–35）**对 th20 缺 16 条**。
3. **`modules/ecl/map_parser.rs` 的 `parse_ecl_map_content` 对未知 `!` 段是优雅跳过**（见其 178–190 行），因此**能直接解析 `.msgm` / `.stdm`**，不需要改解析器。
4. **thmsg / thstd 永远只看到 `ins_N`**：翻译在我们这层做，磁盘上的 `.dmsg` / `.dstd` 不是合法的 thmsg / thstd 输入。
5. 我们的指令命名与 truth 的 mapfile **逐条一致**，格式迁移无损。

---

## Task 1：把 IDE 方言显式化

**Files:**
- Modify: `src-tauri/src/modules/msg/translator.rs`
- Modify: `src-tauri/src/modules/thstd/translator.rs`
- Modify: `src-tauri/src/modules/msg/commands.rs`、`src-tauri/src/modules/thstd/commands.rs`
- Modify: `src/components/Common/MenuBar.vue`

**Interfaces:**
- Produces：`pub const DIALECT_HEADER_MSG: &str` / `DIALECT_HEADER_STD: &str`；Tauri 命令 `export_raw_dmsg` / `export_raw_dstd`。
- Consumes：既有的 `dmsg_to_readable` / `readable_to_dmsg`。

- [ ] **Step 1: 写失败测试**

追加到 `modules/msg/translator.rs` 的 `mod tests`：

```rust
#[test]
fn readable_output_carries_a_dialect_header() {
    let semantics = test_semantics();
    let out = dmsg_to_readable("ins_3(0)\n", &semantics, false);
    assert!(
        out.starts_with("# THTK-Studio dmsg"),
        "产物必须自带方言声明, got: {out}"
    );
    assert!(out.contains("thmsg"), "要说明它不能直接喂给 thmsg");
}

/// 方言头是我们加的，翻译回去时必须原样去掉，否则 thmsg 会把它当指令。
#[test]
fn dialect_header_round_trips_away() {
    let semantics = test_semantics();
    let readable = dmsg_to_readable("ins_3(0)\n", &semantics, false);
    let raw = readable_to_dmsg(&readable, &semantics);
    assert!(!raw.contains("THTK-Studio"), "方言头不能进 thmsg 的输入");
    assert_eq!(raw.trim(), "ins_3(0)");
}
```

> `test_semantics()` 是该文件既有的测试辅助；若不存在，照同文件其他测试的构造方式补一个返回 `MsgSemanticData` 的函数，至少包含 `{opcode: 3, name: "textboxShow"}`。

- [ ] **Step 2: 跑测试确认失败**

```bash
P=/data/sunyunbo/miniconda3/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig LD_LIBRARY_PATH=$P/lib PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml msg::translator
```

预期：`readable_output_carries_a_dialect_header` 失败（产物没有该前缀）。

- [ ] **Step 3: 写实现**

在 `modules/msg/translator.rs` 顶部加：

```rust
/// 写在可读 .dmsg 开头的方言声明。
///
/// thmsg **没有** `-m`，指令名映射是本 IDE 在它外面做的：解包时把 `ins_N` 换成
/// 名字写盘，打包时再换回来喂给 thmsg。因此磁盘上的 .dmsg **不是合法的 thmsg
/// 输入**——直接跑 `thmsg -c` 会在每条带名字的指令上失败。
///
/// 之所以仍在磁盘上存名字：`git diff` 里 `textboxShow(0)` 与 `ins_3(0)` 的可读性
/// 差距是决定性的。代价就是必须把这件事写在文件里，而不是指望用户记得。
pub const DIALECT_HEADER_MSG: &str =
    "# THTK-Studio dmsg（IDE 方言，非 thmsg 原始格式）\n\
     # 指令名由 IDE 映射，thmsg 只认 ins_N。需要原始格式请用「导出原始 .dmsg」。\n";
```

`dmsg_to_readable` 的返回值前面拼上该常量；`readable_to_dmsg` 在逐行处理时跳过以 `# THTK-Studio` 开头的行（既有的 `#` 注释透传逻辑可能已覆盖，若已覆盖则只需确认测试通过）。

`modules/thstd/translator.rs` 同样加 `DIALECT_HEADER_STD`，措辞把 thmsg 换成 thstd。

- [ ] **Step 4: 加导出原始格式的命令**

在 `modules/msg/commands.rs` 追加：

```rust
/// 导出不含方言的原始 .dmsg（thmsg 可直接编译的形式）。
///
/// 给需要脱离 IDE 走命令行 / CI 的场景用。输入是磁盘上的方言 .dmsg，
/// 输出是 `ins_N` 形式。
#[tauri::command]
pub async fn export_raw_dmsg(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
) -> Result<String, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    let version =
        crate::common::game_version::resolve_from(ctx.project.as_ref(), &ctx.config, "thmsg")?
            .to_string();
    let semantics = super::map_parser::parse_msg_semantics(&version)?;

    let content = crate::utils::read_text_file(&input_path, "utf-8")
        .map_err(|e| format!("读取 {input_path} 失败: {e}"))?;
    let raw = super::translator::readable_to_dmsg(&content, &semantics);
    crate::utils::write_file_utf8(&output_path, &raw)
        .map_err(|e| format!("写入 {output_path} 失败: {e}"))?;
    Ok(output_path)
}
```

`modules/thstd/commands.rs` 同理写 `export_raw_dstd`（工具 id 换 `thstd`，语义解析换 `parse_std_semantics`，翻译函数换 `readable_to_dstd`）。

在 `main.rs` 的 `invoke_handler!` 注册这两个命令。

`MenuBar.vue` 的脚本菜单里，MSG / STD 分组各加一项：

```ts
toolItem('thmsg', '导出原始 .dmsg（供命令行使用）', 'script.exportRawDmsg', !activeIsDmsg.value),
```

`handleSelect` 里对应分支调用 API（弹保存对话框选输出路径，参照既有的 `runPackDat` 取路径的写法）。

- [ ] **Step 5: 跑门禁并提交**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck && npm test && npm run build
git add -A
git commit -m "feat(msg,std): 显式声明 .dmsg/.dstd 是 IDE 方言，并提供导出原始格式

thmsg/thstd 没有 -m，指令名映射由 IDE 在工具外面做，磁盘上的 .dmsg/.dstd
因此不是合法的 thmsg/thstd 输入——直接跑 thmsg -c 会在每条带名字的指令上
失败。此前这件事没有任何地方说明。

仍在磁盘上存名字是因为 git diff 的可读性差距是决定性的；代价是必须把方言
声明写进文件本身，并提供脱离 IDE 的导出通道。"
```

---

## Task 2：mapfile 解析器提到 common，验证可吃 `.msgm` / `.stdm`

**Files:**
- Create: `src-tauri/src/common/map_file.rs`
- Modify: `src-tauri/src/common/mod.rs`
- Modify: `src-tauri/src/modules/ecl/map_parser.rs`（改为转调 common）

**Interfaces:**
- Produces：`pub struct MapFileData { pub source_path: String, pub instructions: Vec<MapInstruction>, pub globals: Vec<MapGlobal> }`、`pub fn parse_map_content(path: &str, content: &str) -> Result<MapFileData, String>`。
- Consumes：无。

> **不改解析逻辑，只搬家。** 既有 `parse_ecl_map_content` 对未知 `!` 段优雅跳过，因此 `!msgmap` / `!stdmap` 头会被正确忽略——这一点由 Step 1 的测试钉住。

- [ ] **Step 1: 写失败测试**

新建 `src-tauri/src/common/map_file.rs`，先只写测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// 取自 truth 的 map/th11.msgm，格式与 eclmap 同族。
    const TH11_MSGM: &str = "\
!msgmap
!ins_names

0 end
1 playerShow
3 textboxShow   # nop in TH128+
";

    const TH14_STDM: &str = "\
!stdmap

# STD - TH14 to TH17

!ins_names
0 stop
1 jmp
2 pos
";

    /// 关键：`!msgmap` / `!stdmap` 这些未知段头必须被跳过而不是报错，
    /// 否则同一个解析器吃不下三种 mapfile。
    #[test]
    fn parses_msgm_despite_unknown_section_header() {
        let data = parse_map_content("th11.msgm", TH11_MSGM).expect("应能解析");
        let names: Vec<&str> = data.instructions.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["end", "playerShow", "textboxShow"]);
    }

    #[test]
    fn parses_stdm_and_ignores_comments() {
        let data = parse_map_content("th14.stdm", TH14_STDM).expect("应能解析");
        assert_eq!(data.instructions.len(), 3);
        assert_eq!(data.instructions[1].opcode, 1);
        assert_eq!(data.instructions[1].name, "jmp");
    }

    #[test]
    fn trailing_hash_comment_is_not_part_of_the_name() {
        let data = parse_map_content("x.msgm", TH11_MSGM).unwrap();
        let textbox = data.instructions.iter().find(|i| i.opcode == 3).unwrap();
        assert_eq!(textbox.name, "textboxShow", "行尾注释不能混进名字");
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml map_file
```

预期：`cannot find function parse_map_content`。

- [ ] **Step 3: 搬运实现**

把 `modules/ecl/map_parser.rs` 中 `parse_ecl_map_content` 的**函数体原样**搬到 `common/map_file.rs` 的 `parse_map_content`，同时把它依赖的三个正则辅助（`instruction_line_regex` / `signature_line_regex` / `gvar_line_regex`）和 `build_signature_params` 一并搬过来。结构体改名：

- `EclMapInstructionSpec` → `MapInstruction`（字段不变）
- `EclMapGlobalVar` → `MapGlobal`（字段不变）
- `EclMapSemanticData` → `MapFileData`，**去掉** `version` 字段（版本归属改由 gamemap 决定，见 Task 3）

`common/mod.rs` 加 `pub mod map_file;`。

`modules/ecl/map_parser.rs` 改为转调，并保留它自己的 `EclMapSemanticData`（含 `version`、`builtins`）以免动到前端契约：

```rust
pub fn parse_ecl_map_content(path: &str, content: &str) -> Result<EclMapSemanticData, String> {
    let data = crate::common::map_file::parse_map_content(path, content)?;
    Ok(EclMapSemanticData {
        source_path: data.source_path,
        version: infer_version_from_path(path),
        instructions: data.instructions,
        builtins: Vec::new(),
        globals: data.globals,
    })
}
```

> 若 `EclMapSemanticData` 的 `instructions` / `globals` 字段类型与 common 的不同名，在 `modules/ecl/map_parser.rs` 里 `pub use crate::common::map_file::{MapInstruction as EclMapInstructionSpec, MapGlobal as EclMapGlobalVar};` 保持前端契约不变。

- [ ] **Step 4: 跑全套确认无回归**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

预期：全绿。ECL 既有的 map 相关测试必须仍然通过——这是"只搬家不改逻辑"的证明。

- [ ] **Step 5: 提交**

```bash
git add -A
git commit -m "refactor(map): mapfile 解析器提到 common，供三种 mapfile 共用

parse_ecl_map_content 对未知 ! 段本来就优雅跳过，所以同一套逻辑能直接吃
.msgm / .stdm。搬到 common 并加测试钉住这一点，ECL 侧改为转调、前端契约不变。"
```

---

## Task 3：数据迁移到 `.msgm` / `.stdm` + gamemap 间接

**Files:**
- Create: `src-tauri/assets/maps/any.msgm`、`any.stdm`、`th11.msgm`、`th14.stdm`
- Delete: `src-tauri/assets/msg-th17.json`、`std-th17.json`
- Modify: `src-tauri/src/modules/msg/map_parser.rs`、`src-tauri/src/modules/thstd/map_parser.rs`
- Create: `src-tauri/src/common/game_map.rs`

**Interfaces:**
- Produces：`pub fn resolve_map_file(gamemap: &str, version: u32) -> Option<String>`。
- Consumes：Task 2 的 `parse_map_content`。

**背景：** 现在两个 `map_parser` 都写着 `let _ = version;`——**任何版本都返回 th17 数据**。结果碰巧对（thstd 的 th14–th20 同档、thmsg 的名字到 th18.5 一致），但理由是错的。gamemap 把"哪些版本共用哪份表"变成**数据**。

- [ ] **Step 1: 写失败测试**

新建 `src-tauri/src/common/game_map.rs`：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const ANY_STDM: &str = "\
!gamemap
!game_files
6   th06.stdm
14  th14.stdm
17  th14.stdm
18  th14.stdm
185 th14.stdm
";

    #[test]
    fn maps_version_to_shared_file() {
        assert_eq!(resolve_map_file(ANY_STDM, 17).as_deref(), Some("th14.stdm"));
        assert_eq!(resolve_map_file(ANY_STDM, 185).as_deref(), Some("th14.stdm"));
        assert_eq!(resolve_map_file(ANY_STDM, 6).as_deref(), Some("th06.stdm"));
    }

    /// 关键：查不到就是查不到，**不许**回退到任意一份表。
    /// 这正是 `let _ = version;` 的错误——靠只有一份数据蒙对。
    #[test]
    fn unmapped_version_returns_none_instead_of_guessing() {
        assert_eq!(resolve_map_file(ANY_STDM, 20), None);
    }

    #[test]
    fn ignores_comments_and_section_headers() {
        let with_noise = format!("{ANY_STDM}# NEWHU: 185\n");
        assert_eq!(resolve_map_file(&with_noise, 17).as_deref(), Some("th14.stdm"));
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml game_map
```

- [ ] **Step 3: 写 gamemap 解析**

```rust
//! gamemap：把游戏版本映射到共享的 mapfile。
//!
//! 格式取自 truth 的 `map/any.msgm`，这样我们的表和它的表可以互换：
//!
//! ```text
//! !gamemap
//! !game_files
//! 17  th14.stdm
//! ```
//!
//! 存在的意义是把「哪些版本共用哪份表」变成**数据**。此前两个 map_parser 都写着
//! `let _ = version;`，任何版本都返回 th17 的表——结果碰巧对（thstd 的 th14–th20
//! 共用 formats_v2），但理由是错的：它靠"只有一份数据"蒙对，将来 ZUN 分了新档
//! 会继续静默返回旧表。

use std::sync::OnceLock;
use regex::Regex;

fn entry_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\d+)\s+(\S+)").expect("valid regex"))
}

/// 查不到返回 None——**不做任何回退**。调用方必须把"这个版本没有数据"如实告诉用户。
pub fn resolve_map_file(gamemap: &str, version: u32) -> Option<String> {
    for raw in gamemap.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some(caps) = entry_regex().captures(line) {
            if caps[1].parse::<u32>().ok() == Some(version) {
                return Some(caps[2].to_string());
            }
        }
    }
    None
}
```

`common/mod.rs` 加 `pub mod game_map;`。

- [ ] **Step 4: 生成 mapfile 资产**

用脚本把现有 JSON 转成 mapfile（一次性，转完删 JSON）：

```bash
cd /data/sunyunbo/www/THTK-Studio-2
mkdir -p src-tauri/assets/maps
python3 - <<'PY'
import json, pathlib
for tool, ext, header in [("msg", "msgm", "!msgmap"), ("std", "stdm", "!stdmap")]:
    src = pathlib.Path(f"src-tauri/assets/{tool}-th17.json")
    data = json.loads(src.read_text(encoding="utf-8"))
    base = "th11" if tool == "msg" else "th14"
    lines = [header, "", "!ins_names"]
    for ins in sorted(data["instructions"], key=lambda i: i["opcode"]):
        lines.append(f"{ins['opcode']} {ins['name']}")
    out = pathlib.Path(f"src-tauri/assets/maps/{base}.{ext}")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("写出", out)
PY
```

手写两个 gamemap。`src-tauri/assets/maps/any.msgm`：

```text
!gamemap
!game_files
11  th11.msgm
12  th11.msgm
128 th11.msgm
13  th11.msgm
14  th11.msgm
143 th11.msgm
15  th11.msgm
16  th11.msgm
165 th11.msgm
17  th11.msgm
18  th11.msgm
185 th11.msgm
```

> th19 / th20 **故意不列**——thmsg 的 `th19_msg_fmts[]` 有 13 条我们没有的指令，
> 列上去等于谎称覆盖。Task 5 补全后再加。
> th06–th10 同样暂缺（`th06.msgm` / `th10.msgm` 未迁移），需要时从 truth 的
> `map/` 取（Apache-2.0）。

`src-tauri/assets/maps/any.stdm`：

```text
!gamemap
!game_files
14  th14.stdm
143 th14.stdm
15  th14.stdm
16  th14.stdm
165 th14.stdm
17  th14.stdm
18  th14.stdm
185 th14.stdm
19  th14.stdm
20  th14.stdm
```

> th19 / th20 **可以列**：thstd 的 `formats_v2` 覆盖 th14–th20 且无 per-version
> 分支，指令签名表完全相同。这是与 MSG 的关键差异。

- [ ] **Step 5: 改两个 map_parser，去掉静默回退**

`modules/msg/map_parser.rs` 整体改为：

```rust
use crate::common::{game_map, map_file};

const GAMEMAP: &str = include_str!("../../../assets/maps/any.msgm");
const TH11: &str = include_str!("../../../assets/maps/th11.msgm");

/// 按版本取 MSG 语义数据。**查不到就报错**，不回退到任意一份表。
pub fn parse_msg_semantics(version: &str) -> Result<map_file::MapFileData, String> {
    let id: u32 = version
        .trim()
        .parse()
        .map_err(|_| format!("版本号非法: {version:?}"))?;

    let file = game_map::resolve_map_file(GAMEMAP, id).ok_or_else(|| {
        format!(
            "尚无 th{id} 的 MSG 指令表。thmsg 对 th19/th20 有独立的签名表\
             （th19_msg_fmts，新增 13 条指令），我们尚未补全；\
             指令会以 ins_N 原样显示。"
        )
    })?;

    match file.as_str() {
        "th11.msgm" => map_file::parse_map_content("th11.msgm", TH11),
        other => Err(format!("gamemap 指向未内置的表: {other}")),
    }
}
```

`modules/thstd/map_parser.rs` 同构（`any.stdm` / `th14.stdm`，错误文案改成 STD 的）。

> ⚠️ **返回类型变了，翻译器签名要跟着改。** 现有的
> `dmsg_to_readable(raw, &MsgSemanticData, bool)` / `readable_to_dmsg(&str, &MsgSemanticData)`
> 收的是旧结构。两个选择：
>
> - **推荐**：把 `MsgSemanticData` / `StdSemanticData` 整个删掉，翻译器改收
>   `&map_file::MapFileData`。两者字段用途一致（`instructions: Vec<{opcode, name, ...}>`），
>   删掉重复类型是净收益。
> - 保守：在 map_parser 里把 `MapFileData` 转回 `MsgSemanticData`。多一层无谓映射，
>   除非 `MsgSemanticData` 上有 `MapFileData` 没有的字段，否则不要这么做。
>
> 先确认 `MsgSemanticData` 是否有额外字段（`description` 中文说明可能只在旧结构上）：
> 若有，说明中文说明需要单独的旁挂机制——**那属于 Task 3 的范围**，在本 Step 一并处理：
> mapfile 只放 `opcode name`，中文说明放 `assets/maps/th11.msgm.zh.json`（`{"3": "显示对话框。"}`），
> 由 map_parser 加载后合并。这样 mapfile 本体与生态兼容，中文是我们的增量。

调用方（`modules/msg/compiler.rs`、`modules/thstd/compiler.rs`）此前把解析失败当致命错误。**改为降级**：拿不到语义就跳过翻译、原样输出 `ins_N`，并在结果 message 里附上原因。找到 `parse_msg_semantics` 的调用点：

```rust
let semantics = match super::map_parser::parse_msg_semantics(&version) {
    Ok(s) => s,
    Err(e) => return fail(request, format!("Failed to load msg semantics: {e}")),
};
```

改为：

```rust
// 拿不到语义不该让整个解包失败——用户仍然需要看到 ins_N 形式的内容。
// 但必须说清楚为什么没有名字。
let (semantics, semantics_note) = match super::map_parser::parse_msg_semantics(&version) {
    Ok(s) => (Some(s), None),
    Err(e) => (None, Some(e)),
};
let readable = match &semantics {
    Some(s) => super::translator::dmsg_to_readable(&raw_dmsg, s, request.with_comments),
    None => raw_dmsg.clone(),
};
```

并在成功消息里追加 `semantics_note`（参照 Task 1 里 `mojibake_warning` 的拼接方式）。

- [ ] **Step 6: 跑门禁并提交**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck && npm test && npm run build
git add -A
git commit -m "refactor(map): 语义数据改用 .msgm/.stdm + gamemap，去掉静默回退

此前两个 map_parser 都写着 let _ = version;——任何版本都返回 th17 的表。
结果碰巧对（thstd 的 th14–th20 共用 formats_v2、thmsg 的名字到 th18.5 一致），
但理由是错的：靠'只有一份数据'蒙对，将来分了新档会继续静默返回旧表。

改用 truth 的 gamemap 格式把版本分档变成数据，两边 mapfile 可互换。
查不到版本时不再回退，而是降级为 ins_N 原样输出并说明原因——
th19/th20 的 MSG 正处于这个状态（thmsg 的 th19_msg_fmts 有 13 条我们没有）。"
```

---

## Task 4：识别 STD 跳转指令（导航的第一步）

**Files:**
- Create: `src/services/languages/std/jumpNavigation.ts`
- Create: `tests/unit/stdJumpNavigation.spec.ts`

> **范围**：本 Task 只做**识别**，产出可测的纯函数。偏移→行号的换算与 Monaco
> 接线（可点击链接、code lens）是下一轮——那需要指令计数规则，值得单独一批。

**Interfaces:**
- Produces：`export function findJumpTargets(text: string): JumpLink[]`，`interface JumpLink { sourceLine: number; targetLine: number; offset: number }`。
- Consumes：无（纯文本分析）。

**背景：** thstd 的跳转是裸字节偏移。在文本里生成 `goto label` 需要复刻 thstd 对每条指令的大小计算，任何一处对不上就是**静默产出错误跳转**。而用户要的是"看得懂跳到哪"，这一点 IDE 不改文件格式就能给。

- [ ] **Step 1: 写失败测试**

```ts
import { describe, expect, it } from 'vitest'
import { findJumpTargets } from '../../src/services/languages/std/jumpNavigation'

/** 时间标签形如 `120:`，跳转形如 `jmp(time, offset)`；offset 是**行内偏移**。 */
const SAMPLE = [
  '0:',                    // line 1
  '    pos(0f, 0f, 0f)',   // line 2
  '60:',                   // line 3
  '    jmp(0, 1)',         // line 4 → 目标是第 1 条指令
  '    stop()'             // line 5
].join('\n')

describe('findJumpTargets', () => {
  it('把 jmp 的偏移解析成源行与目标行', () => {
    const links = findJumpTargets(SAMPLE)
    expect(links).toHaveLength(1)
    expect(links[0].sourceLine).toBe(4)
    expect(links[0].offset).toBe(1)
  })

  it('没有 jmp 时返回空', () => {
    expect(findJumpTargets('    pos(0f, 0f, 0f)\n    stop()')).toEqual([])
  })

  it('忽略注释行里的 jmp', () => {
    expect(findJumpTargets('    // jmp(0, 1)')).toEqual([])
  })

  it('容忍 ins_1 未被翻译的情形', () => {
    const links = findJumpTargets('    ins_1(1, 0)')
    expect(links).toHaveLength(1)
  })
})
```

- [ ] **Step 2: 跑测试确认失败**

```bash
npm test -- stdJumpNavigation
```

- [ ] **Step 3: 写实现**

```ts
/**
 * STD 跳转导航。
 *
 * thstd 的跳转目标是裸偏移，没有符号标签。要在文本里生成 `goto label` 必须复刻
 * thstd 对每条指令的大小计算——任何一处对不上就是静默产出错误的跳转目标。
 *
 * 用户真正要的是"看得懂跳到哪"，这一点在编辑器层给就够了，而且**比真 label 更好**：
 * 不改文件格式，`.dstd` 仍然能经翻译回去喂给 thstd。
 */
export interface JumpLink {
  /** 1 起的源行号 */
  sourceLine: number
  /** 1 起的目标行号；解析不出时为 0 */
  targetLine: number
  /** 指令中写的偏移实参 */
  offset: number
}

// `jmp(time, offset)`（我们翻译后的形式）或 `ins_1(offset, time)`（未翻译的原始形式）
const JUMP_RE = /^\s*(?:[+\-]?\d+|@\w+:)?\s*(jmp|ins_1)\s*\(\s*(-?\d+)\s*,\s*(-?\d+)\s*\)/

export function findJumpTargets(text: string): JumpLink[] {
  const lines = text.split('\n')
  const links: JumpLink[] = []

  lines.forEach((line, index) => {
    if (line.trim().startsWith('//') || line.trim().startsWith('#')) return
    const match = JUMP_RE.exec(line)
    if (!match) return

    // 参数顺序不同：翻译后是 (time, offset)，原始是 (offset, time)
    const offset = match[1] === 'jmp' ? Number(match[3]) : Number(match[2])
    links.push({ sourceLine: index + 1, targetLine: 0, offset })
  })

  return links
}
```

> `targetLine` 暂留 0：把偏移换算成行号需要指令计数规则，属于下一轮工作。
> 本 Task 先把**识别**做对并钉住，Monaco 侧先只显示"跳转到偏移 N"的悬停提示。

- [ ] **Step 4: 跑测试确认通过**

```bash
npm test -- stdJumpNavigation
npm run typecheck
```

- [ ] **Step 5: 提交**

```bash
git add -A
git commit -m "feat(std): 识别 STD 跳转指令，为导航做准备

thstd 的跳转是裸偏移。在文本里生成 goto label 需要复刻 thstd 的偏移计算，
任何一处对不上就是静默产出错误跳转——因此走编辑器导航而非改文件格式。
本次先把识别做对（同时认翻译后的 jmp(time,offset) 与原始 ins_1(offset,time)，
两者参数顺序相反），偏移→行号的换算下一轮做。"
```

---

## Task 5：补全 th19 / th20 的 MSG 指令表 —— ⏸ 暂缓（2026-08-09 决定）

**Files:**
- Modify: `src-tauri/assets/maps/th19.msgm`（新建）
- Modify: `src-tauri/assets/maps/any.msgm`

**这个 Task 需要逆向工作，不能纯靠代码完成。** 它被排在最后，因为前四个 Task 都不依赖它。

> **状态（2026-08-09）：名字已从上游取得，本 Task 关闭。Step 2 的逆向不必做了。**
>
> 原计划假定「没有二手来源，只能从零逆向」。先查 ExpHP/truth 证实了这一半：
> `map/` 里最新的 MSG 表是 `th11.msgm`，没有 `th19.msgm`；`any.msgm` 的 gamemap
> 停在 `185`（末尾标记 `# NEWHU: 185`）；MSG 侧自 2022-08-18 的
> `add missing gamemap entries for BM`（BM = th185）起再无变动。
>
> 但 **zero318/TouhouMaps** 有（Unlicense，公有领域）：`th18/th185/th19/th20.msgm`
> 齐全，另有 `th20.stdm`、`th20.anmm`、`any.endm`/`th20.endm`。缺的 16 条名字全在
> 那里，他为 th20 提交过两次（2025-08-25 / 08-31）。
>
> 实际落地时与原计划有三处偏离，都记在这里：
>
> 1. **只取名字，不取签名。** zero318 用的是 truth 的扩展签名语法
>    （`S(enum="PortraitIndex")`、`z(bs=4;mask=0x77,7,16;furibug)`），我们的解析器
>    不认；而且他的签名与 thtk 有冲突（他说 th18/th20 的 opcode 19 无参，thmsg 的
>    `th16_msg_fmts` 说是 `S`）。签名改为**按 `th06_find_format()` 的 fallthrough 链
>    逐条算出来**，一手数据，且「表里的 opcode 集合」= 「thmsg 解得动的集合」。
>
> 2. **名字原样保留 snake_case 与 `__` 前缀**，不改写成我们的 camelCase。`__` 是
>    zero318 自己的「未坐实」标记，保留它等于把可信度写进名字：`textboxShow` 是
>    确认过的，`__focus_current_side` 是工作假设，用户一眼能分辨。代价是同一张表里
>    两种命名风格并存——可以接受，因为风格差异恰好对应可信度差异。
>
> 3. **th19 与 th20 拆成两张表**，与原计划的「19/20 共用一份」相反。thtk 的
>    `th06_find_format()` 把 `case 20:` 和 `case 19:` 并成一个分支，但那是
>    `/* NEWHU: 20 */` 加新版本时的顺手分组，没验证过——对 thmsg 无害，因为 th20
>    文件里根本不出现那些 opcode。zero318 逐条整理的 th20 表只到 36：42–56 是 th19
>    作为对战作特有的左右阵营指令（`__focus_current_side` / `__opposite_side`），
>    常规 STG 用不上。冲突时取窄的一边：th20 真出现 42–56 就显示 `ins_N`，不显示一个
>    可能张冠李戴的名字。
>
> **顺带发现的 thtk 缺口（可提 issue）：** zero318 的表里有 opcode 40、41、48、49
> （40/41 是字符串，48/49 是 `S`），thmsg 的**任何**格式表都没有它们。所以 th19 /
> th185 的 `.msg` 里一旦出现 48，`thmsg -d` 会直接报
> `id 48 was not found in the format table` 而失败。这是 thtk 的问题，不是我们的。
>
> Step 4「回馈上游」仍然成立，但对象变了：给 ExpHP/truth 提的应当是 zero318 已有的
> th19/th20 数据（两人格式同源），或者干脆只提 thtk 那四条缺失的格式表项。

**已知条件**（来自 `thmsg/thmsg06.c`，无需再查）：

```
th185_msg_fmts[]:  37 ""   38 ""   39 ""
th19_msg_fmts[]:   42 "S"  43 "S"  44 "ff"  45 "ff"  46 "SS"  47 "SS"
                   50 "S"  51 "S"  52 ""    53 ""    54 ""    55 ""   56 ""
```

**签名是白送的，缺的只有名字。** 格式串含义：`S` = int32，`f` = float，`""` = 无参数。

- [ ] **Step 1: 建一份只有签名、没有名字的表**

先让 IDE 至少知道参数形状（避免把 `ins_44(1.0, 2.0)` 误当无参指令）：

```text
!msgmap

# MSG - TH19 / TH20
# 签名取自 thtk thmsg/thmsg06.c 的 th19_msg_fmts[]；名字待逆向确认。
# 未命名的指令会以 ins_N 显示，这是有意的——宁可显示编号，
# 也不要编一个可能是错的名字。

!ins_signatures
37
38
39
42 S
43 S
44 ff
45 ff
46 SS
47 SS
50 S
51 S
52
53
54
55
56
```

- [ ] **Step 2: 逆向确认名字**

沿用 `research/msg/` 的方法：在 Ghidra 里定位 th19/th20 的 `GuiMsgVm::run`，对照 opcode 分支的行为命名。每确认一条就往 `!ins_names` 加一行。

**证据链纪律**（见项目记忆）：每条结论记录 发现 → 推测 → 验证 → 结论（可信度 + 版本）→ 证据（地址 / 出处）。一手反汇编 > 推断 > 社区单源。

- [ ] **Step 3: 名字补齐后，把 th19/th20 加进 gamemap**

`any.msgm` 追加：

```text
19  th19.msgm
20  th19.msgm
```

同时 `th19.msgm` 需要 `!ins_names` 段包含 th11 那份的全部条目（或实现 mapfile 继承）——**先用复制，别引入继承机制**，除非确实出现第三份需要继承的表。

- [ ] **Step 4: 回馈上游**

把 `th19.msgm` 和 `any.stdm` 的 th19/th20 两行提给 [ExpHP/truth](https://github.com/ExpHP/truth)。他 2026-03 仍在做 `trumsg --ending`，且格式已经对齐，接受概率远高于给 thtk 提 PR（thtk 的 msg/std 八年无人问津，issue #49 零评论）。

---

## 最终验证

```bash
npm test && npm run typecheck && npm run build
```

```bash
P=/data/sunyunbo/miniconda3/envs/tauri-dev
export PKG_CONFIG_PATH=$P/lib/pkgconfig:$P/share/pkgconfig LD_LIBRARY_PATH=$P/lib PATH=$P/bin:$PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows 手动验收（追加到 MVP 清单）

1. 解包一个 `.msg`，产物开头有方言声明注释；把它直接喂给命令行 `thmsg -c` 会失败。
2. 「导出原始 .dmsg」产出的文件不含方言头，命令行 `thmsg -c` 能成功编译。
3. 项目版本设为 th20 解包 `.msg`：输出面板出现「尚无 th20 的 MSG 指令表」说明，
   内容以 `ins_N` 显示且**不报错**。
4. 项目版本设为 th17 解包 `.msg`：指令名正常显示，无任何缺表提示。
5. 项目版本设为 th20 解包 `.std`：指令名**正常显示**（`formats_v2` 覆盖 th14–th20）。
6. **错误行号对得上**：在 `.dmsg` 第 N 行故意写一个不存在的指令名，打包失败卡片
   指的应当是第 N 行。thmsg 看到的是临时文件，行号靠逐行 1:1 替换保证对齐，
   此前从未实测。
