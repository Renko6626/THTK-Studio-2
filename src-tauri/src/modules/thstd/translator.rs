use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::map_parser::{StdInstructionSpec, StdSemanticData};

fn name_by_opcode(data: &StdSemanticData) -> HashMap<u32, &StdInstructionSpec> {
    data.instructions.iter().map(|i| (i.opcode, i)).collect()
}

fn opcode_by_name<'a>(data: &'a StdSemanticData) -> HashMap<&'a str, u32> {
    data.instructions
        .iter()
        .map(|i| (i.name.as_str(), i.opcode))
        .collect()
}

/// 交换 args 字符串里的前两个逗号分隔实参。0/1 参时不变。
/// 用于 thstd opcode 1 (jmp):thstd 二进制约定 ins_1(offset, time),
/// 与 ref/ECL 生态的 jmp(time, offset) 约定相反,翻译两侧都需要交换。
fn swap_first_two_args(args: &str) -> String {
    let parts: Vec<&str> = args.splitn(3, ',').collect();
    match parts.as_slice() {
        [a, b] => format!("{},{}", b.trim_start(), a),
        [a, b, rest] => format!("{},{},{}", b.trim_start(), a, rest),
        _ => args.to_string(),
    }
}

/// thstd 原始 dstd(`ins_N` 形式)→ 可读 dstd(`jmp`、`pos` 等)。
/// 行级文本变换:
///   - 形如 `[time_label:]ins_<opcode>(<args>)` 的行:opcode 查 semantics,
///     找到则替换为 name;找不到保留原 `ins_N`。
///   - opcode == 1 (jmp) 特例:在 name 替换后交换前两个实参。
///   - with_comments=true 时,翻译成功的行末追加 ` // <description>`(若 description 存在)。
///   - 其他行(空行、注释、段标签如 SCRIPT:、文本字符串等)原样透传。
pub fn dstd_to_readable(raw: &str, semantics: &StdSemanticData, with_comments: bool) -> String {
    let map = name_by_opcode(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^(\s*(?:[+\-]?\d+|@\w+):)?(\s*)ins_(\d+)\((.*)\)\s*$").unwrap()
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
///   - 行末的 ` // ...` 注释 strip。
///   - 其他行原样透传。
pub fn readable_to_dstd(readable: &str, semantics: &StdSemanticData) -> String {
    let map = opcode_by_name(semantics);

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^(\s*(?:[+\-]?\d+|@\w+):)?(\s*)([A-Za-z_]\w*)\((.*?)\)(\s*//.*)?\s*$")
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
            } else {
                // Unknown name (including bare `ins_N` which is not in the map):
                // preserve the body but strip any trailing comment.
                let stripped_body = if let Some(cm) = caps.get(5) {
                    let cm_str = cm.as_str();
                    let cm_start = body.len() - cm_str.len();
                    &body[..cm_start]
                } else {
                    body
                };
                out.push_str(stripped_body);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::thstd::map_parser::parse_std_semantics;

    fn data() -> StdSemanticData {
        parse_std_semantics("17").expect("seed")
    }

    #[test]
    fn translates_known_opcodes() {
        // opcode 2 is pos(float x, float y, float z) — non-jmp, no swap
        let raw = "0:ins_2(1.0, 2.0, 3.0)\n";
        let out = dstd_to_readable(raw, &data(), false);
        assert_eq!(out, "0:pos(1.0, 2.0, 3.0)\n");
    }

    #[test]
    fn unknown_opcode_preserved() {
        let raw = "0:ins_999(42)\n";
        let out = dstd_to_readable(raw, &data(), true);
        assert_eq!(out, "0:ins_999(42)\n");
    }

    #[test]
    fn passthrough_for_non_instruction_lines() {
        // STD section markers and arbitrary text lines must survive untouched.
        let raw = "// header comment\n\nSCRIPT:\nPOSITION: 0.0 0.0 0.0\n";
        let out = dstd_to_readable(raw, &data(), true);
        assert_eq!(out, raw);
    }

    #[test]
    fn preserves_time_label_variants() {
        let raw = "0:ins_2(1.0, 2.0, 3.0)\n60:ins_2(1.0, 2.0, 3.0)\n+30:ins_2(1.0, 2.0, 3.0)\n";
        let out = dstd_to_readable(raw, &data(), false);
        assert_eq!(
            out,
            "0:pos(1.0, 2.0, 3.0)\n60:pos(1.0, 2.0, 3.0)\n+30:pos(1.0, 2.0, 3.0)\n"
        );
    }

    #[test]
    fn jmp_decompile_swaps_args() {
        // raw thstd: ins_1(offset, time) ; readable: jmp(time, offset)
        // 实际输出 `jmp(60,100)`(swap 把 b 前导空格 trim 掉,分隔符是裸 `,`)。
        let raw = "0:ins_1(100, 60)\n";
        let out = dstd_to_readable(raw, &data(), true);
        assert!(out.starts_with("0:jmp(60,100)"), "got: {out}");
        // comment retained when with_comments=true
        assert!(out.contains("// 跳转"), "got: {out}");
    }

    #[test]
    fn jmp_compile_swaps_back() {
        let readable = "0:jmp(60,100)\n";
        let back = readable_to_dstd(readable, &data());
        assert_eq!(back, "0:ins_1(100,60)\n");
    }

    #[test]
    fn jmp_roundtrip_recovers_original() {
        // 无空格参数能精确往返;`100, 60` 这种含空格输入往返后会丢失空格,
        // 这是 swap 函数 spec 的固定行为(b.trim_start + 裸逗号),不在 jmp 路径上保证空格保真。
        let raw = "0:ins_1(100,60)\n";
        let readable = dstd_to_readable(raw, &data(), false);
        let back = readable_to_dstd(&readable, &data());
        assert_eq!(back, raw);
    }

    #[test]
    fn swap_first_two_args_zero_or_one_arg_no_crash() {
        assert_eq!(swap_first_two_args(""), "");
        assert_eq!(swap_first_two_args("42"), "42");
    }
}
