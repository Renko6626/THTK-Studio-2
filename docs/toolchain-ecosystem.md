# 工具链生态与可读性分层

**日期：** 2026-08-05
**状态：** 已决策，待实施（见 `superpowers/plans/2026-08-05-msg-std-ergonomics.md`）

本文回答一个问题：**东方 modding 有 thtk 和 truth 两套工具链，THTK-Studio 该怎么站位。**

结论先行：

> **thtk 当编解码器，mapfile 当数据契约，IDE 当可读性层，truth 当数据源和设计参照。**
> 不 fork 任何一方、不要求用户装第二套工具链、不等任何人更新。

---

## 1. 事实依据

以下全部来自一手核对（thtk / truth 源码、官方 man page、thcrap 文档），不是推断。

### 1.1 thtk 的 `-m` 支持是**不对称**的

| 工具 | Synopsis | map 文件 |
| --- | --- | --- |
| thecl | `[-Vrsxj] [[-c\|-h\|-d] ver] [-m eclmap]... [in [out]]` | ✅ |
| thanm | `[-Vfouv] [[-l\|-x\|-X\|-r\|-c] ver] [-m anmmap]... [-s syms] [archive...]` | ✅ |
| **thmsg** | `[-Ve] [[-c\|-d] ver] [in [out]]` | ❌ |
| **thstd** | `[-V] [-c\|-d ver] [in [out]]` | ❌ |
| thdat | `[-Vg] [-C dir] [[-c\|-l\|-x] [d\|ver]] [archive [file...]]` | ❌（无脚本语义） |

thstd 的 synopsis 短到**一个选项都没有**。它的输出有**时间标签**（`123:`），但跳转目标是**裸字节偏移**，没有跳转标签。

### 1.2 thtk 覆盖 th06–th20，且 msg/std 的格式极其稳定

`thstd.c` 把版本映射为三档格式，**无任何 per-version 分支**：

```c
case 6: case 7: case 8: case 9: case 95:                    option_version = 0;  // formats_v0
case 10: case 103: case 11: case 12: case 125: case 128: case 13:
                                                             option_version = 1;  // formats_v1
case 14: case 143: case 15: case 16: case 165:
case 17: case 18: case 185: case 19: case 20:                option_version = 2;  // formats_v2
```

**th14–th20 共用 `formats_v2`，指令签名表完全相同。** 所以 th17 的 STD 语义数据对 th20 是正确的。

`thmsg.c` 的**模块**共用（`th06_msg` 覆盖 th06–th20，仅 th95 单列、th125 按 `-e` 分支），但 `th06_find_format()` 是**级联穿透**的，**签名表按版本组分开**：

```c
switch (version) {
case 20: case 19:
    if ((ret = find_format(th19_msg_fmts, id))) break;   // ← th19/20 独有
case 185:
    if ((ret = find_format(th185_msg_fmts, id))) break;
...
}
```

- `th185_msg_fmts[]`：新增 id 37, 38, 39
- `th19_msg_fmts[]`：新增 id 42–47, 50–56（13 条）

> ⚠️ **MSG 与 STD 在这一点上不同。** 早期分析曾把 MSG 也判为"格式万年不变"，那是错的——模块共用不等于签名表共用。

### 1.3 truth 的现状

[ExpHP/truth](https://github.com/ExpHP/truth)（"touhou rust thing"，Apache-2.0）是 thtk 的 Rust 重写：

- `trustd` / `truanm` / `trumsg` 覆盖 **TH06–TH185**；`truecl` 仍是原型（仅 TH06–TH095）
- **全部工具支持 `-m`**，另有 `TRUTH_MAP_PATH` 自动发现
- 有**符号跳转标签**：`goto lol;` / `goto lol @ 123;`（快照测试可证）
- mapfile 用 **gamemap 间接**：`any.msgm` 把版本映射到共享表，`th11.msgm` 一份服务 th11–th185
- **不支持 th19/th20**（代码库搜索 0 结果，gamemap 顶到 `# NEWHU: 185`；2026-08-09
  复查仍然如此，`map/` 的 MSG 部分自 2022-08 起未动）——这一缺口由 §3 的第三个
  数据源补上
- 最后 release **2022-08-18**，但主干活跃（2026-03 仍在做 `trumsg --ending`）；`CHANGELOG.md` 主干留有未解决的合并冲突标记

**我们的指令命名与 truth 逐条一致**（两边都源自 ExpHP 的 thpages 文档），所以格式迁移无损。

### 1.4 编码：原版游戏不支持 Unicode

thcrap 官方文档原文：

> because the original games **don't support any form of Unicode**, we need to somehow inject some Unicode into them
>
> the game uses a NUL byte as a string delimiter (because it works with plain C strings in **SHIFT-JIS** encoding)

thcrap 的多语言靠**运行时把 Win32 `A` 函数换成 `U` 版本**（`TextOutExA` → `TextOutExU`），**不改 `.msg` 文件**。因此 UTF-8 的 `.msg` 原版读不了；汉化走的是改 `CreateFontIndirectA` 的 `lfCharSet` + 转区那条老路，GBK 可用但**需游戏侧已适配**。

---

## 2. 决策

### 2.1 thtk 是唯一必需后端；truth **不做**运行时后端

理由不是"truth 覆盖不到 th20"，而是更根本的一条：

> **truth 的输出是另一种源语言，不是同一格式的另一个后端。**

```
thstd  反编译 → ins_1(1200, 60)     只能喂回 thstd
trustd 反编译 → goto lol @ 123;     只能喂回 trustd
```

两个后端并存会让项目里的 `.dstd` **分裂成两种方言**：扩展名相同、语法高亮要分叉、翻译层要判断来源。代价远超收益。

加之 truth 覆盖不到主力版本 th20、2022 年后无 release（用户须自行构建）。

**truth 的正确用法是数据来源与设计参照，用户无需安装。**

### 2.2 可读性分层

| 层 | 归属 | 说明 |
| --- | --- | --- |
| 编解码 | thtk | th06–th20 五种格式完整 |
| 语义数据 | `.msgm` / `.stdm` / `.eclm` / `.anmm`，外挂文件 | eclmap 家族格式 + gamemap 间接 |
| 可读性 | **IDE** | 名字映射、跳转导航、悬停签名、补全 |
| 回馈 | 提给 thtk | thmsg 格式表缺的 opcode 40/41/48/49（见 §3） |

**ECL / ANM 用 thtk 原生 `-m`，MSG / STD 用 IDE 自己的翻译层。** 这不是妥协，是对 §1.1 那张表的正确响应。

### 2.3 `.dmsg` / `.dstd` 是 **IDE 方言**（已采纳）

这是 §2.2 的直接后果，必须显式承认：

```
解包: thmsg -d → ins_3(0) → 翻译 → 磁盘存 textboxShow(0)
打包: 磁盘 textboxShow(0) → 翻译回 → 临时文件 ins_3(0) → thmsg -c
                                      ↑ thmsg 只见过这个
```

**thmsg / thstd 永远只看到 `ins_N`。** 因此项目里的 `.dmsg` / `.dstd` **不是合法的 thmsg / thstd 输入**——直接跑 `thmsg -c 20 st01.dmsg out.msg` 会在每条带名字的指令上失败。

ECL 没有这个问题：thecl 认识 eclmap 里的名字，`.decl` 本身就是合法输入。

**为什么仍然选择在磁盘上存名字**：`git diff` 里 `textboxShow(0)` 与 `ins_3(0)` 的可读性差距是决定性的。代价是需要显式声明方言并提供导出通道（见计划 Task 1）。

被否决的替代方案：

- **磁盘存 `ins_N`、仅在编辑器里显示名字**——可逆性风险并未消失，只是从"保存时"挪到"显示时"，且 Monaco 要做虚拟文档映射，复杂度显著上升，还牺牲了 git diff 可读性。
- **不翻译**——等于放弃 IDE 在 msg/std 上唯一能提供的价值。

### 2.4 跳转标签用**导航**而非写入文件

thstd 的跳转是裸偏移。要在文本里生成 `goto label`，必须复刻 thstd 对每个版本每条指令的大小计算——任何一处对不上就是**静默产出错误的跳转目标**。

而用户要的是"看得懂跳到哪"，不是"文件里有 label 这个字符串"。因此：

- `jmp(time, offset)` 的 offset 做成**可点击链接**，跳到目标行
- 目标行加 **code lens**：「← 被第 42 行跳转」

零风险、任何版本通用，且**比真 label 更好**——真 label 写进 `.dstd` 后连 thstd 都喂不回去了。

---

## 3. 第三个数据源：zero318/TouhouMaps

> 本节修订于 2026-08-09。此前这里写着「两个生态都没有 th19/th20 的 MSG 指令名，
> 补全这 16 条是填真空」——**前半句对，后半句错**：只查了 thtk 和 truth，漏了第三家。

[zero318/TouhouMaps](https://github.com/zero318/TouhouMaps)（**Unlicense，公有领域**）
是 thcrap 维护者 zero318 的私人 mapfile 集，格式与 truth 同源。它覆盖到 th20：
`th18/th185/th19/th20.msgm`、`th20.stdm`、`th20.anmm`，另有 truth 和 thtk 都没有的
`any.endm`/`th20.endm`（结局 msg 单独一套）。

三家的分工因此是：

| 来源 | 提供 | 覆盖 | 可信度 |
| --- | --- | --- | --- |
| thtk `thmsg06.c` | **签名**（`id_format_pair_t` 表） | th06–th20 | 一手，是解码器本身 |
| ExpHP/truth `map/` | 名字（camelCase） | 至 th185 | 高，与 thpages 文档同源 |
| zero318/TouhouMaps | 名字（snake_case） | 至 th20 | **他自己用 `__` 前缀标未坐实** |

我们的 `th19.msgm` / `th20.msgm` 就是这么拼出来的：**签名按 `th06_find_format()` 的
fallthrough 链逐条算**（于是「表里的 opcode 集合」= 「thmsg 解得动的集合」），
opcode 0–36 的名字用 truth 体系，37 起用 zero318 的并**原样保留 `__` 前缀**——
可信度写在名字里，比写在注释里更难被忽略。

两处需要判断的冲突，以及我们的取舍：

- **th20 是否与 th19 共表。** thtk 的 `case 20:` / `case 19:` 是同一分支，zero318 的
  th20 表只到 36。取 zero318：42–56 是 th19 作为对战作特有的左右阵营指令，常规 STG
  用不上；thtk 那个分组是 `/* NEWHU: 20 */` 加版本时的顺手之举，对它自己无害。
  取窄的一边，宁可显示 `ins_N`。
- **签名不采用 zero318 的。** 他的 th18/th20 opcode 19 标成无参，而 thmsg 的
  `th16_msg_fmts` 说是 `S`；且他用 truth 的扩展语法（`S(enum="PortraitIndex")`、
  `z(bs=4;mask=0x77,7,16;furibug)`），我们的解析器不认。签名一律以 thtk 为准。

**thtk 的真缺口（可提 issue）：** zero318 的表里有 opcode 40、41、48、49
（40/41 是字符串，48/49 是 `S`），thmsg 的任何格式表都没有。th19/th185 的 `.msg`
里一旦出现 48，`thmsg -d` 会报 `id 48 was not found in the format table` 直接失败。
这比补名字更值得回馈——thtk 的 msg/std 已八年无人问津（issue #49「Notes and
suggestions on thstd」2018 年提出，零评论），但格式表缺项是硬 bug。

---

## 4. 相关文档

- 实施计划：`superpowers/plans/2026-08-05-msg-std-ergonomics.md`
- 版本权威表：`src-tauri/src/common/game_version.rs`
- 编码处理：`src-tauri/src/common/text_encoding.rs`
- TH16 MSG 逆向：`research/msg/`
