use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgInstructionParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgInstructionSpec {
    pub opcode: u32,
    pub name: String,
    #[serde(default)]
    pub params: Vec<MsgInstructionParameter>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsgSemanticData {
    pub tool: String,
    pub version: String,
    pub instructions: Vec<MsgInstructionSpec>,
}

// 内嵌种子(目前只有 th17,后续按版本扩展时改成 LUT)
const SEED_TH17: &str = include_str!("../../../assets/msg-th17.json");

/// 按版本读取语义数据。找不到对应版本则回退到 th17 种子。
/// 版本号已归一化(无 "th" 前缀,例如 "17")。
pub fn parse_msg_semantics(version: &str) -> Result<MsgSemanticData, String> {
    // 目前只内嵌了 th17;未来扩展按版本 match。任何输入都回退 th17。
    let _ = version;
    serde_json::from_str(SEED_TH17)
        .map_err(|e| format!("Failed to parse embedded msg-th17.json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_th17_parses_and_contains_known_instructions() {
        let data = parse_msg_semantics("17").expect("parse");
        assert_eq!(data.tool, "thmsg");
        assert_eq!(data.version, "17");
        assert!(!data.instructions.is_empty(), "seed must have entries");

        // textboxShow opcode 3 is canonical for thmsg
        let textbox = data
            .instructions
            .iter()
            .find(|i| i.name == "textboxShow")
            .expect("textboxShow must be in seed");
        assert_eq!(textbox.opcode, 3);

        // playerShow has one int param
        let player = data
            .instructions
            .iter()
            .find(|i| i.name == "playerShow")
            .expect("playerShow must be in seed");
        assert_eq!(player.opcode, 1);
        assert_eq!(player.params.len(), 1);
        assert_eq!(player.params[0].type_name, "int");
    }

    #[test]
    fn unknown_version_falls_back_to_th17() {
        let data = parse_msg_semantics("99").expect("parse");
        // Falls back to embedded th17; same shape
        assert_eq!(data.version, "17");
    }

    #[test]
    fn schema_tolerates_missing_optional_fields() {
        let minimal = r#"{"tool":"thmsg","version":"17","instructions":[
            {"opcode":42,"name":"foo"}
        ]}"#;
        let data: MsgSemanticData = serde_json::from_str(minimal).expect("parse");
        let foo = &data.instructions[0];
        assert_eq!(foo.opcode, 42);
        assert!(foo.params.is_empty());
        assert!(foo.section.is_none());
        assert!(foo.description.is_none());
    }
}
