use super::{compiler::resolve_script_context, maps::TruthScriptKind};
use crate::{app_state::AppState, config::AppConfig};
use serde::Serialize;
use std::{fs, path::Path};

include!(concat!(env!("OUT_DIR"), "/agent_references.rs"));
const SKILL: &str = include_str!("../../../assets/truth-skill-template.md");
const RULES: &str = "## MSG、END、STD 脚本编程\n\n阅读 `.claude/skills/touhou-truth-scripts/SKILL.md` 和 `.claude/skills/touhou-truth-scripts/references/README.md`。生成时的默认版本、默认编码与工具位置见 `.claude/skills/touhou-truth-scripts/references/PROJECT.md`。版本和二进制文本编码仅供参考，应按具体文件、目标游戏或补丁及当前任务选择；源码保持 UTF-8。完成一组相关修改后，使用该指南中的 CLI 或 Studio MCP 检查。\n";

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

/// Copy the maintained manuals and record default tool configuration for reference.
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
    let mut project = format!(
        "# 项目配置参考\n\n以下为生成时的默认配置快照。版本和二进制文本编码仅供参考，请按具体文件、目标游戏或补丁及当前任务选择；不要为迁就默认值改写脚本或转换编码。源码始终 UTF-8。\n\n人工指令参考：[MSG](msg-instructions.md)、[END](ending-instructions.md)、[STD](std-instructions.md)，按各自版本章节阅读。项目自定义 map 的签名可通过现有 MCP 查询。\n\n检查器：{}。也可通过 `THTK_CHECK` 指定位置，命令见 [入口](README.md#版本编码与检查)。\n\n",
        checker.as_ref().map(|p| format!("`{}`", p.display()))
            .unwrap_or_else(|| "尚未安装；可使用 Studio MCP 的 run_script(check)".into())
    );
    if let Some(checker) = checker {
        project.push_str("## 外部终端首次使用\n\nStudio 新建终端会自动设置 THTK_CHECK。外部终端在当前会话执行以下命令，再把示例源码路径换成要检查的文件；--help 用于确认检查器可执行。\n\n");
        if cfg!(windows) {
            let path = checker.to_string_lossy().replace('\'', "''");
            project.push_str(&format!(
                "PowerShell：\n\n```powershell\n$env:THTK_CHECK = '{path}'\n& $env:THTK_CHECK --help\n& $env:THTK_CHECK check scripts/dialogue.msgspec --json\n```\n\n"
            ));
        } else {
            let path = checker.to_string_lossy().replace('\'', "'\\''");
            project.push_str(&format!(
                "Bash / sh：\n\n```sh\nexport THTK_CHECK='{path}'\n\"$THTK_CHECK\" --help\n\"$THTK_CHECK\" check scripts/dialogue.msgspec --json\n```\n\n"
            ));
        }
        project.push_str("在项目根目录运行示例，或使用源码绝对路径。变量只在当前终端会话生效；也可直接用带引号的检查器完整路径代替变量。目标版本和编码按本次任务指定，见 [检查用法](README.md#版本编码与检查)。\n\n");
    }
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
                project.push_str(&format!("## {}\n\n默认游戏版本（仅供参考）：th{}。默认二进制文本编码（可按文件和任务选择）：{}。工具：`{}`{}。\n\n", kind.as_str(), context.game_version,
                    context.encoding.as_deref().unwrap_or("不适用"), context.tool_path.display(),
                    if context.tool_path.is_file() { "" } else { "（当前文件不存在）" }));
                project.push_str("生成时默认 map（按应用顺序）：\n\n");
                for path in &context.map_paths {
                    project.push_str(&format!("- `{}`\n", path.display()));
                }
                project.push('\n');
            }
            Err(error) => project.push_str(&format!(
                "## {}\n\n默认工具配置当前不可用：{error}\n\n",
                kind.as_str()
            )),
        }
    }
    project.push_str("\n本辅助包的 references 由 Studio 导出，重新生成时刷新；自定义约定写入 SKILL.md 或项目根 AGENTS.md。\n");
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
    fn first_generation_exports_manuals_and_project_defaults() {
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
        assert_eq!(
            fs::read_to_string(refs.join("msg-instructions.md")).unwrap(),
            include_str!("../../../../docs/agent/msg-instructions.md")
        );
        assert!(!refs.join("instructions").exists());
        assert!(!refs.join("sources.md").exists());
        assert!(!refs.join("tooling.md").exists());
        let context = fs::read_to_string(refs.join("PROJECT.md")).unwrap();
        assert!(context.contains("th18"));
        assert!(context.contains("gbk"));
        assert!(context.contains("ending-instructions.md"));
        assert!(context.contains("project.msgm"));
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
        assert!(refs.join("ending-instructions.md").is_file());
        let context = fs::read_to_string(refs.join("PROJECT.md")).unwrap();
        assert!(context.contains("msg-instructions.md"));
        assert!(context.contains("std-instructions.md"));
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
