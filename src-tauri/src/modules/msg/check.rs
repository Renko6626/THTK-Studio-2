//! `.dmsg` 编译前预检查，以及 `thmsg` stderr 的结构化诊断。
//!
//! 预检查只做**有确定依据**的判断：
//! - 未知指令名 / 未知 opcode；
//! - 参数个数与版本精确签名不符；
//! - `entry` / `header` 的结构问题；
//! - thtk 已知缺口（40/41/48/49）。
//!
//! 它不尝试替代 thmsg 的完整值解析。目标是让用户在一次点击内拿到位于正确
//! 行号的问题，而不是等 thmsg 退出后只看一段无行号的 stderr。

use crate::common::map_file::{MapFileData, MapInstruction};
use crate::modules::ecl::error_parser::Diagnostic;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::MsgScriptKind;

fn readable_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^([A-Za-z_]\w*)\s*\((.*)\)\s*(?://.*)?$").unwrap())
}

fn raw_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\d+)\s*(?:;(.*))?$").unwrap())
}

fn entry_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^entry\s+(\d+)(?:\s*\((.*)\))?$").unwrap())
}

fn header_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^header\s*\((.*)\)$").unwrap())
}

fn time_label_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^@[+-]?\d+$").unwrap())
}

fn is_integer(text: &str) -> bool {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("-0x").or_else(|| t.strip_prefix("-0X")) {
        return !rest.is_empty() && i64::from_str_radix(rest, 16).is_ok();
    }
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return !rest.is_empty() && i64::from_str_radix(rest, 16).is_ok();
    }
    t.parse::<i64>().is_ok()
}

fn is_float(text: &str) -> bool {
    let t = text.trim().trim_end_matches(['f', 'F']);
    t.parse::<f64>().is_ok()
}

fn param_is_text(param: &crate::common::map_file::MapParameter) -> bool {
    param.type_name == "text" || param.type_name == "string"
}

fn param_accepts(param: &crate::common::map_file::MapParameter, raw: &str) -> bool {
    match param.type_name.as_str() {
        "text" | "string" => true,
        "float" => is_float(raw),
        "int" | "int16" | "byte" | "color" | "difficulty" => is_integer(raw),
        // offset / time / label 的实际写法由 thmsg 决定；预检查不拦。
        _ => true,
    }
}

struct Checker<'a> {
    path: &'a str,
    kind: MsgScriptKind,
    version: u32,
    by_name: HashMap<&'a str, &'a MapInstruction>,
    by_opcode: HashMap<u32, &'a MapInstruction>,
    diagnostics: Vec<Diagnostic>,
    seen: HashSet<String>,
    entry_ids: HashSet<u32>,
    entry_count: usize,
    seen_header: bool,
}

impl<'a> Checker<'a> {
    fn new(
        semantics: &'a MapFileData,
        path: &'a str,
        kind: MsgScriptKind,
        version: u32,
    ) -> Self {
        Self {
            path,
            kind,
            version,
            by_name: semantics
                .instructions
                .iter()
                .map(|instruction| (instruction.name.as_str(), instruction))
                .collect(),
            by_opcode: semantics
                .instructions
                .iter()
                .map(|instruction| (instruction.opcode, instruction))
                .collect(),
            diagnostics: Vec::new(),
            seen: HashSet::new(),
            entry_ids: HashSet::new(),
            entry_count: 0,
            seen_header: false,
        }
    }

    fn push(&mut self, line: u32, severity: &str, message: String) {
        let key = format!("{line}:{severity}:{message}");
        if !self.seen.insert(key) {
            return;
        }
        self.diagnostics.push(Diagnostic {
            path: self.path.to_string(),
            line,
            column: None,
            severity: severity.to_string(),
            message,
        });
    }

    fn warn(&mut self, line: u32, message: impl Into<String>) {
        self.push(line, "warning", message.into());
    }

    fn error(&mut self, line: u32, message: impl Into<String>) {
        self.push(line, "error", message.into());
    }

    fn check_line(&mut self, line_no: u32, trimmed: &str) {
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return;
        }

        if trimmed.starts_with('#') {
            // 方言头会被 strip_dialect_header 剥掉；其余 `#` 注释 thmsg 不认
            // （它只跳过 `//`），必须在这里拦下。
            if !trimmed.starts_with(super::translator::DIALECT_MARKER) {
                self.error(
                    line_no,
                    "thmsg 不支持 `#` 注释；请改用 `//`。".to_string(),
                );
            }
            return;
        }

        // `@` / `header` / `entry` 行允许行尾 `//` 注释——thmsg 的 sscanf
        // 会忽略它。指令行交给下面的正则处理（它能识别行尾注释）。
        let structural = trimmed.split("//").next().unwrap_or(trimmed).trim_end();

        if trimmed.starts_with('@') {
            if !time_label_re().is_match(structural) {
                self.error(
                    line_no,
                    format!("时间标签必须是 @<数字>（thmsg 不接受 {trimmed:?}）"),
                );
            }
            return;
        }

        if let Some(caps) = header_re().captures(structural) {
            self.check_header(line_no, &caps[1]);
            return;
        }

        if let Some(caps) = entry_re().captures(structural) {
            self.check_entry(line_no, &caps[1], caps.get(2).map(|m| m.as_str()));
            return;
        }

        if let Some(caps) = readable_re().captures(trimmed) {
            let name = &caps[1];
            let args = &caps[2];
            self.check_instruction(line_no, name, args);
            return;
        }

        if let Some(caps) = raw_re().captures(trimmed) {
            let opcode = caps[1].parse::<u32>().unwrap_or(u32::MAX);
            let args = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            // 让 check_instruction 走同一条 opcode 路径：`ins_N` 不会被名字表命中。
            self.check_instruction(line_no, &format!("ins_{opcode}"), args);
            return;
        }

        self.error(
            line_no,
            format!("无法识别的行，thmsg 会报 no parse：{trimmed}"),
        );
    }

    fn check_header(&mut self, line_no: u32, body: &str) {
        self.seen_header = true;

        if !(self.kind == MsgScriptKind::Stage && self.version == 19) {
            self.warn(
                line_no,
                if self.kind == MsgScriptKind::Ending {
                    "结局/staff 模式没有 TH19 的 0x50 额外头，thmsg 会忽略这一行。"
                } else {
                    "只有 TH19 的关卡 MSG 有 0x50 额外头；thmsg 会忽略这一行。"
                },
            );
            return;
        }

        let values: Vec<&str> = body.split(',').map(|value| value.trim()).collect();
        if values.len() != 20 {
            self.warn(
                line_no,
                format!(
                    "TH19 的 header 应为 20 个 int32；当前 {} 个，thmsg 会把其余写成 0。",
                    values.len()
                ),
            );
        }
        if values.iter().any(|value| !is_integer(value)) {
            self.error(line_no, "header 里的值必须是 int32。".to_string());
        }
    }

    fn check_entry(&mut self, line_no: u32, id: &str, flags: Option<&str>) {
        let id = match id.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                self.error(line_no, "entry 编号必须是整数。".to_string());
                return;
            }
        };

        if !self.entry_ids.insert(id) {
            self.warn(line_no, format!("entry {id} 重复；thmsg 只会保留最后一次。"));
        }
        self.entry_count += 1;

        // th09 起每条入口是 {offset, flags}，thmsg 要求 flags 必须存在。
        if self.version < 9 {
            return;
        }

        let Some(flags) = flags else {
            self.error(
                line_no,
                format!("th{version} 的 entry 必须写成 `entry {id} (<flags>)`。", version = self.version),
            );
            return;
        };

        let values: Vec<&str> = flags.split(',').map(|value| value.trim()).collect();
        let expected = if self.kind == MsgScriptKind::Stage && self.version == 19 {
            4
        } else {
            1
        };

        if values.len() != expected {
            self.error(
                line_no,
                format!(
                    "th{version} 的 entry flags 应为 {expected} 个字节值，当前 {} 个。",
                    values.len(),
                    version = self.version
                ),
            );
            return;
        }

        if self.kind == MsgScriptKind::Stage && self.version == 19 {
            for value in values {
                match value.parse::<u16>() {
                    Ok(v) if v <= 255 => {}
                    _ => {
                        self.error(line_no, "TH19 的 4 个 entry flags 必须是 0–255 的整数。".to_string());
                        return;
                    }
                }
            }
        } else if values[0].parse::<u32>().is_err() {
            // th09+ 的其余版本把 flags 当一个 u32 写回；真实文件通常是 256。
            self.error(line_no, "entry flags 必须是 32 位无符号整数。".to_string());
        }
    }

    fn check_instruction(&mut self, line_no: u32, name: &str, args: &str) {
        let resolved: Option<&MapInstruction> = if name.starts_with("ins_") {
            name.strip_prefix("ins_")
                .and_then(|n| n.parse::<u32>().ok())
                .and_then(|opcode| self.by_opcode.get(&opcode).copied())
        } else {
            self.by_name.get(name).copied()
        };

        let Some(instruction) = resolved else {
            let opcode = name
                .strip_prefix("ins_")
                .and_then(|n| n.parse::<u32>().ok());
            let message = match opcode {
                Some(op) if self.kind == MsgScriptKind::Stage && matches!(op, 40 | 41 | 48 | 49) => {
                    format!(
                        "thtk 缺少 opcode {op} 的格式表项，thmsg 会直接失败（已知上游缺口）。"
                    )
                }
                Some(op) if self.kind == MsgScriptKind::Ending && matches!(op, 4 | 13) => {
                    format!(
                        "thtk 的结局格式表没有 opcode {op}（引擎里有分支，但 thmsg -c 会拒绝）。"
                    )
                }
                Some(op) => format!(
                    "th{} 的{}表里没有 opcode {op}；thmsg 也会拒绝。",
                    self.version,
                    self.kind.label_zh()
                ),
                None => format!(
                    "th{} 的{}表里没有指令名 `{name}`。",
                    self.version,
                    self.kind.label_zh()
                ),
            };
            self.error(line_no, message);
            return;
        };

        // thtk 的 th16–18 表要求 musicBoss 带参数，但真实文件是零参数。
        // 两种写法都接受；编译时由 apply_thtk_quirks 补齐缺失的那个。
        if self.kind == MsgScriptKind::Stage
            && matches!(self.version, 16 | 17 | 165 | 18)
            && instruction.opcode == 19
        {
            if args.trim().is_empty() {
                return;
            }
            if args.split(';').count() == 1 && is_integer(args.trim()) {
                return;
            }
            self.error(
                line_no,
                "TH16–18 的 musicBoss 可省略参数；若填写，只能是一个整数。".to_string(),
            );
            return;
        }

        let expected = instruction.params.len();
        let display_name = if instruction.name.is_empty() {
            format!("ins_{}", instruction.opcode)
        } else {
            instruction.name.clone()
        };

        if expected == 0 {
            if !args.trim().is_empty() {
                self.error(
                    line_no,
                    format!("`{display_name}` 不接受参数，但写入了 `{args}`。"),
                );
            }
            return;
        }

        let string_pos = instruction.params.iter().position(param_is_text);

        // 只有一个文本/字符串参数：参数里可以出现分号，整段都是正文。
        if expected == 1 && string_pos == Some(0) {
            return;
        }

        if args.trim().is_empty() {
            self.error(
                line_no,
                format!("`{display_name}` 需要 {expected} 个参数，当前 0 个。"),
            );
            return;
        }

        let parts: Vec<&str> = args.split(';').collect();
        let trailing_string = string_pos == Some(expected - 1);

        if trailing_string {
            if parts.len() < expected {
                self.error(
                    line_no,
                    format!(
                        "`{display_name}` 需要 {expected} 个参数（最后一个可为文本），当前 {} 个。",
                        parts.len()
                    ),
                );
                return;
            }
        } else if parts.len() != expected {
            self.error(
                line_no,
                format!(
                    "`{display_name}` 需要 {expected} 个参数，当前 {} 个。",
                    parts.len()
                ),
            );
            return;
        }

        for index in 0..expected {
            if Some(index) == string_pos {
                continue;
            }
            let raw = parts.get(index).map(|value| value.trim()).unwrap_or("");
            if !param_accepts(&instruction.params[index], raw) {
                self.error(
                    line_no,
                    format!(
                        "`{display_name}` 的第 {} 个参数应为 {}，当前是 `{raw}`。",
                        index + 1,
                        instruction.params[index].type_name
                    ),
                );
            }
        }
    }

    fn finish(mut self) -> Vec<Diagnostic> {
        if self.entry_count == 0 {
            self.error(1, "文件里没有 `entry`；thmsg 需要一个入口表。".to_string());
        }
        if self.kind == MsgScriptKind::Stage
            && self.version == 19
            && !self.seen_header
        {
            self.warn(
                1,
                "TH19 的关卡 MSG 应带一行 `header(...)`；缺失时 thmsg 会把 0x50 头写成全零。"
                    .to_string(),
            );
        }

        self.diagnostics.sort_by_key(|d| (d.line, d.severity != "error"));
        self.diagnostics
    }
}

/// 对可读 `.dmsg` 做编译前检查。`content` 应包含方言头——行号按原始文件算。
pub fn check_readable(
    content: &str,
    path: &str,
    semantics: &MapFileData,
    kind: MsgScriptKind,
    version: u32,
) -> Vec<Diagnostic> {
    let mut checker = Checker::new(semantics, path, kind, version);

    for (index, raw) in content.lines().enumerate() {
        checker.check_line(index as u32 + 1, raw.trim());
    }

    checker.finish()
}

/// 把 `thmsg` 的 stderr 变成带行号的诊断。
///
/// thmsg 的 `no parse` 消息带的是**行内容**而不是行号（`program.c` 没有维护
/// 行计数器），所以这里按内容回查源文件。编译时传给 thmsg 的是去掉方言头后的
/// 临时文件，`line_offset` 就是被剥掉的行数。
pub fn parse_thmsg_stderr(
    stderr: &str,
    path: &str,
    source: &str,
    line_offset: u32,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    for raw in stderr.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload = trimmed
            .strip_prefix("thmsg.exe:")
            .or_else(|| trimmed.strip_prefix("thmsg:"))
            .unwrap_or(trimmed)
            .trim();

        let (line, message) = if let Some(pos) = payload.find("no parse:") {
            let buffer = payload[pos + "no parse:".len()..].trim();
            let line = find_source_line(source, buffer)
                .map(|line| line + line_offset)
                .unwrap_or(1);
            (line, format!("thmsg 无法解析这一行：{buffer}"))
        } else if payload.contains("was not found in the") {
            let known_gap = ["id 40 ", "id 41 ", "id 48 ", "id 49 "]
                .iter()
                .any(|needle| payload.contains(needle));
            let hint = if known_gap {
                "（thtk 已知缺口：该 opcode 没有任何格式表项）"
            } else {
                "（当前版本的格式表没有收录这个 opcode）"
            };
            (1, format!("{payload}{hint}"))
        } else {
            (1, payload.to_string())
        };

        let key = format!("{line}:{message}");
        if seen.insert(key) {
            diagnostics.push(Diagnostic {
                path: path.to_string(),
                line,
                column: None,
                severity: "error".to_string(),
                message,
            });
        }
    }

    diagnostics
}

fn find_source_line(source: &str, needle: &str) -> Option<u32> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }

    source.lines().position(|line| {
        let line = line.trim();
        line == needle || (!line.is_empty() && line.starts_with(needle))
    }).map(|index| index as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::msg::map_parser::parse_msg_semantics;

    fn stage17() -> MapFileData {
        parse_msg_semantics("17", MsgScriptKind::Stage).expect("stage seed")
    }

    fn ending18() -> MapFileData {
        parse_msg_semantics("18", MsgScriptKind::Ending).expect("ending seed")
    }

    #[test]
    fn accepts_a_normal_stage_file() {
        let content = "# THTK-Studio dmsg 方言（关卡对话）\nentry 0 (256)\n@0\n\ttextboxShow()\n\tplayerShow(0)\n\ttextAdd(こんにちは;世界)\n";
        let diagnostics = check_readable(content, "/p/st01.dmsg", &stage17(), MsgScriptKind::Stage, 17);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn reports_unknown_instruction_and_arity() {
        let content = "entry 0 (256)\n@0\n\tnotAnInstruction()\n\tplayerShow()\n\ttextboxShow(1)\n";
        let diagnostics = check_readable(content, "/p/st01.dmsg", &stage17(), MsgScriptKind::Stage, 17);
        let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert!(messages.iter().any(|m| m.contains("notAnInstruction")));
        assert!(messages.iter().any(|m| m.contains("playerShow") && m.contains("1 个参数")));
        assert!(messages.iter().any(|m| m.contains("textboxShow") && m.contains("不接受参数")));
        assert!(diagnostics.iter().all(|d| d.severity == "error"));
    }

    #[test]
    fn line_numbers_include_the_dialect_header() {
        let content = "# THTK-Studio dmsg 方言（关卡对话）\nentry 0 (256)\n@0\n\tnotAnInstruction()\n";
        let diagnostics = check_readable(
            content,
            "/p/x.dmsg",
            &stage17(),
            MsgScriptKind::Stage,
            17,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 4);
    }

    #[test]
    fn hash_comments_are_rejected_because_thmsg_only_skips_slash_comments() {
        let content = "# THTK-Studio dmsg 方言（关卡对话）\n# 用户注释\nentry 0 (256)\n\ttextboxShow()\n";
        let diagnostics = check_readable(
            content,
            "/p/x.dmsg",
            &stage17(),
            MsgScriptKind::Stage,
            17,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
        assert!(diagnostics[0].message.contains("//"));
    }

    #[test]
    fn reports_missing_entry_and_bad_time_label() {
        let content = "@abc\n\ttextboxShow()\n";
        let diagnostics = check_readable(content, "/p/st01.dmsg", &stage17(), MsgScriptKind::Stage, 17);
        assert!(diagnostics.iter().any(|d| d.message.contains("没有 `entry`")));
        assert!(diagnostics.iter().any(|d| d.message.contains("时间标签")));
    }

    #[test]
    fn th19_requires_four_flag_bytes() {
        let semantics = parse_msg_semantics("19", MsgScriptKind::Stage).expect("th19");
        let bad = "entry 0 (256)\n\ttextboxShow()\n";
        let diagnostics = check_readable(bad, "/p/t19.dmsg", &semantics, MsgScriptKind::Stage, 19);
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("entry flags") && d.line == 1));

        let goodbye = "header(0, 0)\nentry 0 (1, 2, 3, 4)\n\ttextboxShow()\n";
        let diagnostics = check_readable(goodbye, "/p/t19.dmsg", &semantics, MsgScriptKind::Stage, 19);
        assert!(diagnostics.iter().any(|d| d.message.contains("header 应为 20")));
        assert!(!diagnostics.iter().any(|d| d.message.contains("entry flags")));
    }

    #[test]
    fn ending_file_accepts_ending_names_and_rejects_stage_names() {
        let content = "entry 0 (256)\n\ttextAdd(一行文本)\n\tanmSetSlot(0;0;1)\n\tplayerShow(0)\n";
        let diagnostics = check_readable(content, "/p/e01.dmsg", &ending18(), MsgScriptKind::Ending, 18);
        assert!(diagnostics.iter().any(|d| d.message.contains("playerShow")));
        assert!(!diagnostics.iter().any(|d| d.message.contains("textAdd")));
        assert!(!diagnostics.iter().any(|d| d.message.contains("anmSetSlot")));
    }

    /// 真实 TH18 `e01.msg` 的 thmsg dump 形状（`thmsg -e -d 18`）。
    /// 结局表里 7/8/9/10 这些参数组合必须全部通过，否则预检查会误伤正常文件。
    #[test]
    fn accepts_a_real_ending_dump_shape() {
        let content = "entry 0 (256)\n@0\n\t7;0;e01.anm\n\t8;0;0;0\n\t10;bgm/th18_15\n\t9;13680832\n\t3;　　博麗神社。\n@1\n\t5;6000000\n\t3;次の行\n@2\n\t6;6000000\n\t11\n\t14;60\n\t0\n";
        let diagnostics = check_readable(
            content,
            "/p/e01.dmsg",
            &ending18(),
            MsgScriptKind::Ending,
            18,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn thtk_gap_opcodes_get_a_specific_hint() {
        let content = "entry 0 (256)\n@0\n\tins_48(1)\n";
        let diagnostics = check_readable(content, "/p/x.dmsg", &stage17(), MsgScriptKind::Stage, 17);
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("thtk 缺少 opcode 48")));
    }

    #[test]
    fn ending_thtk_gap_opcodes_get_a_specific_hint() {
        let content = "entry 0 (256)\n\tins_4()\n\tins_13(60)\n";
        let diagnostics = check_readable(
            content,
            "/p/e01.dmsg",
            &ending18(),
            MsgScriptKind::Ending,
            18,
        );
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("结局格式表没有 opcode 4")));
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("结局格式表没有 opcode 13")));
    }

    #[test]
    fn stderr_no_parse_maps_back_to_the_source_line() {
        let source = "entry 0 (256)\n\ttextboxShow()\n\tbad(1)\n";
        let stderr = "thmsg:/tmp/thtk-msg-1.dmsg: no parse: \tbad(1)\n";
        // source 是剥掉方言头后的临时文件内容；line_offset=1 表示原始文件
        // 在第 1 行上方还有一行方言头。
        let diagnostics = parse_thmsg_stderr(stderr, "/p/x.dmsg", source, 1);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 4);
        assert_eq!(diagnostics[0].path, "/p/x.dmsg");
        assert!(diagnostics[0].message.contains("bad(1)"));
    }

    #[test]
    fn stderr_format_table_gap_is_explained() {
        let stderr = "thmsg: id 48 was not found in the format table\n";
        let diagnostics = parse_thmsg_stderr(stderr, "/p/x.msg", "", 0);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("thtk"));
    }
}
