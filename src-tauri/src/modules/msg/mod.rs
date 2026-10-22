pub mod check;
pub mod commands;
pub mod compiler;
pub mod map_parser;
pub mod signatures;
pub mod translator;

use serde::{Deserialize, Serialize};
use std::path::Path;

/// `.msg` 的两套消费者。文件格式相同，但 opcode 表完全不同。
///
/// - `Stage`：关卡内对话（`st01a.msg`…），由 `GuiMsgVm` 执行。
/// - `Ending`：结局与 staff roll（`e01.msg`、`staff1.msg`…），由 `Ending`
///   场景自己的解释器执行。thmsg 用 `-e` 切换。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MsgScriptKind {
    #[default]
    Stage,
    Ending,
}

impl MsgScriptKind {
    /// 写进 `MsgResult.scriptKind`，也用于方言头与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            MsgScriptKind::Stage => "msg",
            MsgScriptKind::Ending => "ending",
        }
    }

    pub fn label_zh(self) -> &'static str {
        match self {
            MsgScriptKind::Stage => "关卡对话",
            MsgScriptKind::Ending => "结局/staff",
        }
    }

    /// 按文件名推断类型。结局与 staff 的命名是固定的：
    /// `e01.msg`…`e12.msg`、`staff1.msg`…`staff4.msg`。
    ///
    /// 只看 stem，所以 `.msg` 与 `.dmsg` 都适用；改名后的文件仍可靠方言头
    /// 兜底（见 `translator::dialect_kind`）。
    pub fn infer_from_path(path: &str) -> Self {
        let stem = Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let looks_like_ending = stem.starts_with("staff")
            || (stem.len() == 3
                && stem.starts_with('e')
                && stem[1..].chars().all(|c| c.is_ascii_digit()));

        if looks_like_ending {
            MsgScriptKind::Ending
        } else {
            MsgScriptKind::Stage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_ending_from_standard_file_names() {
        for path in ["/p/e01.msg", "/p/e12.dmsg", "/p/staff1.msg", "/p/staff4.dmsg"] {
            assert_eq!(
                MsgScriptKind::infer_from_path(path),
                MsgScriptKind::Ending,
                "{path}"
            );
        }
    }

    #[test]
    fn defaults_to_stage_for_stage_files_and_odd_names() {
        for path in ["/p/st01a.msg", "/p/st07d.dmsg", "/p/ending.msg", "/p/e1.msg"] {
            assert_eq!(
                MsgScriptKind::infer_from_path(path),
                MsgScriptKind::Stage,
                "{path}"
            );
        }
    }
}
