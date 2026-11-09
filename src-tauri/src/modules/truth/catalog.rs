use super::{bundled_maps::MAPS, maps::TruthScriptKind};
use crate::common::project_config::ProjectConfig;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub fn versions_for_tool(tool: &str) -> Vec<u32> {
    MAPS.iter()
        .filter(|(_, kind, _, _)| match tool {
            "trumsg" => *kind == TruthScriptKind::Msg,
            "trustd" => *kind == TruthScriptKind::Std,
            _ => false,
        })
        .map(|(game, _, _, _)| *game)
        .collect()
}

fn defaults_dir() -> Result<&'static PathBuf, String> {
    static DIRECTORY: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    DIRECTORY
        .get_or_init(|| {
            let base = directories::ProjectDirs::from("com", "abl", "thtk-studio")
                .ok_or("Cannot locate application data directory")?;
            let dir = base.data_dir().join("deps/maps/truth-c78a541-v1");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            for (_, _, name, content) in MAPS {
                let path = dir.join(name);
                if std::fs::read_to_string(&path).ok().as_deref() != Some(content) {
                    std::fs::write(&path, content)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                }
            }
            Ok(dir)
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn map_paths(
    game: u32,
    kind: TruthScriptKind,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
) -> Result<Vec<PathBuf>, String> {
    let (_, _, name, _) = MAPS
        .iter()
        .find(|(g, k, _, _)| *g == game && *k == kind)
        .ok_or_else(|| format!("th{game} 不支持当前 truth 脚本类型 {kind:?}"))?;
    let mut paths = vec![defaults_dir()?.join(name)];
    if let Some(project) = project {
        let custom = match kind {
            TruthScriptKind::Msg => &project.script_maps.msg,
            TruthScriptKind::Ending => &project.script_maps.ending,
            TruthScriptKind::Std => &project.script_maps.std,
        };
        for path in custom {
            let path = Path::new(path);
            paths.push(if path.is_absolute() {
                path.into()
            } else {
                Path::new(root.unwrap_or(".")).join(path)
            });
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_bundled_profiles_load_with_their_declared_parameter_shapes() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/maps/truth/defaults");
        for (game, kind, file, _) in MAPS {
            let defs = super::super::maps::load_truth_maps(*game, *kind, &[dir.join(file)])
                .unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(!defs.instructions.is_empty(), "{file}");
            assert!(
                defs.instructions.iter().all(|i| i.params.is_some()),
                "{file}"
            );
        }
    }
}
