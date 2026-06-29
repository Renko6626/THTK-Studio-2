use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StdInstructionParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StdInstructionSpec {
    pub opcode: u32,
    pub name: String,
    #[serde(default)]
    pub params: Vec<StdInstructionParameter>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StdSemanticData {
    pub tool: String,
    pub version: String,
    pub instructions: Vec<StdInstructionSpec>,
}

// 内嵌种子(目前只有 th17,后续按版本扩展时改成 LUT)
const SEED_TH17: &str = include_str!("../../../assets/std-th17.json");

/// 按版本读取语义数据。找不到对应版本则回退到 th17 种子。
/// 版本号已归一化(无 "th" 前缀,例如 "17")。
pub fn parse_std_semantics(version: &str) -> Result<StdSemanticData, String> {
    // 目前只内嵌了 th17;未来扩展按版本 match。任何输入都回退 th17。
    let _ = version;
    serde_json::from_str(SEED_TH17)
        .map_err(|e| format!("Failed to parse embedded std-th17.json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_th17_parses_and_contains_known_instructions() {
        let data = parse_std_semantics("17").expect("parse");
        assert_eq!(data.tool, "thstd");
        assert_eq!(data.version, "17");
        assert!(!data.instructions.is_empty(), "seed must have entries");

        // opcode 1 is the special-case jmp anchor; it must be present and named "jmp"
        // because translator.rs swaps args on opcode 1 exclusively.
        let jmp = data
            .instructions
            .iter()
            .find(|i| i.opcode == 1)
            .expect("opcode 1 must be in seed");
        assert_eq!(jmp.name, "jmp");
    }

    #[test]
    fn unknown_version_falls_back_to_th17() {
        let data = parse_std_semantics("99").expect("parse");
        // Falls back to embedded th17; same shape
        assert_eq!(data.version, "17");
    }

    #[test]
    fn schema_tolerates_missing_optional_fields() {
        let minimal = r#"{"tool":"thstd","version":"17","instructions":[
            {"opcode":42,"name":"foo"}
        ]}"#;
        let data: StdSemanticData = serde_json::from_str(minimal).expect("parse");
        let foo = &data.instructions[0];
        assert_eq!(foo.opcode, 42);
        assert!(foo.params.is_empty());
        assert!(foo.section.is_none());
        assert!(foo.description.is_none());
    }
}
