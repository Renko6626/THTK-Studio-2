use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::map_parser::{MsgInstructionSpec, MsgSemanticData};

fn name_by_opcode(data: &MsgSemanticData) -> HashMap<u32, &MsgInstructionSpec> {
    data.instructions.iter().map(|i| (i.opcode, i)).collect()
}

fn opcode_by_name<'a>(data: &'a MsgSemanticData) -> HashMap<&'a str, u32> {
    data.instructions
        .iter()
        .map(|i| (i.name.as_str(), i.opcode))
        .collect()
}

/// thmsg 原始 dmsg(`ins_N` 形式)→ 可读 dmsg(`textboxShow` 等)。
/// 行级文本变换:
///   - 形如 `[time_label:]ins_<opcode>(<args>)` 的行:opcode 查 semantics,
///     找到则替换为 name;找不到保留原 `ins_N`。
///   - with_comments=true 时,翻译成功的行末追加 ` // <description>`(若 description 存在)。
///   - 其他行(空行、注释、文本字符串、难度/时间标签、引号包裹等)原样透传。
pub fn dmsg_to_readable(raw: &str, semantics: &MsgSemanticData, with_comments: bool) -> String {
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
                out.push_str(label);
                out.push_str(indent);
                out.push_str(&spec.name);
                out.push('(');
                out.push_str(args);
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

/// 可读 dmsg → thmsg 原始 dmsg。
/// 反向变换:
///   - 形如 `[time_label:]<name>(<args>)` 且 name 在 semantics 里 → 替换为 `ins_<opcode>`;
///     找不到名字保留原文(让 thmsg 自己报错)。
///   - 行末的 ` // ...` 注释 strip。
///   - 其他行原样透传。
pub fn readable_to_dmsg(readable: &str, semantics: &MsgSemanticData) -> String {
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
                out.push_str(label);
                out.push_str(indent);
                out.push_str(&format!("ins_{opcode}("));
                out.push_str(args);
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
    use crate::modules::msg::map_parser::parse_msg_semantics;

    fn data() -> MsgSemanticData {
        parse_msg_semantics("17").expect("seed")
    }

    #[test]
    fn translates_known_opcodes_to_names_with_descriptions() {
        let raw = "0:ins_3()\n";
        let out = dmsg_to_readable(raw, &data(), true);
        assert!(out.contains("textboxShow"), "got: {out}");
        assert!(out.contains("// 显示对话框"), "got: {out}");
    }

    #[test]
    fn translates_with_args_and_no_comments() {
        let raw = "0:ins_1(0)\n";
        let out = dmsg_to_readable(raw, &data(), false);
        assert_eq!(out, "0:playerShow(0)\n");
    }

    #[test]
    fn unknown_opcode_preserved_as_ins_n() {
        let raw = "0:ins_999(42)\n";
        let out = dmsg_to_readable(raw, &data(), true);
        assert_eq!(out, "0:ins_999(42)\n");
    }

    #[test]
    fn passthrough_for_non_instruction_lines() {
        let raw = "// a comment line\n\n!EN\nT=\"hello\"\n";
        let out = dmsg_to_readable(raw, &data(), true);
        assert_eq!(out, raw);
    }

    #[test]
    fn preserves_time_label_variants() {
        let raw = "0:ins_3()\n60:ins_3()\n+30:ins_3()\n";
        let out = dmsg_to_readable(raw, &data(), false);
        assert_eq!(out, "0:textboxShow()\n60:textboxShow()\n+30:textboxShow()\n");
    }

    #[test]
    fn roundtrip_known_name_back_to_ins_n() {
        let raw = "0:ins_3()\n0:ins_1(0)\n";
        let readable = dmsg_to_readable(raw, &data(), true);
        let back = readable_to_dmsg(&readable, &data());
        assert_eq!(back, raw);
    }

    #[test]
    fn strips_trailing_comments_in_reverse() {
        let readable = "0:textboxShow() // 显示对话框\n";
        let back = readable_to_dmsg(readable, &data());
        assert_eq!(back, "0:ins_3()\n");
    }

    #[test]
    fn unknown_name_preserved_strip_comment() {
        let readable = "0:somethingUnknown(1) // foo\n";
        let back = readable_to_dmsg(readable, &data());
        assert_eq!(back, "0:somethingUnknown(1)\n");
    }

    #[test]
    fn raw_ins_n_in_readable_survives_reverse_pass() {
        // 用户没翻译的 ins_N 在 readable→dmsg 中应原样保留(comment 已 strip)
        let readable = "0:ins_999(42)\n";
        let back = readable_to_dmsg(readable, &data());
        assert_eq!(back, "0:ins_999(42)\n");
    }
}
