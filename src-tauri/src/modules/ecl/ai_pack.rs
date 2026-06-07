use super::map_parser::EclMapSemanticData;
use serde::Serialize;
use std::fs;
use std::path::Path;

const SKILL_TEMPLATE: &str = include_str!("../../../assets/ecl-skill-template.md");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPackResult {
    pub skill_path: String,
    pub skill_written: bool,
    pub reference_files: Vec<String>,
    pub version: String,
}

fn render_instructions_markdown(data: &EclMapSemanticData) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# th{} ECL 指令参考\n\n由 THTK-Studio 从 eclmap 自动生成，请勿手工编辑（会被刷新覆盖）。\n\n",
        data.version
    ));
    out.push_str("| opcode | 指令名 | 签名 | 参数 | 分组 |\n|---|---|---|---|---|\n");
    for ins in &data.instructions {
        let params = ins
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.type_name))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            ins.opcode,
            ins.name,
            ins.signature.as_deref().unwrap_or(""),
            params,
            ins.section.as_deref().unwrap_or("")
        ));
    }
    out
}

fn render_registers_markdown(data: &EclMapSemanticData) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# th{} 全局寄存器参考\n\n由 THTK-Studio 从 eclmap 自动生成，请勿手工编辑。\n\n",
        data.version
    ));
    out.push_str("| id | 名称 | 类型 |\n|---|---|---|\n");
    for global in &data.globals {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            global.id, global.name, global.var_type
        ));
    }
    out
}

/// 在项目根生成 .claude/skills/ecl-modding/：
/// SKILL.md 仅在缺失时写入（保护用户修改）；references/ 总是刷新。
pub fn generate(project_root: &str, data: &EclMapSemanticData) -> Result<AiPackResult, String> {
    let skill_dir = Path::new(project_root).join(".claude/skills/ecl-modding");
    let references_dir = skill_dir.join("references");
    fs::create_dir_all(&references_dir)
        .map_err(|e| format!("Failed to create skill directories: {e}"))?;

    let instructions_file = format!("th{}-instructions.md", data.version);
    let registers_file = format!("th{}-registers.md", data.version);

    let mut reference_files = Vec::new();

    let instructions_path = references_dir.join(&instructions_file);
    fs::write(&instructions_path, render_instructions_markdown(data))
        .map_err(|e| format!("Failed to write instructions reference: {e}"))?;
    reference_files.push(instructions_path.to_string_lossy().to_string());

    let registers_path = references_dir.join(&registers_file);
    fs::write(&registers_path, render_registers_markdown(data))
        .map_err(|e| format!("Failed to write registers reference: {e}"))?;
    reference_files.push(registers_path.to_string_lossy().to_string());

    let skill_path = skill_dir.join("SKILL.md");
    let skill_written = if skill_path.exists() {
        false
    } else {
        let content = SKILL_TEMPLATE
            .replace("{{VERSION}}", &data.version)
            .replace("{{INSTRUCTIONS_FILE}}", &instructions_file)
            .replace("{{REGISTERS_FILE}}", &registers_file);
        fs::write(&skill_path, content).map_err(|e| format!("Failed to write SKILL.md: {e}"))?;
        true
    };

    Ok(AiPackResult {
        skill_path: skill_path.to_string_lossy().to_string(),
        skill_written,
        reference_files,
        version: data.version.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ecl::map_parser;

    const SAMPLE_MAP: &str = "!eclmap\n!ins_names\n10 jump\n23 wait\n!ins_signatures\n10 ot\n23 S\n!gvar_names\n-9985 I0\n!gvar_types\n-9985 $\n";

    fn sample_semantics() -> map_parser::EclMapSemanticData {
        map_parser::parse_ecl_map_content("maps/th17.eclm", SAMPLE_MAP).expect("parse")
    }

    #[test]
    fn generates_skill_and_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();

        let result = generate(&root, &sample_semantics()).expect("generate");

        assert!(result.skill_written);
        let skill = fs::read_to_string(dir.path().join(".claude/skills/ecl-modding/SKILL.md"))
            .expect("skill exists");
        assert!(skill.contains("th17"), "version substituted");
        assert!(!skill.contains("{{VERSION}}"), "no placeholder left");

        let instructions = fs::read_to_string(
            dir.path()
                .join(".claude/skills/ecl-modding/references/th17-instructions.md"),
        )
        .expect("instructions exist");
        assert!(instructions.contains("wait"));
        assert!(instructions.contains("| 23 |"));

        let registers = fs::read_to_string(
            dir.path()
                .join(".claude/skills/ecl-modding/references/th17-registers.md"),
        )
        .expect("registers exist");
        assert!(registers.contains("I0"));
    }

    #[test]
    fn rerun_preserves_user_skill_but_refreshes_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_string_lossy().to_string();
        generate(&root, &sample_semantics()).expect("first run");

        let skill_path = dir.path().join(".claude/skills/ecl-modding/SKILL.md");
        fs::write(&skill_path, "USER EDITED").expect("user edit");
        let ref_path = dir
            .path()
            .join(".claude/skills/ecl-modding/references/th17-instructions.md");
        fs::write(&ref_path, "STALE").expect("stale ref");

        let result = generate(&root, &sample_semantics()).expect("second run");

        assert!(!result.skill_written, "must not overwrite user skill");
        assert_eq!(fs::read_to_string(&skill_path).unwrap(), "USER EDITED");
        assert!(
            fs::read_to_string(&ref_path).unwrap().contains("wait"),
            "references must be regenerated"
        );
    }
}
