use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclMapInstructionParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclMapInstructionSpec {
    pub opcode: u32,
    pub name: String,
    pub section: Option<String>,
    pub signature: Option<String>,
    pub params: Vec<EclMapInstructionParameter>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclMapGlobalVar {
    pub id: i32,
    pub name: String,
    pub var_type: String, // "int" | "float" | "unknown"
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EclMapSemanticData {
    pub source_path: String,
    pub version: String,
    pub instructions: Vec<EclMapInstructionSpec>,
    pub builtins: Vec<String>,
    pub globals: Vec<EclMapGlobalVar>,
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

fn build_signature_params(signature: &str) -> Vec<EclMapInstructionParameter> {
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

        params.push(EclMapInstructionParameter {
            name: format!("{type_name}{}", *counter),
            type_name,
        });
    }

    params
}

fn infer_version_from_path(path: &str) -> String {
    let lower = path.to_lowercase();
    let file_name = lower
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(lower.as_str())
        .to_string();

    let stem = file_name
        .strip_suffix(".eclmap")
        .or_else(|| file_name.strip_suffix(".eclm"))
        .unwrap_or(file_name.as_str());
    stem.trim_start_matches("th").to_string()
}

pub fn parse_ecl_map_file(path: &str) -> Result<EclMapSemanticData, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("Failed to read eclmap file: {error}"))?;
    parse_ecl_map_content(path, &content)
}

pub fn parse_ecl_map_content(path: &str, content: &str) -> Result<EclMapSemanticData, String> {
    let mut in_instruction_names = false;
    let mut in_instruction_signatures = false;
    let mut in_gvar_names = false;
    let mut in_gvar_types = false;
    let mut current_section: Option<String> = None;
    let mut instructions = Vec::new();
    let mut signatures: BTreeMap<u32, String> = BTreeMap::new();
    let mut globals: Vec<EclMapGlobalVar> = Vec::new();
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

        if line.starts_with('!')
            && !line.starts_with("!ins_names")
            && !line.starts_with("!ins_signatures")
            && !line.starts_with("!gvar_names")
            && !line.starts_with("!gvar_types")
        {
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

                instructions.push(EclMapInstructionSpec {
                    opcode,
                    name: name.to_string(),
                    section: current_section.clone(),
                    signature: None,
                    params: Vec::new(),
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
                globals.push(EclMapGlobalVar {
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

    for instruction in &mut instructions {
        if let Some(signature) = signatures.get(&instruction.opcode) {
            if !signature.is_empty() {
                instruction.signature = Some(signature.clone());
                instruction.params = build_signature_params(signature);
            }
        }
    }

    for global in &mut globals {
        if let Some(var_type) = gvar_types.get(&global.id) {
            global.var_type = var_type.clone();
        }
    }

    Ok(EclMapSemanticData {
        source_path: path.to_string(),
        version: infer_version_from_path(path),
        builtins: instructions.iter().map(|item| item.name.clone()).collect(),
        instructions,
        globals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"!eclmap
!ins_names
10 jump
11 callSub
!ins_signatures
10 ot
11 m
!gvar_names
-9985 I0
-9982 F0
!gvar_types
-9985 $
-9982 %
"#;

    #[test]
    fn parses_instructions_and_globals() {
        let data = parse_ecl_map_content("maps/th17.eclm", SAMPLE).expect("parse");
        assert_eq!(data.instructions.len(), 2);
        assert_eq!(data.instructions[0].name, "jump");
        assert_eq!(data.instructions[0].signature.as_deref(), Some("ot"));

        assert_eq!(data.globals.len(), 2);
        let i0 = &data.globals[0];
        assert_eq!(i0.id, -9985);
        assert_eq!(i0.name, "I0");
        assert_eq!(i0.var_type, "int");
        let f0 = &data.globals[1];
        assert_eq!(f0.id, -9982);
        assert_eq!(f0.var_type, "float");
    }

    #[test]
    fn gvar_without_type_is_unknown() {
        let sample = "!gvar_names\n-1 X\n";
        let data = parse_ecl_map_content("maps/th17.eclm", sample).expect("parse");
        assert_eq!(data.globals[0].var_type, "unknown");
    }

    #[test]
    fn infer_version_from_eclmap_extension() {
        let data = parse_ecl_map_content("maps/th17.eclmap", "!ins_names\n").expect("parse");
        assert_eq!(data.version, "17");
    }
}
