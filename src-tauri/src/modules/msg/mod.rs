pub mod commands;
pub use crate::modules::truth::compiler::ScriptResult as MsgResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MsgScriptKind {
    #[default]
    Stage,
    Ending,
}
impl MsgScriptKind {
    pub fn infer_from_path(path: &str) -> Self {
        let path = std::path::Path::new(path);
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("endspec"))
        {
            return Self::Ending;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("msgspec"))
        {
            return Self::Stage;
        }
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if stem.starts_with("staff")
            || (stem.len() == 3
                && stem.starts_with('e')
                && stem[1..].chars().all(|c| c.is_ascii_digit()))
        {
            Self::Ending
        } else {
            Self::Stage
        }
    }
    pub fn truth_kind(self) -> crate::modules::truth::maps::TruthScriptKind {
        match self {
            Self::Stage => crate::modules::truth::maps::TruthScriptKind::Msg,
            Self::Ending => crate::modules::truth::maps::TruthScriptKind::Ending,
        }
    }
}
