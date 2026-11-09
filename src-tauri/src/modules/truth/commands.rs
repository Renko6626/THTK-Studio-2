use super::maps::{load_truth_maps, TruthDefinitions, TruthScriptKind};
use std::path::PathBuf;

/// Parse the explicit map list for inspection. Builtin version data and source
/// pragmas are intentionally not guessed by this low-level entry point.
#[tauri::command]
pub async fn get_truth_map_definitions(
    game_version: u32,
    script_kind: TruthScriptKind,
    map_paths: Vec<String>,
) -> Result<TruthDefinitions, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths: Vec<PathBuf> = map_paths.into_iter().map(PathBuf::from).collect();
        load_truth_maps(game_version, script_kind, &paths)
    })
    .await
    .map_err(|e| format!("truth map task failed: {e}"))?
}

#[tauri::command]
pub async fn get_truth_source_definitions(
    state: tauri::State<'_, crate::app_state::AppState>,
    input_path: String,
    script_kind: TruthScriptKind,
    source_text: String,
) -> Result<TruthDefinitions, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = crate::common::toolchain::effective_context(
        &state.config_manager.get_config(),
        root.as_deref(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let game = crate::common::game_version::resolve_from(
            ctx.project.as_ref(),
            &ctx.config,
            script_kind.tool(),
        )?;
        source_definitions(
            game,
            script_kind,
            root.as_deref(),
            ctx.project.as_ref(),
            &input_path,
            &source_text,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

pub fn source_definitions(
    game: u32,
    kind: TruthScriptKind,
    root: Option<&str>,
    project: Option<&crate::common::project_config::ProjectConfig>,
    path: &str,
    text: &str,
) -> Result<TruthDefinitions, String> {
    let mut paths = super::catalog::map_paths(game, kind, root, project)?;
    let cwd = std::path::Path::new(path)
        .parent()
        .unwrap_or(std::path::Path::new(root.unwrap_or(".")));
    paths.extend(super::source::pragma_maps(text, cwd)?);
    let mut definitions = load_truth_maps(game, kind, &paths)?;
    let descriptions = match kind {
        TruthScriptKind::Msg if game == 19 => {
            include_str!("../../../assets/maps/th19.msgm.zh.json")
        }
        TruthScriptKind::Msg if game == 20 => {
            include_str!("../../../assets/maps/th20.msgm.zh.json")
        }
        TruthScriptKind::Msg if game >= 10 => {
            include_str!("../../../assets/maps/th11.msgm.zh.json")
        }
        TruthScriptKind::Ending if game == 20 => {
            include_str!("../../../assets/maps/th20.endm.zh.json")
        }
        TruthScriptKind::Ending => include_str!("../../../assets/maps/th10.endm.zh.json"),
        TruthScriptKind::Std if game >= 14 && game < 100 => {
            include_str!("../../../assets/maps/th14.stdm.zh.json")
        }
        _ => "{}",
    };
    let docs: std::collections::BTreeMap<String, String> =
        serde_json::from_str(descriptions).map_err(|e| e.to_string())?;
    for instruction in &mut definitions.instructions {
        // TH11 inserted speakerNone at 9. TH10's later named operations are one lower.
        let doc_opcode = if kind == TruthScriptKind::Msg && game == 10 {
            match instruction.opcode {
                0..=8 => instruction.opcode,
                9..=23 => instruction.opcode + 1,
                _ => continue,
            }
        } else {
            instruction.opcode
        };
        // Scene games use this opcode for a tutorial, not portrait darkening.
        if kind == TruthScriptKind::Msg && [143, 165].contains(&game) && instruction.opcode == 33 {
            continue;
        }
        instruction.description = docs.get(&doc_opcode.to_string()).cloned();
    }
    Ok(definitions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn descriptions_match_th10_numbering_and_th20_ending() {
        let msg = source_definitions(
            10,
            TruthScriptKind::Msg,
            None,
            None,
            "/tmp/test.msgspec",
            "",
        )
        .unwrap();
        let wait = msg.instructions.iter().find(|i| i.opcode == 10).unwrap();
        assert_eq!(wait.name.as_deref(), Some("textPause"));
        assert!(wait.description.as_deref().unwrap().contains("等待玩家"));
        let clear = msg.instructions.iter().find(|i| i.opcode == 17).unwrap();
        assert!(clear.description.as_deref().unwrap().contains("清除"));
        let end = source_definitions(
            20,
            TruthScriptKind::Ending,
            None,
            None,
            "/tmp/test.endspec",
            "",
        )
        .unwrap();
        assert!(end
            .instructions
            .iter()
            .find(|i| i.opcode == 12)
            .unwrap()
            .description
            .as_deref()
            .unwrap()
            .contains("文件名"));
    }
    #[test]
    fn scene_games_use_tutorial_instead_of_portrait_darkening() {
        for game in [143, 165] {
            let defs = source_definitions(
                game,
                TruthScriptKind::Msg,
                None,
                None,
                "/tmp/test.msgspec",
                "",
            )
            .unwrap();
            let instruction = defs.instructions.iter().find(|i| i.opcode == 33).unwrap();
            assert_eq!(instruction.name.as_deref(), Some("tutorial"));
            assert_eq!(instruction.params.as_ref().unwrap().len(), 1);
            assert!(!instruction
                .description
                .as_deref()
                .unwrap_or("")
                .contains("变暗"));
        }
    }
}
