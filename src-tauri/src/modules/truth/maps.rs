//! truth map 读取与编辑器可见的指令定义。编码有效性仍交给 truth。

use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, rmcp::schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TruthScriptKind {
    Msg,
    Ending,
    Std,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruthParameter {
    #[serde(rename = "type")]
    pub type_name: String,
    pub encoding: String,
    pub enum_name: Option<String>,
    pub attributes: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruthInstruction {
    pub opcode: u16,
    pub description: Option<String>,
    pub name: Option<String>,
    pub aliases: Vec<String>,
    /// None = 未提供签名；Some("") = 明确无参。
    pub signature: Option<String>,
    pub params: Option<Vec<TruthParameter>>,
    pub intrinsic: Option<String>,
    pub name_source: Option<String>,
    pub signature_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruthDefinitions {
    pub game_version: u32,
    pub script_kind: TruthScriptKind,
    pub source_paths: Vec<String>,
    pub instructions: Vec<TruthInstruction>,
    /// 枚举名 → 成员名 → 数值，保留同值的多个别名。
    pub enums: BTreeMap<String, BTreeMap<String, i32>>,
}

pub use super::signature::parse_truth_signature;

#[derive(Debug)]
struct MapEntry {
    section: String,
    key: i32,
    value: String,
    line: usize,
}

struct MapDocument {
    magic: String,
    entries: Vec<MapEntry>,
}

impl TruthScriptKind {
    fn magic(self) -> &'static str {
        match self {
            Self::Msg => "!msgmap",
            Self::Ending => "!endmap",
            Self::Std => "!stdmap",
        }
    }
}

fn read_map(path: &Path) -> Result<MapDocument, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = content.lines();
    let magic = lines.next().unwrap_or_default().trim().to_string();
    if !["!gamemap", "!msgmap", "!endmap", "!stdmap"].contains(&magic.as_str()) {
        return Err(format!(
            "{}: unsupported truth map kind {magic:?}",
            path.display()
        ));
    }
    let mut section = String::new();
    let mut entries = Vec::new();
    for (index, raw) in lines.enumerate() {
        // truth seqmap treats # as a comment delimiter, including inside a value.
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('!') {
            section = header.to_string();
            continue;
        }
        let (key, value) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let key = key
            .parse::<i32>()
            .map_err(|_| format!("{}:{}: invalid map key {key:?}", path.display(), index + 2))?;
        if section.is_empty() {
            return Err(format!(
                "{}:{}: missing map section",
                path.display(),
                index + 2
            ));
        }
        entries.push(MapEntry {
            section: section.clone(),
            key,
            value: value.trim().into(),
            line: index + 2,
        });
    }
    Ok(MapDocument { magic, entries })
}

fn select_map(path: &Path, game: u32) -> Result<(PathBuf, MapDocument, Vec<String>), String> {
    let path = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut sources = vec![path.to_string_lossy().into_owned()];
    let map = read_map(&path)?;
    if map.magic != "!gamemap" {
        return Ok((path, map, sources));
    }
    let mut game_files = BTreeMap::new();
    for entry in map.entries.iter().filter(|e| e.section == "game_files") {
        if game_files.insert(entry.key, &entry.value).is_some() {
            return Err(format!(
                "{}:{}: duplicate game {}",
                path.display(),
                entry.line,
                entry.key
            ));
        }
    }
    let file = i32::try_from(game)
        .ok()
        .and_then(|g| game_files.get(&g))
        .ok_or_else(|| format!("{}: no map for game {game}", path.display()))?;
    let selected = path.parent().unwrap_or(Path::new(".")).join(file);
    let selected = selected
        .canonicalize()
        .map_err(|e| format!("{}: {e}", selected.display()))?;
    let map = read_map(&selected)?;
    sources.push(selected.to_string_lossy().into_owned());
    // Like truth, gamemap selects one script map; it is not recursive inclusion.
    Ok((selected, map, sources))
}

fn valid_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Reads precisely the supplied map list, in order. No compiler builtins, environment
/// lookup or source pragmas are injected here. The caller must supply those inputs
/// explicitly when assembling a project's full vocabulary.
pub fn load_truth_maps(
    game: u32,
    kind: TruthScriptKind,
    paths: &[PathBuf],
) -> Result<TruthDefinitions, String> {
    let mut instructions: BTreeMap<u16, TruthInstruction> = BTreeMap::new();
    let mut alias_owners: BTreeMap<String, (u16, String)> = BTreeMap::new();
    let mut enums: BTreeMap<String, BTreeMap<String, i32>> = BTreeMap::new();
    let mut source_paths = Vec::new();
    for path in paths {
        let (selected, map, sources) = select_map(path, game)?;
        if map.magic != kind.magic() {
            return Err(format!(
                "{}: map kind {} does not match {}",
                selected.display(),
                map.magic,
                kind.magic()
            ));
        }
        source_paths.extend(sources);
        let source = selected.to_string_lossy().into_owned();
        for entry in map.entries {
            let error = |message: String| format!("{source}:{}: {message}", entry.line);
            if let Some(name) = entry
                .section
                .strip_prefix("enum(name=\"")
                .and_then(|s| s.strip_suffix("\")"))
            {
                if !valid_ident(name) || !valid_ident(&entry.value) {
                    return Err(error("invalid enum identifier".into()));
                }
                let values = enums.entry(name.into()).or_default();
                if let Some(previous) = values.get(&entry.value) {
                    if *previous != entry.key {
                        return Err(error(format!(
                            "conflicting enum member {name}.{}",
                            entry.value
                        )));
                    }
                }
                values.insert(entry.value, entry.key);
                continue;
            }
            if !["ins_names", "ins_signatures", "ins_intrinsics"].contains(&entry.section.as_str())
            {
                continue;
            }
            let opcode = u16::try_from(entry.key)
                .map_err(|_| error(format!("invalid opcode {}", entry.key)))?;
            let instruction = instructions
                .entry(opcode)
                .or_insert_with(|| TruthInstruction {
                    opcode,
                    description: None,
                    name: None,
                    aliases: Vec::new(),
                    signature: None,
                    params: None,
                    intrinsic: None,
                    name_source: None,
                    signature_source: None,
                });
            match entry.section.as_str() {
                "ins_names" => {
                    if !valid_ident(&entry.value) || entry.value.starts_with("ins_") {
                        return Err(error(format!("invalid instruction name {:?}", entry.value)));
                    }
                    alias_owners.insert(entry.value.clone(), (opcode, source.clone()));
                    instruction.aliases.retain(|name| name != &entry.value);
                    instruction.aliases.push(entry.value);
                }
                "ins_signatures" => {
                    let params = parse_truth_signature(&entry.value).map_err(error)?;
                    instruction.signature = Some(entry.value);
                    instruction.params = Some(params);
                    instruction.signature_source = Some(source.clone());
                }
                "ins_intrinsics" => instruction.intrinsic = Some(entry.value),
                _ => unreachable!(),
            }
        }
    }
    for instruction in instructions.values_mut() {
        instruction
            .aliases
            .retain(|name| alias_owners[name].0 == instruction.opcode);
        instruction.name = instruction.aliases.last().cloned();
        instruction.name_source = instruction
            .name
            .as_ref()
            .map(|name| alias_owners[name].1.clone());
    }
    Ok(TruthDefinitions {
        game_version: game,
        script_kind: kind,
        source_paths,
        instructions: instructions.into_values().collect(),
        enums,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_attributes_do_not_become_parameters() {
        let params =
            parse_truth_signature(r#"SU(enum="InterpMode")fffb---z(bs=4;mask=0x77,7,16;furibug)"#)
                .unwrap();
        assert_eq!(params.len(), 7);
        assert_eq!(params[1].enum_name.as_deref(), Some("InterpMode"));
        assert_eq!(params[6].type_name, "string");
        assert_eq!(params[6].attributes["mask"].as_deref(), Some("0x77,7,16"));
        assert_eq!(params[6].attributes["furibug"], None);
        assert!(parse_truth_signature("").unwrap().is_empty());
        assert!(parse_truth_signature(r#"S(enum="Unclosed)"#).is_err());
    }

    fn write(dir: &Path, file: &str, contents: &str) -> PathBuf {
        let path = dir.join(file);
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn maps_merge_names_signatures_and_enum_aliases_independently() {
        let dir = tempfile::tempdir().unwrap();
        let base = write(dir.path(), "base.stdm", "!stdmap\n!ins_names\n0 stop\n3 posTime\n4 namedOnly\n!ins_signatures\n0\n3 SU(enum=\"InterpMode\")fff\n!enum(name=\"InterpMode\")\n0 Linear\n");
        let overlay = write(dir.path(), "overlay.stdm", "!stdmap\n!ins_names\n3 cameraMove\n!ins_signatures\n3 SSfff\n!enum(name=\"InterpMode\")\n0 Constant\n1 Accelerate\n");
        let result = load_truth_maps(20, TruthScriptKind::Std, &[base, overlay.clone()]).unwrap();
        let stop = &result.instructions[0];
        assert_eq!(stop.signature.as_deref(), Some(""));
        assert!(stop.params.as_ref().unwrap().is_empty());
        let pos = &result.instructions[1];
        assert_eq!(pos.name.as_deref(), Some("cameraMove"));
        assert_eq!(pos.aliases, ["posTime", "cameraMove"]);
        assert_eq!(pos.params.as_ref().unwrap().len(), 5);
        assert_eq!(pos.signature_source.as_deref(), overlay.to_str());
        assert_eq!(result.instructions[2].params, None);
        assert_eq!(result.enums["InterpMode"]["Linear"], 0);
        assert_eq!(result.enums["InterpMode"]["Constant"], 0);
    }

    #[test]
    fn gamemap_resolves_relative_to_itself_and_keeps_script_kinds_separate() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "game.endm",
            "!endmap\n!ins_names\n3 text\n!ins_signatures\n3 z(bs=4;mask=0x77,7,16)\n",
        );
        let index = write(
            dir.path(),
            "any.endm",
            "!gamemap\n!game_files\n18 game.endm # selected version\n",
        );
        let result = load_truth_maps(18, TruthScriptKind::Ending, &[index.clone()]).unwrap();
        assert_eq!(
            result.instructions[0].params.as_ref().unwrap()[0].type_name,
            "string"
        );
        assert!(
            load_truth_maps(17, TruthScriptKind::Ending, &[index.clone()])
                .unwrap_err()
                .contains("17")
        );
        assert!(load_truth_maps(18, TruthScriptKind::Msg, &[index])
            .unwrap_err()
            .contains("kind"));
    }
    #[test]
    fn later_alias_reassignment_does_not_leave_a_wrong_completion() {
        let dir = tempfile::tempdir().unwrap();
        let first = write(
            dir.path(),
            "a.stdm",
            "!stdmap\n!ins_names\n1 oldName\n1 sharedName\n!ins_signatures\n1 S\n",
        );
        let second = write(
            dir.path(),
            "b.stdm",
            "!stdmap\n!ins_names\n2 sharedName\n!ins_signatures\n2 ff\n",
        );
        let result = load_truth_maps(20, TruthScriptKind::Std, &[first, second]).unwrap();
        assert_eq!(result.instructions[0].name.as_deref(), Some("oldName"));
        assert_eq!(result.instructions[0].aliases, ["oldName"]);
        assert_eq!(result.instructions[1].aliases, ["sharedName"]);
    }

    #[test]
    fn loads_actual_zero318_maps_with_user_parameter_shapes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/maps/truth/zero318");
        let std = load_truth_maps(20, TruthScriptKind::Std, &[root.join("th20.stdm")]).unwrap();
        let camera = std.instructions.iter().find(|i| i.opcode == 3).unwrap();
        assert_eq!(camera.name.as_deref(), Some("camera_position_interp"));
        assert_eq!(camera.params.as_ref().unwrap().len(), 5);
        assert_eq!(std.enums["InterpMode"]["Linear"], 0);
        assert_eq!(
            std.instructions
                .iter()
                .find(|i| i.opcode == 1)
                .unwrap()
                .intrinsic
                .as_deref(),
            Some("Jmp()")
        );
        let msg = load_truth_maps(20, TruthScriptKind::Msg, &[root.join("th20.msgm")]).unwrap();
        let text = msg.instructions.iter().find(|i| i.opcode == 17).unwrap();
        assert_eq!(text.params.as_ref().unwrap().len(), 1);
        assert_eq!(text.params.as_ref().unwrap()[0].type_name, "string");
        let end = load_truth_maps(20, TruthScriptKind::Ending, &[root.join("any.endm")]).unwrap();
        assert_eq!(
            end.instructions
                .iter()
                .find(|i| i.opcode == 9)
                .unwrap()
                .params
                .as_ref()
                .unwrap()
                .len(),
            2
        );
    }
}
