use super::maps::TruthScriptKind;
use crate::common::project_config::ProjectConfig;
use std::path::PathBuf;
pub fn versions_for_tool(tool: &str) -> Vec<u32> {
    script_check::catalog::versions_for_tool(tool)
}
pub fn map_paths(
    game: u32,
    kind: TruthScriptKind,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
) -> Result<Vec<PathBuf>, String> {
    let snapshot = project.map(|p| script_check::ProjectSnapshot {
        script_maps: script_check::ScriptMaps {
            msg: p.script_maps.msg.clone(),
            ending: p.script_maps.ending.clone(),
            std: p.script_maps.std.clone(),
        },
        ..Default::default()
    });
    script_check::catalog::map_paths(game, kind.into(), root, snapshot.as_ref())
}
#[cfg(test)]
mod tests {
    use super::super::bundled_maps::MAPS;
    use std::path::Path;
    #[test]
    fn all_bundled_profiles_load_with_their_declared_parameter_shapes() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/maps/truth/defaults");
        for (game, kind, file, _) in MAPS {
            let defs =
                super::super::maps::load_truth_maps(*game, (*kind).into(), &[dir.join(file)])
                    .unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(!defs.instructions.is_empty(), "{file}");
            assert!(
                defs.instructions.iter().all(|i| i.params.is_some()),
                "{file}"
            );
        }
    }
}
