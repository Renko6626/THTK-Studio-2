//! thtk 生态的 mapfile 解析器（eclmap 家族）。
//!
//! 同一套语法服务四种 mapfile：`.eclm` / `.anmm` / `.msgm` / `.stdm`。它们只有
//! 首行的段头不同（`!eclmap` / `!msgmap` / `!stdmap`），而本解析器对**未知 `!` 段
//! 一律跳过**，所以四种都能吃。
//!
//! ```text
//! !msgmap
//! !ins_names
//!
//! 0 end
//! 3 textboxShow   # nop in TH128+
//! ```
//!
//! 格式与 [truth](https://github.com/ExpHP/truth) 的 `map/` 目录一致，两边的表
//! 可以互换——这是刻意的：thmsg / thstd 没有 `-m`，指令名映射由本 IDE 提供，
//! 但数据不该是自创格式，否则既喂不给别人也接不了别人的。

use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapInstruction {
    pub opcode: u32,
    pub name: String,
    pub section: Option<String>,
    pub signature: Option<String>,
    pub params: Vec<MapParameter>,
    /// 中文说明。**不来自 mapfile 本体**——mapfile 保持与生态一致，只有
    /// `opcode name`；说明由旁挂的 `<mapfile>.zh.json` 提供并在加载后合并。
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapGlobal {
    pub id: i32,
    pub name: String,
    pub var_type: String, // "int" | "float" | "unknown"
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapFileData {
    pub source_path: String,
    pub instructions: Vec<MapInstruction>,
    pub globals: Vec<MapGlobal>,
}

fn instruction_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(\d+)\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid regex"))
}

fn signature_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(\d+)\s*(.*)$").expect("valid regex"))
}

fn gvar_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(-?\d+)\s+(\S+)").expect("valid regex"))
}

pub fn build_signature_params(signature: &str) -> Vec<MapParameter> {
    let mut params = Vec::new();
    let mut pending_pointer = false;
    let mut type_index: BTreeMap<String, usize> = BTreeMap::new();

    for ch in signature.chars() {
        if ch.is_whitespace() {
            continue;
        }

        if ch == '*' {
            pending_pointer = true;
            continue;
        }

        let base_type = match ch {
            'S' => "int",
            'f' => "float",
            'm' => "subroutine",
            'D' => "label",
            'o' => "offset",
            't' => "time",
            'x' => "difficulty",
            _ => "arg",
        };

        let type_name = if pending_pointer {
            pending_pointer = false;
            format!("ref_{base_type}")
        } else {
            base_type.to_string()
        };

        let counter = type_index.entry(type_name.clone()).or_insert(0);
        *counter += 1;

        params.push(MapParameter {
            name: format!("{type_name}{}", *counter),
            type_name,
        });
    }

    params
}

pub fn parse_map_file(path: &str) -> Result<MapFileData, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("Failed to read map file: {error}"))?;
    parse_map_content(path, &content)
}

pub fn parse_map_content(path: &str, content: &str) -> Result<MapFileData, String> {
    let mut in_instruction_names = false;
    let mut in_instruction_signatures = false;
    let mut in_gvar_names = false;
    let mut in_gvar_types = false;
    let mut current_section: Option<String> = None;
    let mut instructions: Vec<MapInstruction> = Vec::new();
    let mut signatures: BTreeMap<u32, String> = BTreeMap::new();
    let mut globals: Vec<MapGlobal> = Vec::new();
    let mut gvar_types: BTreeMap<i32, String> = BTreeMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("!ins_names") {
            in_instruction_names = true;
            in_instruction_signatures = false;
            in_gvar_names = false;
            in_gvar_types = false;
            current_section = None;
            continue;
        }

        if line.starts_with("!ins_signatures") {
            in_instruction_names = false;
            in_instruction_signatures = true;
            in_gvar_names = false;
            in_gvar_types = false;
            current_section = None;
            continue;
        }

        if line.starts_with("!gvar_names") {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = true;
            in_gvar_types = false;
            current_section = None;
            continue;
        }

        if line.starts_with("!gvar_types") {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = false;
            in_gvar_types = true;
            current_section = None;
            continue;
        }

        // 未知的 `!` 段（`!msgmap` / `!stdmap` / `!eclmap` / `!gamemap` 等）：
        // 重置状态并跳过。这正是同一个解析器能吃四种 mapfile 的原因。
        if line.starts_with('!') {
            in_instruction_names = false;
            in_instruction_signatures = false;
            in_gvar_names = false;
            in_gvar_types = false;
            current_section = None;
            continue;
        }

        if let Some(section_name) = line.strip_prefix("##") {
            current_section = Some(section_name.trim().to_string());
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        if in_instruction_names {
            if let Some(captures) = instruction_line_regex().captures(line) {
                let opcode = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<u32>().ok())
                    .unwrap_or_default();
                let name = captures.get(2).map(|value| value.as_str()).unwrap_or_default();

                instructions.push(MapInstruction {
                    opcode,
                    name: name.to_string(),
                    section: current_section.clone(),
                    signature: None,
                    params: Vec::new(),
                    description: None,
                });
            }
            continue;
        }

        if in_instruction_signatures {
            if let Some(captures) = signature_line_regex().captures(line) {
                let opcode = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<u32>().ok())
                    .unwrap_or_default();
                let signature = captures
                    .get(2)
                    .map(|value| value.as_str().trim())
                    .unwrap_or_default()
                    .split('#')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                signatures.insert(opcode, signature);
            }
            continue;
        }

        if in_gvar_names {
            if let Some(captures) = gvar_line_regex().captures(line) {
                let id = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<i32>().ok())
                    .unwrap_or_default();
                let name = captures.get(2).map(|v| v.as_str()).unwrap_or_default();
                globals.push(MapGlobal {
                    id,
                    name: name.to_string(),
                    var_type: "unknown".to_string(),
                });
            }
            continue;
        }

        if in_gvar_types {
            if let Some(captures) = gvar_line_regex().captures(line) {
                let id = captures
                    .get(1)
                    .and_then(|value| value.as_str().parse::<i32>().ok())
                    .unwrap_or_default();
                let type_mark = captures.get(2).map(|v| v.as_str()).unwrap_or_default();
                let var_type = match type_mark {
                    "$" => "int",
                    "%" => "float",
                    _ => "unknown",
                };
                gvar_types.insert(id, var_type.to_string());
            }
            continue;
        }
    }

    // 只有 !ins_signatures 段声明过的指令才补签名。
    // 有名字没签名是正常的（多数 msgm/stdm 只写名字）。
    for instruction in &mut instructions {
        if let Some(signature) = signatures.get(&instruction.opcode) {
            if !signature.is_empty() {
                instruction.signature = Some(signature.clone());
                instruction.params = build_signature_params(signature);
            }
        }
    }

    // !ins_signatures 里出现但 !ins_names 里没有的 opcode：作为无名指令收进来。
    // th19/th20 的 MSG 正是这种状态——thtk 源码给了签名，名字还没逆向出来。
    // 不收进来的话，编辑器连参数形状都不知道。
    for (opcode, signature) in &signatures {
        if instructions.iter().any(|i| i.opcode == *opcode) {
            continue;
        }
        instructions.push(MapInstruction {
            opcode: *opcode,
            name: String::new(),
            section: None,
            signature: if signature.is_empty() {
                None
            } else {
                Some(signature.clone())
            },
            params: build_signature_params(signature),
            description: None,
        });
    }
    instructions.sort_by_key(|i| i.opcode);

    for global in &mut globals {
        if let Some(var_type) = gvar_types.get(&global.id) {
            global.var_type = var_type.clone();
        }
    }

    Ok(MapFileData {
        source_path: path.to_string(),
        instructions,
        globals,
    })
}

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
    /// 否则同一个解析器吃不下四种 mapfile。
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

    #[test]
    fn signature_section_fills_params() {
        let content = "\
!msgmap
!ins_names
1 playerShow
!ins_signatures
1 S
";
        let data = parse_map_content("x.msgm", content).unwrap();
        let player = &data.instructions[0];
        assert_eq!(player.signature.as_deref(), Some("S"));
        assert_eq!(player.params.len(), 1);
        assert_eq!(player.params[0].type_name, "int");
    }

    /// th19/th20 的 MSG 正是这种状态：thtk 源码给了签名，名字还没逆向出来。
    /// 只有签名没名字的 opcode 必须也被收进来，否则编辑器连参数形状都不知道。
    #[test]
    fn signature_without_name_is_still_collected() {
        let content = "\
!msgmap
!ins_names
1 playerShow
!ins_signatures
1 S
44 ff
";
        let data = parse_map_content("x.msgm", content).unwrap();
        let unnamed = data.instructions.iter().find(|i| i.opcode == 44).unwrap();
        assert_eq!(unnamed.name, "", "没名字就是空串，不要编一个");
        assert_eq!(unnamed.params.len(), 2);
        assert_eq!(unnamed.params[0].type_name, "float");
    }

    #[test]
    fn empty_content_yields_no_instructions() {
        let data = parse_map_content("empty.msgm", "!msgmap\n").unwrap();
        assert!(data.instructions.is_empty());
        assert!(data.globals.is_empty());
    }
}
