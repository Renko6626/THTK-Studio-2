pub use crate::bundled_maps::MAPS;
use crate::{ProjectSnapshot, ScriptKind};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub fn versions_for_tool(tool: &str) -> Vec<u32> {
    MAPS.iter()
        .filter(|(_, kind, _, _)| match tool {
            "trumsg" => *kind == ScriptKind::Msg,
            "trustd" => *kind == ScriptKind::Std,
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
    kind: ScriptKind,
    root: Option<&str>,
    project: Option<&ProjectSnapshot>,
) -> Result<Vec<PathBuf>, String> {
    let (_, _, name, _) = MAPS
        .iter()
        .find(|(g, k, _, _)| *g == game && *k == kind)
        .ok_or_else(|| format!("th{game} 不支持当前 truth 脚本类型 {kind:?}"))?;
    let mut paths = vec![defaults_dir()?.join(name)];
    if let Some(project) = project {
        let custom = match kind {
            ScriptKind::Msg => &project.script_maps.msg,
            ScriptKind::Ending => &project.script_maps.ending,
            ScriptKind::Std => &project.script_maps.std,
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
