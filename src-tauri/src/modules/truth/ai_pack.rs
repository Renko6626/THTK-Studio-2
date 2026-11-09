use crate::app_state::AppState;
#[tauri::command]
pub fn generate_truth_assist_pack(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or("请先打开项目")?;
    let dir = std::path::Path::new(&root).join(".claude/skills/touhou-truth-scripts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("SKILL.md");
    if !path.exists() {
        std::fs::write(
            &path,
            include_str!("../../../assets/truth-skill-template.md"),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().into_owned())
}
