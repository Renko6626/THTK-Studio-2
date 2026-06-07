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
pub struct EclMapSemanticData {
    pub source_path: String,
    pub version: String,
    pub instructions: Vec<EclMapInstructionSpec>,
    pub builtins: Vec<String>,
}

fn instruction_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(\d+)\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid regex"))
}

fn signature_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(\d+)\s*(.*)$").expect("valid regex"))
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

    file_name
        .strip_suffix(".eclm")
        .unwrap_or(file_name.as_str())
        .trim_start_matches("th")
        .to_string()
}

pub fn parse_ecl_map_file(path: &str) -> Result<EclMapSemanticData, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("Failed to read eclmap file: {error}"))?;

    let mut in_instruction_names = false;
    let mut in_instruction_signatures = false;
    let mut current_section: Option<String> = None;
    let mut instructions = Vec::new();
    let mut signatures: BTreeMap<u32, String> = BTreeMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("!ins_names") {
            in_instruction_names = true;
            in_instruction_signatures = false;
            current_section = None;
            continue;
        }

        if line.starts_with("!ins_signatures") {
            in_instruction_names = false;
            in_instruction_signatures = true;
            current_section = None;
            continue;
        }

        if line.starts_with("!") && !line.starts_with("!ins_names") && !line.starts_with("!ins_signatures") {
            in_instruction_names = false;
            in_instruction_signatures = false;
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

    Ok(EclMapSemanticData {
        source_path: path.to_string(),
        version: infer_version_from_path(path),
        builtins: instructions.iter().map(|item| item.name.clone()).collect(),
        instructions,
    })
}
