use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::common::map_file::{MapFileData, MapInstruction};

use super::MsgScriptKind;

fn name_by_opcode(data: &MapFileData) -> HashMap<u32, &MapInstruction> {
    data.instructions.iter().map(|i| (i.opcode, i)).collect()
}

fn opcode_by_name<'a>(data: &'a MapFileData) -> HashMap<&'a str, u32> {
    data.instructions
        .iter()
        .map(|i| (i.name.as_str(), i.opcode))
        .collect()
}

/// 写在可读 .dmsg 开头的方言声明。
///
/// `thmsg` **没有** `-m`，指令名映射是本 IDE 在它外面做的：解包时把操作号换成
/// 名字写盘，打包时再换回来喂给 `thmsg`。因此磁盘上的 .dmsg **不是合法的
/// `thmsg` 输入**——直接跑 `thmsg -c` 会在每条带名字的指令上失败。
///
/// 之所以仍在磁盘上存名字：`git diff` 里 `textboxShow()` 与 `3` 的可读性差距
/// 是决定性的。代价就是必须把这件事写在文件里，而不是指望用户记得。
///
/// 头的第二项是**脚本类型**。关卡对话与结局/staff 的 opcode 编号重叠但语义
/// 完全不同，光看 `3 textAdd()` 无法判断它该走 stage 表还是 `-e` 表。
pub const DIALECT_HEADER_STAGE: &str = "# THTK-Studio dmsg 方言（关卡对话）：指令名由 IDE 映射，thmsg 只认 `3;参数`；需要原始格式请用「导出原始 .dmsg」。\n";
pub const DIALECT_HEADER_ENDING: &str = "# THTK-Studio dmsg 方言（结局/staff）：指令名由 IDE 映射，thmsg -e 只认 `3;参数`；需要原始格式请用「导出原始 .dmsg」。\n";

pub fn dialect_header(kind: MsgScriptKind) -> &'static str {
    match kind {
        MsgScriptKind::Stage => DIALECT_HEADER_STAGE,
        MsgScriptKind::Ending => DIALECT_HEADER_ENDING,
    }
}

/// 从方言头读脚本类型。旧文件没有类型标记时返回 `None`，由调用方按文件名兜底。
pub fn dialect_kind(content: &str) -> Option<MsgScriptKind> {
    content.lines().take(8).find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(DIALECT_MARKER) {
            return None;
        }
        if trimmed.contains("结局") || trimmed.contains("staff") || trimmed.contains("ending") {
            Some(MsgScriptKind::Ending)
        } else if trimmed.contains("关卡") || trimmed.contains("stage") {
            Some(MsgScriptKind::Stage)
        } else {
            None
        }
    })
}

/// 方言头的识别前缀。反向翻译时必须**整行剥掉**——thmsg 不认识它，
/// 原样透传会让它把这行当成指令而报错。
///
/// 头**必须是单行**：写成多行时只有第一行带得上这个前缀，后续行会漏网
/// 一路送进 thmsg。这个 bug 被 compiler 层的剥离测试抓到过一次。
pub const DIALECT_MARKER: &str = "# THTK-Studio ";

/// thmsg 原始 dmsg → 可读 dmsg(`textboxShow()` 等)。
///
/// ## 输入形状以 thmsg 源码为准
///
/// `thtk/thmsg/thmsg06.c` 的 dump 函数（th06–th20 都走 `th06_msg` 模块）：
///
/// ```c
/// fprintf(out, "@%u\n", time);        // @120     ★是 @ 前缀，不是 `120:`
/// fprintf(out, "\t%d", msg->type);    // \t17     ★TAB + 裸操作号，没有 ins_ 前缀
/// fprintf(out, ";%s", disp);          // ;文本    ★分号分隔，没有括号
/// ```
///
/// 也就是说 **`ins_N(...)` 这种写法 thmsg 一次也没产生过**。本函数原先按
/// `ins_N(...)` 匹配，于是对真实输出是恒等函数，磁盘上的 .dmsg 从来没有过指令名。
///
/// ## 为什么括号里仍用分号分隔
///
/// `util/value.c` 的 `value_to_text` 对 MSG 对白（类型 `m`）是**裸拷贝**：
///
/// ```c
/// case 'm':
///     memcpy(temp, value->val.m.data, value->val.m.length);
///     temp[value->val.m.length] = '\0';   // 不加引号、不转义
/// ```
///
/// 对白里可以出现任何字符，包括半角逗号。若改成逗号分隔参数再切回去，一句带
/// 逗号的台词会被劈成两个参数——**静默损坏正文**，而正文是 MSG 文件里唯一重要
/// 的东西。沿用 thmsg 自己的分号，切分风险与 thmsg 本身完全相同，我们不引入新
/// 的损坏面。这也是 THTK-Studio 一代的做法。
///
/// 行级变换:
///   - `[缩进]<opcode>[;<args>]` → `[缩进]<name>(<args>)`;opcode 不认识则写成 `ins_N(...)`。
///   - with_comments=true 时行末追加 ` // <description>`。
///   - 其他行(`@120` 时间标签、`entry 0`、`header(...)`、空行、注释)原样透传。
pub fn dmsg_to_readable(raw: &str, semantics: &MapFileData, with_comments: bool) -> String {
    let map = name_by_opcode(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    // `\t17;参数` —— 缩进、裸操作号、可选的 `;参数…`（整段原样搬走，不做切分）
    let re = RE.get_or_init(|| Regex::new(r"^([ \t]*)(\d+)(;.*)?$").unwrap());

    let mut out = String::with_capacity(raw.len() + 256);
    for line in raw.split_inclusive('\n') {
        let trailing_newline = line.ends_with('\n');
        let body = if trailing_newline {
            &line[..line.len() - 1]
        } else {
            line
        };

        if let Some(caps) = re.captures(body) {
            let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let opcode: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(u32::MAX);
            // 去掉引导的 `;`，剩下的整段就是括号里的内容
            let args = caps.get(3).map(|m| &m.as_str()[1..]).unwrap_or("");

            let spec = map.get(&opcode);
            out.push_str(indent);
            match spec {
                Some(s) if !s.name.is_empty() => out.push_str(&s.name),
                // 有名表项也可能是「只有签名、没有名字」的未命名条目
                // （th20 的 37–56）；这种条目同样显示 ins_N。
                Some(_) | None => out.push_str(&format!("ins_{opcode}")),
            }
            out.push('(');
            out.push_str(args);
            out.push(')');
            if with_comments {
                if let Some(desc) = spec
                    .and_then(|s| s.description.as_deref())
                    .filter(|s| !s.is_empty())
                {
                    out.push_str(" // ");
                    out.push_str(desc);
                }
            }
        } else {
            // 时间标签 `@120`、`entry 0`、`header(...)`、空行、注释 → 原样透传
            out.push_str(body);
        }

        if trailing_newline {
            out.push('\n');
        }
    }
    out
}

/// 可读 dmsg → thmsg 原始 dmsg（`\t<opcode>;<args>`）。
///
/// 反向变换:
///   - `[缩进]<name>(<args>)` 且 name 在 semantics 里 → `[缩进]<opcode>[;<args>]`;
///     `ins_N(...)` 同样还原为 `N`。无参数时**不写分号**——thmsg 的空参数行就是裸操作号。
///   - 行末的 ` // ...` 注释 strip。
///   - 名字不认识（如 thmsg 自己输出的 `header(0, 0)`）→ 保留原文，让 thmsg 自己报错。
///   - 其他行（`@120`、`entry 0`、空行）原样透传。
pub fn readable_to_dmsg(readable: &str, semantics: &MapFileData) -> String {
    let map = opcode_by_name(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        // 贪婪 `(.*)` 而非惰性：对白正文里出现 `)` 时要吃到**最后**一个右括号。
        // 惰性会在第一个 `)` 处截断，把正文后半段丢给注释组，整条正则失配。
        Regex::new(r"^([ \t]*)([A-Za-z_]\w*)\((.*)\)\s*(//.*)?\s*$").unwrap()
    });

    let mut out = String::with_capacity(readable.len());
    for line in readable.split_inclusive('\n') {
        let trailing_newline = line.ends_with('\n');
        let body = if trailing_newline {
            &line[..line.len() - 1]
        } else {
            line
        };

        if let Some(caps) = re.captures(body) {
            let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = caps.get(2).unwrap().as_str();
            let args = caps.get(3).unwrap().as_str();

            // 名字表优先；用户手里没翻译的 `ins_N` 也要能还原
            let opcode = map.get(name).copied().or_else(|| {
                name.strip_prefix("ins_")
                    .and_then(|n| n.parse::<u32>().ok())
            });

            match opcode {
                Some(opcode) => {
                    out.push_str(indent);
                    out.push_str(&opcode.to_string());
                    if !args.is_empty() {
                        out.push(';');
                        out.push_str(args);
                    }
                }
                None => {
                    // 不认识的名字：照原样重建，只丢掉行尾注释
                    out.push_str(indent);
                    out.push_str(name);
                    out.push('(');
                    out.push_str(args);
                    out.push(')');
                }
            }
        } else {
            // `@120`、`entry 0`、空行 → 原样透传。
            //
            // 但 `//` 注释必须**顶格**：thmsg 的 `th06_write` 用
            // `strncmp(buffer, "//", 2)` 判注释，前面有缩进就不认。用户习惯
            // 缩进写注释，这里在喂给 thmsg 的临时文本里去掉缩进。
            let trimmed = body.trim_start();
            if trimmed.starts_with("//") {
                out.push_str(trimmed);
            } else {
                out.push_str(body);
            }
        }

        if trailing_newline {
            out.push('\n');
        }
    }
    out
}

/// 修补 thtk 自身的 dump/compile 不一致。
///
/// 目前只有一条：TH16–TH18 的 `musicBoss`（opcode 19）在真实文件里是 size 0，
/// `thmsg -d` 输出裸 `19`；但 `thmsg -c` 按 `th16_msg_fmts` 要求一个 S 参数，
/// 于是「thmsg 自己 dump 出来的文件，thmsg 自己编译不回去」。
///
/// 引擎在 TH16–18 不读这个参数。这里在喂给 thmsg 的临时文本里补一个 `1`
/// （与 TH185 的真实曲目索引一致），让编译通过；代价是产物会多 4 字节，
/// 不再与输入逐字节相同。
///
/// 返回 `(修补后的文本, 修补的行数)`。
pub fn apply_thtk_quirks(raw: &str, version: u32, kind: MsgScriptKind) -> (String, usize) {
    if kind != MsgScriptKind::Stage || !matches!(version, 16 | 17 | 165 | 18) {
        return (raw.to_string(), 0);
    }

    let mut out = String::with_capacity(raw.len() + 8);
    let mut patched = 0;

    for line in raw.split_inclusive('\n') {
        let trailing_newline = line.ends_with('\n');
        let body = if trailing_newline {
            &line[..line.len() - 1]
        } else {
            line
        };

        let trimmed = body.trim();
        if trimmed == "19" || trimmed == "19;" {
            let indent_len = body.len() - body.trim_start().len();
            out.push_str(&body[..indent_len]);
            out.push_str("19;1");
            patched += 1;
        } else {
            out.push_str(body);
        }

        if trailing_newline {
            out.push('\n');
        }
    }

    (out, patched)
}

/// 测试输入一律用 **thmsg 真实吐出来的形状**（`@120` 时间标签 + `\t<opcode>;<args>`），
/// 依据 `thmsg06.c` 的 dump 函数。
///
/// 本模块原先的输入写成 `"0:ins_3()\n"`——`ins_N(...)` 是 thmsg 从不产生的写法，
/// 于是 11 个测试全绿，而真实管线里翻译器是恒等函数、.dmsg 从来没有过指令名。
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::msg::map_parser::parse_msg_semantics;
    use crate::modules::msg::MsgScriptKind;

    fn data() -> MapFileData {
        parse_msg_semantics("17", MsgScriptKind::Stage).expect("seed")
    }

    /// thmsg 原样 dump 的一段：文件头、entry、时间标签、无参指令、带参指令
    const RAW: &str = "header(0, 0)\nentry 0\n@0\n\t3\n\t1;0\n";

    #[test]
    fn translates_real_thmsg_dump() {
        let out = dmsg_to_readable(RAW, &data(), false);
        assert_eq!(
            out,
            "header(0, 0)\nentry 0\n@0\n\ttextboxShow()\n\tplayerShow(0)\n"
        );
    }

    #[test]
    fn roundtrip_recovers_thmsg_output_byte_for_byte() {
        let readable = dmsg_to_readable(RAW, &data(), true);
        assert_eq!(readable_to_dmsg(&readable, &data()), RAW);
    }

    /// th19 的新指令名带 `__` 前缀（zero318 的「未坐实」标记，我们原样保留）。
    /// 反向正则是 `[A-Za-z_]\w*`，前导双下划线必须能被吃回去——否则打包时
    /// 这些指令会当成「无法解析的行」，而它们恰恰是 th19 最常改的那批。
    #[test]
    fn double_underscore_names_survive_roundtrip() {
        let th19 = parse_msg_semantics("19", MsgScriptKind::Stage).expect("th19 seed");
        let raw = "\t44;1.0;2.0\n";

        let out = dmsg_to_readable(raw, &th19, false);
        assert_eq!(out, "\t__unknown_position_A(1.0;2.0)\n");
        assert_eq!(readable_to_dmsg(&out, &th19), raw);
    }

    /// th20 没有 42–56（见 `th20.msgm` 抬头的取舍说明），所以同一行在 th20 下
    /// 必须退回 `ins_44` 而不是借用 th19 的名字。
    #[test]
    fn th20_falls_back_to_ins_n_for_th19_only_opcodes() {
        let th20 = parse_msg_semantics("20", MsgScriptKind::Stage).expect("th20 seed");
        let raw = "\t44;1.0;2.0\n";

        let out = dmsg_to_readable(raw, &th20, false);
        assert_eq!(out, "\tins_44(1.0;2.0)\n");
        assert_eq!(readable_to_dmsg(&out, &th20), raw);
    }

    /// 无参指令在 thmsg 里是**裸操作号**，不带分号。多写一个 `;` 会变成一个空参数。
    #[test]
    fn thtk_music_boss_quirk_is_patched_only_for_th16_to_th18() {
        let raw = "@0\n\t19\n\t19;1\n";
        let (patched, count) = apply_thtk_quirks(raw, 18, MsgScriptKind::Stage);
        assert_eq!(patched, "@0\n\t19;1\n\t19;1\n");
        assert_eq!(count, 1);

        for version in [16, 17, 165] {
            let (_, count) = apply_thtk_quirks(raw, version, MsgScriptKind::Stage);
            assert_eq!(count, 1, "th{version}");
        }

        // TH185/19/20 的 op19 本来就是带参数的，不要动。
        for version in [185, 19, 20] {
            let (patched, count) = apply_thtk_quirks(raw, version, MsgScriptKind::Stage);
            assert_eq!(patched, raw, "th{version}");
            assert_eq!(count, 0);
        }

        // 结局模式没有这条指令。
        let (patched, count) = apply_thtk_quirks(raw, 18, MsgScriptKind::Ending);
        assert_eq!(patched, raw);
        assert_eq!(count, 0);
    }

    #[test]
    fn indented_slash_comments_are_flattened_for_thmsg() {
        assert_eq!(
            readable_to_dmsg("    // note\n\t3\n", &data()),
            "// note\n\t3\n"
        );
    }

    #[test]
    fn no_args_emits_bare_opcode_without_semicolon() {
        assert_eq!(readable_to_dmsg("\ttextboxShow()\n", &data()), "\t3\n");
    }

    /// 参数在括号里仍用分号分隔——沿用 thmsg 自己的分隔符，见 `dmsg_to_readable` 的说明。
    #[test]
    fn multiple_args_keep_semicolon_separator() {
        let raw = "\t14;0;1\n";
        let out = dmsg_to_readable(raw, &data(), false);
        assert_eq!(out, "\tbossFace(0;1)\n");
        assert_eq!(readable_to_dmsg(&out, &data()), raw);
    }

    /// 关键的正文安全性：对白裸输出、不转义，半角逗号必须原样活下来。
    /// 若改用逗号分隔参数，这句会被劈成两个参数——静默损坏正文。
    #[test]
    fn dialogue_text_with_commas_survives_roundtrip() {
        let raw = "\t17;こんにちは、世界, and hello\n";
        let out = dmsg_to_readable(raw, &data(), false);
        assert_eq!(out, "\ttextAdd(こんにちは、世界, and hello)\n");
        assert_eq!(readable_to_dmsg(&out, &data()), raw);
    }

    /// 对白里出现右括号时，反向必须吃到**最后**一个 `)`（贪婪匹配）
    #[test]
    fn dialogue_text_with_parenthesis_survives_roundtrip() {
        let raw = "\t17;笑 (ぐぬぬ) だ\n";
        let out = dmsg_to_readable(raw, &data(), false);
        assert_eq!(readable_to_dmsg(&out, &data()), raw);
    }

    #[test]
    fn unknown_opcode_becomes_ins_n_and_returns() {
        let raw = "\t250;42\n";
        let out = dmsg_to_readable(raw, &data(), true);
        assert_eq!(out, "\tins_250(42)\n");
        assert_eq!(readable_to_dmsg(&out, &data()), raw);
    }

    /// `header(...)` 是 thmsg 自己输出的一行，不是指令；两个方向都不能动它
    #[test]
    fn header_line_is_not_mistaken_for_an_instruction() {
        let raw = "header(0, 0)\n";
        assert_eq!(dmsg_to_readable(raw, &data(), true), raw);
        assert_eq!(readable_to_dmsg(raw, &data()), raw);
    }

    #[test]
    fn comments_are_appended_then_stripped_on_the_way_back() {
        let out = dmsg_to_readable("\t3\n", &data(), true);
        assert!(out.contains("// 显示对话框"), "got: {out}");
        assert_eq!(readable_to_dmsg(&out, &data()), "\t3\n");
    }

    #[test]
    fn passthrough_for_time_labels_and_blank_lines() {
        let raw = "@0\n\n@120\nentry 3\n";
        assert_eq!(dmsg_to_readable(raw, &data(), true), raw);
        assert_eq!(readable_to_dmsg(raw, &data()), raw);
    }

    #[test]
    fn dialect_header_records_the_script_kind() {
        assert_eq!(
            dialect_kind(dialect_header(MsgScriptKind::Stage)),
            Some(MsgScriptKind::Stage)
        );
        assert_eq!(
            dialect_kind(dialect_header(MsgScriptKind::Ending)),
            Some(MsgScriptKind::Ending)
        );
    }

    #[test]
    fn old_dialect_header_has_no_kind_and_uses_filename_fallback() {
        assert_eq!(
            dialect_kind(&DIALECT_HEADER_STAGE.replace("（关卡对话）", "")),
            None
        );
    }

    #[test]
    fn ending_names_translate_both_ways() {
        let end = parse_msg_semantics("18", MsgScriptKind::Ending).expect("ending seed");
        let raw = "\t7;0;e01.anm\n\t8;0;0;1\n";
        let readable = dmsg_to_readable(raw, &end, false);
        assert_eq!(
            readable,
            "\tanmSourceLoad(0;e01.anm)\n\tanmSetSlot(0;0;1)\n"
        );
        assert_eq!(readable_to_dmsg(&readable, &end), raw);
    }
}
