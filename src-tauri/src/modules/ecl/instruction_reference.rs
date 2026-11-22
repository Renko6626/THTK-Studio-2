use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReferenceParameter {
    pub position: u32,
    pub name: String,
    pub direction: Option<String>,
    pub unit: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionReference {
    pub opcode: u32,
    pub zero318_name: Option<String>,
    pub priw8_name: Option<String>,
    pub zero318_source_name: Option<String>,
    pub zero318_source_game: Option<u32>,
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ReferenceParameter>,
    pub reliability: String,
    pub description: Option<String>,
    pub assessment: String,
    pub stock_count: Option<u32>,
    pub stock_version: Option<String>,
    pub stock_example: Option<String>,
    pub dispatch_address: Option<String>,
    pub evidence: Option<String>,
}

impl InstructionReference {
    pub(crate) fn matches(
        &self,
        query_lower: &str,
        opcode: Option<u32>,
        game: Option<u32>,
    ) -> bool {
        if let Some(opcode) = opcode {
            return self.opcode == opcode;
        }
        let local_source_name = self
            .zero318_source_name
            .as_deref()
            .filter(|_| game.is_some() && self.zero318_source_game == game);
        [
            self.zero318_name.as_deref(),
            self.priw8_name.as_deref(),
            local_source_name,
        ]
        .into_iter()
        .flatten()
        .any(|name| name.to_lowercase().contains(query_lower))
    }
}

#[derive(Deserialize)]
struct GameReference {
    game: u32,
    instructions: Vec<InstructionReference>,
}

const REFERENCES: [(u32, &str); 8] = [
    (13, include_str!("../../../assets/ecl-reference/th13.json")),
    (14, include_str!("../../../assets/ecl-reference/th14.json")),
    (15, include_str!("../../../assets/ecl-reference/th15.json")),
    (16, include_str!("../../../assets/ecl-reference/th16.json")),
    (17, include_str!("../../../assets/ecl-reference/th17.json")),
    (18, include_str!("../../../assets/ecl-reference/th18.json")),
    (19, include_str!("../../../assets/ecl-reference/th19.json")),
    (20, include_str!("../../../assets/ecl-reference/th20.json")),
];
static PARSED: [OnceLock<Result<GameReference, String>>; REFERENCES.len()] =
    [const { OnceLock::new() }; REFERENCES.len()];

pub(crate) fn for_game(game: u32) -> Result<Option<&'static [InstructionReference]>, String> {
    let Some(index) = REFERENCES.iter().position(|(id, _)| *id == game) else {
        return Ok(None);
    };
    let reference = PARSED[index].get_or_init(|| {
        let data: GameReference = serde_json::from_str(REFERENCES[index].1)
            .map_err(|error| format!("读取 TH{game} ECL 指令参考失败：{error}"))?;
        if data.game != game {
            return Err(format!(
                "TH{game} ECL 指令参考的作品编号有误：{}",
                data.game
            ));
        }
        Ok(data)
    });
    match reference {
        Ok(data) => Ok(Some(&data.instructions)),
        Err(error) => Err(error.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_games_are_readable_and_have_unique_opcodes() {
        for game in 13..=20 {
            let instructions = for_game(game).unwrap().unwrap();
            assert!(!instructions.is_empty());
            let opcodes: std::collections::HashSet<_> =
                instructions.iter().map(|entry| entry.opcode).collect();
            assert_eq!(opcodes.len(), instructions.len(), "TH{game}");
        }
        assert!(for_game(12).unwrap().is_none());
    }

    #[test]
    fn matches_an_opcode_or_either_name() {
        let entry = InstructionReference {
            opcode: 12,
            zero318_name: Some("jump".into()),
            priw8_name: Some("jmp".into()),
            zero318_source_name: Some("old_jump".into()),
            zero318_source_game: Some(15),
            signature: Some("ot".into()),
            params: vec![],
            reliability: "已核查".into(),
            description: None,
            assessment: String::new(),
            stock_count: None,
            stock_version: None,
            stock_example: None,
            dispatch_address: None,
            evidence: None,
        };
        assert!(entry.matches("12", Some(12), Some(13)));
        assert!(entry.matches("jum", None, Some(13)));
        assert!(entry.matches("jmp", None, Some(13)));
        assert!(!entry.matches("13", Some(13), Some(13)));
        assert!(!entry.matches("old_jump", None, Some(13)));
        assert!(entry.matches("old_jump", None, Some(15)));
    }
}
