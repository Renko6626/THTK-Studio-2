use super::{
    commands::source_definitions_for_context,
    compiler::resolve_script_context,
    maps::{TruthDefinitions, TruthScriptKind},
};
use crate::{app_state::AppState, config::AppConfig};
use serde::Serialize;
use std::{fs, path::Path};

include!(concat!(env!("OUT_DIR"), "/agent_references.rs"));
const SKILL: &str = include_str!("../../../assets/truth-skill-template.md");
const RULES: &str = "## MSG、END、STD 脚本编程\n\n阅读 `.claude/skills/touhou-truth-scripts/SKILL.md` 和 `.claude/skills/touhou-truth-scripts/references/README.md`。项目版本、编码与有效指令表见 `.claude/skills/touhou-truth-scripts/references/PROJECT.md`。源码保持 UTF-8；游戏文本编码沿用项目设置。完成一组相关修改后，使用该指南中的 CLI 或 Studio MCP 检查。\n";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruthAssistPackResult {
    pub skill_path: String,
    pub skill_written: bool,
    pub skill_existed: bool,
    pub reference_files: Vec<String>,
    pub project_rules_path: String,
    pub project_rules_written: bool,
    pub suggested_rules_text: String,
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', "<br>")
}

fn map_label(path: &str, root: &Path) -> String {
    let path = Path::new(path);
    if let Ok(relative) = path.strip_prefix(root) {
        relative.to_string_lossy().into_owned()
    } else {
        // Bundled maps are exported as names, never developer-machine dependencies.
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
}

fn render_instructions(data: &TruthDefinitions, root: &Path) -> String {
    let kind = data.script_kind.as_str();
    let mut out = format!("# th{} {kind} 指令\n\n此文件由 Studio 从版本基线与项目 map 生成，重新生成辅助包时覆盖。文件级 pragma 可另行改变定义；需要时用 `lookup_script_semantics` 查询。\n\n用法与行为：[{}](../{kind}.md)。\n\n| opcode | 名称／别名 | 签名 | 参数 | 说明 | 名称来源 | 签名来源 |\n|---|---|---|---|---|---|---|\n", data.game_version, kind);
    for i in &data.instructions {
        let params = i
            .params
            .as_ref()
            .map(|ps| {
                ps.iter()
                    .map(|p| {
                        let mut value = format!("{} [{}]", p.type_name, p.encoding);
                        if let Some(name) = &p.enum_name {
                            value.push_str(&format!(" enum={name}"));
                        }
                        for (name, attr) in &p.attributes {
                            value.push_str(&format!(
                                " {name}{}",
                                attr.as_ref().map(|v| format!("={v}")).unwrap_or_default()
                            ));
                        }
                        value
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "未知".into());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            i.opcode,
            cell(&i.aliases.join(", ")),
            cell(i.signature.as_deref().unwrap_or("未知")),
            cell(&params),
            cell(i.description.as_deref().unwrap_or("行为说明未收录")),
            cell(
                &i.name_source
                    .as_ref()
                    .map(|p| map_label(p, root))
                    .unwrap_or_default()
            ),
            cell(
                &i.signature_source
                    .as_ref()
                    .map(|p| map_label(p, root))
                    .unwrap_or_default()
            )
        ));
    }
    for (name, values) in &data.enums {
        out.push_str(&format!("\n## 枚举 {name}\n\n| 成员 | 值 |\n|---|---|\n"));
        for (name, value) in values {
            out.push_str(&format!("| {} | {value} |\n", cell(name)));
        }
    }
    out.push_str("\n## Map 来源\n\n");
    for source in &data.source_paths {
        out.push_str(&format!("- `{}`\n", map_label(source, root)));
    }
    out
}

/// Export offline docs plus full project definitions. Only generated references are refreshed.
pub fn generate(
    project_root: &str,
    config: &AppConfig,
    force: bool,
) -> Result<TruthAssistPackResult, String> {
    let root = Path::new(project_root);
    let dir = root.join(".claude/skills/touhou-truth-scripts");
    let references = dir.join("references");
    let mut files = Vec::new();
    let mut write_reference = |relative: &str, content: &[u8]| -> Result<(), String> {
        let path = references.join(relative);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
        files.push(path.to_string_lossy().into_owned());
        Ok(())
    };
    for (relative, content) in AGENT_REFERENCES {
        write_reference(relative, content)?;
    }
    let checker = crate::common::toolchain::bundled_checker_path().filter(|p| p.is_file());
    let mut project = format!("# 当前项目\n\n本目录的 references、examples、PROJECT.md 和 instructions 均由 Studio 生成，重新生成时刷新。自定义指导请写入 SKILL.md 或项目根 AGENTS.md。\n\n检查器：{}。可通过 `THTK_CHECK` 指定检查器位置。\n\n[编程入口](README.md)；[命令用法](tooling.md)；[示例](examples/manifest.json)。\n\n", checker.map(|p| format!("`{}`", p.display())).unwrap_or_else(|| "尚未安装；可使用 Studio MCP 的 run_script(check)".into()));
    for kind in [
        TruthScriptKind::Msg,
        TruthScriptKind::Ending,
        TruthScriptKind::Std,
    ] {
        let input = root.join(format!("untitled.{}", kind.extension()));
        match resolve_script_context(
            config,
            Some(project_root),
            None,
            kind,
            &input.to_string_lossy(),
            None,
        ) {
            Ok(context) => {
                let data =
                    source_definitions_for_context(&context, kind, &input.to_string_lossy(), "")?;
                let table = format!(
                    "instructions/{}-th{}.md",
                    kind.as_str(),
                    context.game_version
                );
                write_reference(&table, render_instructions(&data, root).as_bytes())?;
                project.push_str(&format!("## {}\n\n游戏版本：th{}。二进制编码：{}。工具：`{}`{}。\n\n[完整项目指令表]({table})\n\n", kind.as_str(), context.game_version,
                    context.encoding.as_deref().unwrap_or("不适用"), context.tool_path.display(),
                    if context.tool_path.is_file() { "" } else { "（当前文件不存在）" }));
                project.push_str("实际 map（按应用顺序）：\n\n");
                for path in &context.map_paths {
                    project.push_str(&format!("- `{}`\n", path.display()));
                }
                project.push('\n');
            }
            Err(error) => {
                project.push_str(&format!("## {}\n\n当前不可用：{error}\n\n", kind.as_str()))
            }
        }
    }
    write_reference("PROJECT.md", project.as_bytes())?;
    let skill_path = dir.join("SKILL.md");
    let skill_existed = skill_path.exists();
    let skill_written = !skill_existed || force;
    if skill_written {
        fs::write(&skill_path, SKILL).map_err(|e| e.to_string())?;
    }
    let rules_path = root.join("AGENTS.md");
    let project_rules_written = !rules_path.exists();
    if project_rules_written {
        fs::write(&rules_path, RULES).map_err(|e| e.to_string())?;
    }
    Ok(TruthAssistPackResult {
        skill_path: skill_path.to_string_lossy().into_owned(),
        skill_written,
        skill_existed,
        reference_files: files,
        project_rules_path: rules_path.to_string_lossy().into_owned(),
        project_rules_written,
        suggested_rules_text: RULES.into(),
    })
}

#[tauri::command]
pub async fn generate_truth_assist_pack(
    state: tauri::State<'_, AppState>,
    force: bool,
) -> Result<TruthAssistPackResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("请先打开项目")?;
    let config = state.config_manager.get_config();
    tauri::async_runtime::spawn_blocking(move || generate(&root, &config, force))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::project_config::{save_project_config, ProjectConfig, ScriptMaps};

    fn project(dir: &Path, game: &str) {
        fs::write(
            dir.join("project.msgm"),
            "!msgmap\n!ins_names\n17 projectText\n",
        )
        .unwrap();
        save_project_config(
            dir.to_str().unwrap(),
            &ProjectConfig {
                game_version: game.into(),
                encoding: "gbk".into(),
                script_maps: ScriptMaps {
                    msg: vec!["project.msgm".into()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn first_generation_exports_offline_tree_and_full_project_definitions() {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path(), "18");
        let result = generate(dir.path().to_str().unwrap(), &AppConfig::default(), false).unwrap();
        assert!(result.skill_written);
        assert!(!result.skill_existed);
        assert!(result.project_rules_written);
        let refs = Path::new(&result.skill_path)
            .parent()
            .unwrap()
            .join("references");
        for (relative, content) in AGENT_REFERENCES {
            assert_eq!(
                fs::read(refs.join(relative)).unwrap(),
                *content,
                "{relative}"
            );
        }
        let table = fs::read_to_string(refs.join("instructions/msg-th18.md")).unwrap();
        let data = super::super::commands::source_definitions(
            18,
            TruthScriptKind::Msg,
            Some(dir.path().to_str().unwrap()),
            Some(&ProjectConfig {
                script_maps: ScriptMaps {
                    msg: vec!["project.msgm".into()],
                    ..Default::default()
                },
                ..Default::default()
            }),
            dir.path().join("new.msgspec").to_str().unwrap(),
            "",
        )
        .unwrap();
        assert_eq!(table, render_instructions(&data, dir.path()));
        assert!(table.contains("projectText"));
        assert!(table.contains("project.msgm"));
        let context = fs::read_to_string(refs.join("PROJECT.md")).unwrap();
        assert!(context.contains("th18"));
        assert!(context.contains("gbk"));
        assert!(context.contains("instructions/ending-th18.md"));
        assert!(fs::read_to_string(&result.project_rules_path)
            .unwrap()
            .contains(".claude/skills/touhou-truth-scripts/SKILL.md"));
    }

    #[test]
    fn regeneration_refreshes_references_preserves_user_files_and_force_only_replaces_skill() {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path(), "8");
        let first = generate(dir.path().to_str().unwrap(), &AppConfig::default(), false).unwrap();
        let refs = Path::new(&first.skill_path)
            .parent()
            .unwrap()
            .join("references");
        fs::write(&first.skill_path, "user skill").unwrap();
        fs::write(&first.project_rules_path, "user rules").unwrap();
        fs::write(refs.join("msg.md"), "stale generated reference").unwrap();
        let again = generate(dir.path().to_str().unwrap(), &AppConfig::default(), false).unwrap();
        assert!(again.skill_existed);
        assert!(!again.skill_written);
        assert!(!again.project_rules_written);
        assert_eq!(fs::read_to_string(&first.skill_path).unwrap(), "user skill");
        assert_eq!(
            fs::read_to_string(&first.project_rules_path).unwrap(),
            "user rules"
        );
        let msg = AGENT_REFERENCES
            .iter()
            .find(|(p, _)| *p == "msg.md")
            .unwrap()
            .1;
        assert_eq!(fs::read(refs.join("msg.md")).unwrap(), msg);
        assert!(!refs.join("instructions/ending-th8.md").exists());
        let context = fs::read_to_string(refs.join("PROJECT.md")).unwrap();
        assert!(context.contains("instructions/msg-th8.md"));
        assert!(context.contains("instructions/std-th8.md"));
        assert!(context.contains("当前不可用"));
        let forced = generate(dir.path().to_str().unwrap(), &AppConfig::default(), true).unwrap();
        assert!(forced.skill_written);
        assert_eq!(fs::read_to_string(&first.skill_path).unwrap(), SKILL);
        assert_eq!(
            fs::read_to_string(&first.project_rules_path).unwrap(),
            "user rules"
        );
    }
}
