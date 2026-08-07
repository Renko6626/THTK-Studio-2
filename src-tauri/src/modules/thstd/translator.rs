use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::common::map_file::{MapFileData, MapInstruction};

fn name_by_opcode(data: &MapFileData) -> HashMap<u32, &MapInstruction> {
    data.instructions.iter().map(|i| (i.opcode, i)).collect()
}

fn opcode_by_name<'a>(data: &'a MapFileData) -> HashMap<&'a str, u32> {
    data.instructions
        .iter()
        .map(|i| (i.name.as_str(), i.opcode))
        .collect()
}

/// 交换 args 字符串里的前两个逗号分隔实参,输出统一为 `arg1, arg2` 风格。
/// 0/1 参时不变。3+ 参时保留第三个及之后的原格式不动。
fn swap_first_two_args(args: &str) -> String {
    let parts: Vec<&str> = args.splitn(3, ',').collect();
    match parts.as_slice() {
        [a, b] => format!("{}, {}", b.trim(), a.trim()),
        [a, b, rest] => format!("{}, {},{}", b.trim(), a.trim(), rest),
        _ => args.to_string(),
    }
}

/// 写在可读 .dstd 开头的方言声明。
///
/// `thstd` **没有** `-m`，指令名映射是本 IDE 在它外面做的：解包时把 `ins_N`
/// 换成名字写盘，打包时再换回来喂给 `thstd`。因此磁盘上的 .dstd **不是合法的
/// `thstd` 输入**——直接跑 `thstd -c` 会在每条带名字的指令上失败。
///
/// 之所以仍在磁盘上存名字：`git diff` 里 `textboxShow(0)` 与 `ins_3(0)` 的
/// 可读性差距是决定性的。代价就是必须把这件事写在文件里，而不是指望用户记得。
pub const DIALECT_HEADER: &str = "# THTK-Studio dstd 方言：指令名由 IDE 映射，thstd 只认 ins_N；需要原始格式请用「导出原始 .dstd」。\n";

/// 方言头的识别前缀。反向翻译时必须**整行剥掉**——thstd 不认识它，
/// 原样透传会让它把这行当成指令而报错。
///
/// 头**必须是单行**：写成多行时只有第一行带得上这个前缀，后续行会漏网
/// 一路送进 thstd。这个 bug 被 compiler 层的剥离测试抓到过一次。
pub const DIALECT_MARKER: &str = "# THTK-Studio ";

/// thstd 原始 dstd(`ins_N` 形式)→ 可读 dstd(`jmp`、`pos` 等)。
///
/// ## 输入形状以 thstd 源码为准
///
/// `thtk/thstd/thstd.c` 的 dump 函数逐字如下：
///
/// ```c
/// fprintf(stream, "%i:\n", instr->time);        // 720:      顶格、独占一行
/// fprintf(stream, "    ins_%i(", instr->type);  //     ins_1( 4 空格缩进
/// fprintf(stream, ");\n");                      //     );     ★行尾分号
/// ```
///
/// **行尾那个分号曾经让整个翻译器变成恒等函数**：正则写的是 `\)\s*$`，匹配不上
/// `);`，于是每一行都走 passthrough，磁盘上的 .dstd 从来没有过指令名。原有测试
/// 全绿是因为它们的输入写成 `"0:ins_2(1.0, 2.0, 3.0)\n"`——标签与指令同行、无缩进、
/// 无分号，这是 thstd 从不产生的形状。所以分号必须**认得也留得住**。
///
/// 行级变换:
///   - 形如 `[time_label:]ins_<opcode>(<args>)[;]` 的行:opcode 查 semantics,
///     找到则替换为 name;找不到保留原 `ins_N`。分号原样带回,以便精确往返。
///   - opcode == 1 (jmp) 特例:在 name 替换后交换前两个实参。
///   - with_comments=true 时,翻译成功的行末追加 ` // <description>`(若 description 存在)。
///   - 其他行(空行、注释、`SCRIPT:`/`ENTRY:`/`FACE:` 等文件头、文本行)原样透传。
pub fn dstd_to_readable(raw: &str, semantics: &MapFileData, with_comments: bool) -> String {
    let map = name_by_opcode(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^(\s*(?:[+\-]?\d+|@\w+):)?(\s*)ins_(\d+)\((.*)\)\s*(;?)\s*$").unwrap()
    });

    let mut out = String::with_capacity(raw.len() + 256);
    for line in raw.split_inclusive('\n') {
        let trailing_newline = line.ends_with('\n');
        let body = if trailing_newline {
            &line[..line.len() - 1]
        } else {
            line
        };

        if let Some(caps) = re.captures(body) {
            let label = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let indent = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let opcode: u32 = caps
                .get(3)
                .unwrap()
                .as_str()
                .parse()
                .unwrap_or(u32::MAX);
            let args = caps.get(4).unwrap().as_str();
            let semi = caps.get(5).map(|m| m.as_str()).unwrap_or("");

            if let Some(spec) = map.get(&opcode) {
                let args_out = if opcode == 1 {
                    swap_first_two_args(args)
                } else {
                    args.to_string()
                };
                out.push_str(label);
                out.push_str(indent);
                out.push_str(&spec.name);
                out.push('(');
                out.push_str(&args_out);
                out.push(')');
                out.push_str(semi);
                if with_comments {
                    if let Some(desc) = spec.description.as_deref().filter(|s| !s.is_empty()) {
                        out.push_str(" // ");
                        out.push_str(desc);
                    }
                }
            } else {
                // unknown opcode → preserve raw body
                out.push_str(body);
            }
        } else {
            // not an ins line → passthrough
            out.push_str(body);
        }

        if trailing_newline {
            out.push('\n');
        }
    }
    out
}

/// 可读 dstd → thstd 原始 dstd。
/// 反向变换:
///   - 形如 `[time_label:]<name>(<args>)` 且 name 在 semantics 里 → 替换为 `ins_<opcode>`;
///     找不到名字保留原文(让 thstd 自己报错)。
///   - opcode == 1 (jmp) 特例:在 ins_N 替换后再次交换前两个实参,把 ref 约定的
///     (time, offset) 还原为 thstd 二进制约定的 (offset, time)。
///
/// 为什么归一成 (time, offset)：依据是 **ECL 的既有约定**。truth 的核心签名表里
/// ECL 是 `("to", Jmp)`（time 在前），而 ANM 与 STD 都是 `("ot", Jmp)`（offset 在前）
/// ——三者里只有 ECL 反过来。让 STD 向 ECL 看齐，是为了让在 .decl 和 .dstd 之间
/// 来回看的人不用切换心智模型；ECL 是模组作者的主力语言。
///
/// 代价是我们的 .dstd 与 truth 的渲染顺序相反（truth 保持原始的 ot）。这是有意的
/// 取舍。接 ANM 时会撞上同一个选择——ANM 原始也是 ot。
///   - 行末的 ` // ...` 注释 strip。
///   - 其他行原样透传。
pub fn readable_to_dstd(readable: &str, semantics: &MapFileData) -> String {
    let map = opcode_by_name(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        // 贪婪 `(.*)` 而非惰性：参数里出现 `)` 时要吃到**最后**一个右括号,
        // 否则 `foo(a)b)` 会被截成 `a` 而后半段落到注释组上、整条正则失配。
        Regex::new(r"^(\s*(?:[+\-]?\d+|@\w+):)?(\s*)([A-Za-z_]\w*)\((.*)\)\s*(;?)\s*(//.*)?\s*$")
            .unwrap()
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
            let label = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let indent = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let name = caps.get(3).unwrap().as_str();
            let args = caps.get(4).unwrap().as_str();
            let semi = caps.get(5).map(|m| m.as_str()).unwrap_or("");

            if let Some(&opcode) = map.get(name) {
                let args_out = if opcode == 1 {
                    swap_first_two_args(args)
                } else {
                    args.to_string()
                };
                out.push_str(label);
                out.push_str(indent);
                out.push_str(&format!("ins_{opcode}("));
                out.push_str(&args_out);
                out.push(')');
                out.push_str(semi);
            } else {
                // 名字不在表里(含裸 `ins_N`):照原样重建,只丢掉行尾注释——
                // 让 thstd 自己去报错,而不是我们猜。
                out.push_str(label);
                out.push_str(indent);
                out.push_str(name);
                out.push('(');
                out.push_str(args);
                out.push(')');
                out.push_str(semi);
            }
        } else {
            // Not a name-call line → passthrough
            out.push_str(body);
        }

        if trailing_newline {
            out.push('\n');
        }
    }
    out
}

/// 测试输入一律用 **thstd 真实吐出来的形状**，依据 `thstd.c` 的 dump 函数：
/// 时间标签顶格独占一行、指令 4 空格缩进、行尾带分号。
///
/// 本模块原先的输入写成 `"0:ins_2(1.0, 2.0, 3.0)\n"`——标签与指令同行、无缩进、
/// 无分号，thstd 从不产生这种形状。11 个测试全绿，而真实管线里翻译器是恒等函数。
/// 这就是为什么下面每个用例都从 `SCRIPT:` 开始抄真实 dump，而不是造一个方便断言的串。
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::thstd::map_parser::parse_std_semantics;

    fn data() -> MapFileData {
        parse_std_semantics("14").expect("seed")
    }

    /// 取自 `reference/st01.txt` 对应的真实指令序列
    const RAW: &str = "SCRIPT:\n    ins_6(0.06f, 1.0f, 0.0f);\n720:\n    ins_1(728, 4220);\n";

    #[test]
    fn translates_real_thstd_dump() {
        let out = dstd_to_readable(RAW, &data(), false);
        assert_eq!(
            out,
            "SCRIPT:\n    up(0.06f, 1.0f, 0.0f);\n720:\n    jmp(4220, 728);\n"
        );
    }

    /// 分号是 thstd 的输出的一部分，丢了就往返不回去
    #[test]
    fn roundtrip_recovers_thstd_output_byte_for_byte() {
        let readable = dstd_to_readable(RAW, &data(), false);
        let back = readable_to_dstd(&readable, &data());
        assert_eq!(back, RAW, "往返未还原 thstd 原始文本");
    }

    /// 文件头（`ANM:`/`ENTRY:`/`FACE:` 等）在 `SCRIPT:` 之前，必须原样透传。
    /// 它们不是指令，也不参与跳转偏移的计算。
    #[test]
    fn passthrough_for_header_and_non_instruction_lines() {
        let raw = "ANM: st01wl.anm\nStd_unknown: 0\n\nENTRY:\n    Unknown: 1\n    Position: 0 0 300\n\n    FACE: 256 0 280 0\n\nSCRIPT:\n";
        assert_eq!(dstd_to_readable(raw, &data(), true), raw);
    }

    #[test]
    fn unknown_opcode_preserved_verbatim() {
        let raw = "    ins_999(42);\n";
        assert_eq!(dstd_to_readable(raw, &data(), true), raw);
        assert_eq!(readable_to_dstd(raw, &data()), raw);
    }

    #[test]
    fn preserves_time_label_variants() {
        let raw = "0:\n    ins_2(1.0, 2.0, 3.0);\n60:\n    ins_2(1.0, 2.0, 3.0);\n+30:\n    ins_2(1.0, 2.0, 3.0);\n";
        let out = dstd_to_readable(raw, &data(), false);
        assert_eq!(
            out,
            "0:\n    pos(1.0, 2.0, 3.0);\n60:\n    pos(1.0, 2.0, 3.0);\n+30:\n    pos(1.0, 2.0, 3.0);\n"
        );
    }

    /// 手写的 .dstd 可能不带分号；认得但不硬加，原样往返
    #[test]
    fn tolerates_hand_written_lines_without_semicolon() {
        let raw = "    ins_2(1.0, 2.0, 3.0)\n";
        let out = dstd_to_readable(raw, &data(), false);
        assert_eq!(out, "    pos(1.0, 2.0, 3.0)\n");
        assert_eq!(readable_to_dstd(&out, &data()), raw);
    }

    #[test]
    fn comments_are_appended_then_stripped_on_the_way_back() {
        let raw = "    ins_1(728, 4220);\n";
        let out = dstd_to_readable(raw, &data(), true);
        assert!(out.contains("// 跳转"), "got: {out}");
        assert!(out.contains("jmp(4220, 728);"), "got: {out}");
        assert_eq!(readable_to_dstd(&out, &data()), raw);
    }

    /// jmp 的原始序是 `(offset, time)`，我们归一成 ECL 的 `(time, offset)`。
    /// 依据见 `readable_to_dstd` 的文档注释。
    #[test]
    fn jmp_swaps_both_directions() {
        let out = dstd_to_readable("    ins_1(100, 60);\n", &data(), false);
        assert_eq!(out, "    jmp(60, 100);\n");
        assert_eq!(readable_to_dstd(&out, &data()), "    ins_1(100, 60);\n");
    }

    #[test]
    fn swap_first_two_args_zero_or_one_arg_no_crash() {
        assert_eq!(swap_first_two_args(""), "");
        assert_eq!(swap_first_two_args("42"), "42");
    }

    #[test]
    fn jmp_normalizes_space_after_comma() {
        let out = dstd_to_readable("    ins_1(100,60);\n", &data(), false);
        assert_eq!(out, "    jmp(60, 100);\n");
    }
}
