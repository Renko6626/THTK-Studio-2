# 来源与维护附录

本手册按当前编译接口重新组织知识，供离线写代码使用。下面保留固定版本、取证位置和还未解释的行为，便于后续维护时追溯。核对日期为 2026-11-09。

## 编译器语法、格式与有效签名

| 来源 | 固定版本与入口 | 用于哪些内容 |
| --- | --- | --- |
| zero318/truth（源自 ExpHP/truth） | [`c78a541d1fb09fcefd448c83b097d4f9b815fce3`](https://github.com/zero318/truth/tree/c78a541d1fb09fcefd448c83b097d4f9b815fce3) | `doc/syntax.md`；`src/formats/msg.rs`、`std.rs` 的 meta 与文件结构；`src/core_mapfiles/{msg,end,std}.rs` 的版本签名 |
| Studio 固定工具与编码扩展 | Studio 源码仓库的 `deps/README.md`、`docs/superpowers/specs/2026-11-09-truth-msg-encoding-design.md` | UTF-8 源码、Shift-JIS／GBK／UTF-8 二进制选择；实际 CLI 用法 |
| zero318/TouhouMaps | `0673f4221f5e975842162e07cd7b765fc7570ebd`，见 Studio 源码仓库的 `src-tauri/assets/maps/truth/defaults/PROVENANCE.json` | 合并 truth 签名后的默认名称、枚举和覆盖定义 |
| Studio 默认 map 注册表 | Studio 源码仓库的 `crates/script-check/src/bundled_maps.rs`、`src-tauri/assets/maps/truth/defaults/` | 本手册的 19 MSG + 12 END + 22 STD 支持集合和可调用短名 |

编译器源码在构建缓存 `deps/.cache/truth`，可按固定提交恢复；它不要求随用户项目提供。语法复核还使用了 `tests/integration/msg_features.rs`、`std_features.rs`、`general.rs` 及 `snapshots/integration__integration__bits_2_bits__{msg06_multiple_scripts,std06_general,std12_general}.snap`。正文的八份最小示例由随包 `trumsg`／`trustd` 加版本默认 map 实际编译。

有效签名由内建定义、版本默认 map、项目 map、源码 pragma 顺序叠加。编译器内建签名和最终 map 可能不同，TH13 op31 的随包默认 map 带一个整数，TH18／185 则为零参。旧稿曾把 TH18 的零参写法误记为 TH13，现已纠正；当前写法见人工指令参考。

## THBWiki：固定修订与适用章节

作者为 THBWiki 页面贡献者；任务提供者确认其中 MSG 新内容为本人编写。采用[《脚本对照表/其他》修订 975551](https://thwiki.cc/index.php?title=脚本对照表/其他&oldid=975551)。本次直接请求 MediaWiki `api.php?action=query&titles=脚本对照表/其他&prop=revisions&rvprop=ids|timestamp|content&rvslots=main&format=json`，得到 `revid=975551`、`2026-09-15T16:12:33Z`。搜索缓存中的 `896324` 不代表本次页面。

| 章节 | 本手册使用范围 |
| --- | --- |
| MSG 文件／二进制格式 | 入口表、时间和编码背景；编程写法改用 truth meta |
| MSG／游戏内运行逻辑 | 现代对话与 ECL 配合、分页、立绘 ANM 协议 |
| MSG 指令表、结局／staff 指令表 | 行为解释与各作差异；实际参数按当前有效 map |
| STD 文件／结构、std script 指令 | 现代背景模型、相机、雾、ECL 中断；旧偏移写法改用 truth 标签 |

MSG 章节的现代概括不能反推 TH06–09。STD 章节主要描述现代布局，不用它填补早期文件头差异。TH19／20 部分提供社区行为描述，未获得 TH16／18 同等的逐 case 一手材料。

## renkolab：只读的一手记录

本次读取 `/data/sunyunbo/www/renkolab`，HEAD 为 `c1a0f1672143ee565f662dcacd2031c90a8579e3`，分支 `feat/stg-agent-proto`。工作树存在其他未提交文件，包含 `.gitignore`、其他引擎笔记、游戏索引和 mod；本任务未修改、切换或清理该仓库。以下 MSG 文件在读取时没有本地修改。手册已摘述写代码所需结论，阅读者无需访问这个本机路径。

记录作者为 renkolab 项目维护者及其研究代理。它们引用的 thpages、thmsg 和 th-re-data 只计作对应的社区资料；本项目的一手行为部分来自记录中的游戏反编译与资源比对。

| 记录位置（相对 renkolab 根） | 版本、章节与手册结论 |
| --- | --- |
| `engine/msg/OVERVIEW.md` | 版本证据总览；TH19／20 无同等一手覆盖 |
| `engine/msg/WIKI-DRAFT.md` | MSG Wiki 母稿；用于章节索引与内容迁移，不作为独立第二份验证 |
| `engine/msg/th16/01-architecture-overview.md` §0–3 | TH16 v1.00a；关卡 HUD 和结局解释器分开、对话加载与 ANM 渲染；早期粗表以较新的 02／03／04 为准 |
| `engine/msg/th16/02-msg-vm-opcodes.md` §0–3 | TH16 v1.00a；op0–35、立绘控制、op27 尚有旧疑问；主派发 `0x42A1D0` |
| `engine/msg/th16/03-dialogue-lifecycle.md` §2 | TH16 v1.00a；`eclResume`、ECL 518／519 的信号时序；读点 `0x4216e0`，置位点为 MSG op12 |
| `engine/msg/th16/04-ending-staff-msg-instruction-set.md` §1–2 | TH16 v1.00a；END op0／3–17、ANM 源与槽、staff 选择；派发 `0x4199F0` |
| `engine/msg/th18/01-stage-msg.md` §2–7 | TH18 v1.00a；等待时钟、两行分页、表情参数顺序、ANM 中断、op27 参数；派发 `0x43e550` |
| `engine/msg/th18/02-ending-msg.md` §2–3 | TH18 v1.00a；五行分页、op5／6 负数分支、staff 末张图片差异、黑幕方向；派发 `0x42be70`，淡入／淡出 `0x475480`／`0x475820` |

TH16 01 开头误把 TH16 中文名写成《鬼形兽》，正文按版本号及可执行文件识别，不继承该笔误。renkolab OVERVIEW 里“Wiki 没有结局表”的旧描述也已过期，以固定 Wiki 新修订为准。

## 社区行为资料

下列链接固定提交，说明重新组织成本文自己的例子，未整页复制原教程。

| 作者／项目 | 固定入口 | 范围 |
| --- | --- | --- |
| ExpHP / thpages | 提交 `effc37824ff4373b8251bedcc516b08bd9d452d7` 的 [msg.ts](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/msg.ts) | 早期 MSG 和现代版本映射至 TH185、文本与说话者行为 |
| ExpHP / thpages | 同提交 [end.ts](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/end.ts) | END 映射明确列 TH10–18；注释还保留 TH20 的注音及 staff 变化线索 |
| ExpHP / thpages | 同提交 [std.ts](https://github.com/ExpHP/thpages/blob/effc37824ff4373b8251bedcc516b08bd9d452d7/js/tables/reference/std.ts) | TH06、TH07–09、现代背景指令世代，关键帧、相机与跳转 |
| Priw8 | 提交 `1c0276927ec203f7adfafca01ec157dba52c35a7` 的 [std-ins.md](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-ins.md)、[std-ins-table.md](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-ins-table.md)、[std-tutorial.md](https://github.com/Priw8/priw8.github.io/blob/1c0276927ec203f7adfafca01ec157dba52c35a7/content/modding/std-tutorial.md) | 面向 TH13 起的建模、背景循环和 ANM 配合；旧 thstd 元数据与手算偏移均重写为 truth 字段和符号标签 |

Priw8 的 STD 指令页明确引用 THBWiki 与 Nutzer；renkolab 的 Wiki 母稿与已发布 Wiki 也属于同一资料链。它们没有被计为多个互相独立的游戏实证。编译器文档负责语法／编码，map 负责可接受签名／枚举，游戏记录负责行为；编译通过本身不确认游戏效果。

## 当前差异与历史疑问

| 项目 | 当前结论与写法 |
| --- | --- |
| TH16 MSG op27 | TH16 旧笔记未确认读取 float；固定 truth 和版本 map 都接受 `f`，正文给出 `musicFade(seconds)`。TH18 记录明确读 float（`01-stage-msg.md` §4）。未将 TH18 的一手结果写成 TH16 已经重新验证 |
| TH18 END op6 负数 | thpages `wait-clear` 写“负数不转 999”，renkolab TH18 §2 记录 op5／6 都将负数转 999。当前编译器签名仅定义带符号整数，无法裁决运行行为。教程采用正数帧数；TH18 行为按其一手记录保留 |
| TH18 MSG op23／24 | 旧笔记称 truth 的 shake 带整数；本次固定 `src/core_mapfiles/msg.rs` 和默认 map 均无参，与 TH18 不读参数的记录一致。旧签名冲突已过期，不作为当前工具问题 |
| MSG 表情参数顺序 | thpages 名称数组有 `who, face` 的旧写法；TH18 一手明确 `arg2` 选槽、`arg1 + 17` 为中断，当前 map 也给首参 ANM 脚本类型、次参 PortraitIndex。教程采用 `playerFace(face, who)`／`bossFace(face, who)` |
| END 画面效果与 staff | thpages 的 op13／14 描述未完整，且“TH18 四个 staff 相同”与资源比对不符。教程采用 TH18 记录：13 淡入、14 淡出，四文件末张 CG 脚本不同 |
| TH19／20 版本支持 | 编译器和 53 份 map 已覆盖；该事实不补全 TH19 对战立绘或 TH20 新增行为资料。保留两类覆盖的区别 |

这些记录不触发编译器、map 或游戏修复。本轮没有新增逆向或改动签名。

## 尚未整理的行为

下面列出继续写常用脚本时通常可以原样保留的部分：

- MSG：TH08／09 部分多立绘组合、路线和特殊系统指令；现代 op26 的文本样式位；TH185 op40／41；TH19 新增相对侧立绘及 op44–56 的完整参数行为。TH143／165 op33 与 TH16 op33 用途不同。
- END：TH20 op9 两种颜色的分别用途；TH16／18 以外逐版本的行数、负数等待和音乐室细节。正文明确标版本的行为不补成全系列结论。
- STD：Bezier 三组向量的完整运行语义、各作摇动／扭曲模式、TH20 op22；旧资料对对象剔除层和个别尺寸细节仍不完整。TH06NC 研究不纳入本支持集合。

后续补全时，优先增加能改变编程写法的参数解释与短例子。完整指令签名从有效 map 导出，避免手工维护第二张易漂移的全量签名表。
