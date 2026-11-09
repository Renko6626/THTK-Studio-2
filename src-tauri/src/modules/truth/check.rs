//! Checks editor buffers through the same compiler used for builds.
use super::{
    compiler::{self, ScriptMode, ScriptRequest, ScriptResult},
    maps::TruthScriptKind,
};
use crate::{
    common::{project_config::ProjectConfig, toolchain},
    config::AppConfig,
};
use std::{io::Write, path::Path};

pub fn check(
    config: &AppConfig,
    root: Option<&str>,
    project: Option<&ProjectConfig>,
    input_path: &str,
    kind: TruthScriptKind,
    text: &str,
) -> Result<ScriptResult, String> {
    let original = std::path::absolute(input_path).map_err(|e| e.to_string())?;
    let parent = original.parent().ok_or("源码路径没有父目录")?;
    let id = uuid::Uuid::new_v4();
    let source = parent.join(format!(".thtk-check-{id}.{}", kind.extension()));
    let output = std::env::temp_dir().join(format!("thtk-check-{id}.bin"));
    let attempt = (|| {
        let mut file =
            std::fs::File::create_new(&source).map_err(|e| format!("无法创建检查临时文件：{e}"))?;
        file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        drop(file);
        let mut result = compiler::run(
            config,
            root,
            project,
            &ScriptRequest {
                mode: ScriptMode::Compile,
                kind,
                input_path: source.to_string_lossy().into_owned(),
                output_path: Some(output.to_string_lossy().into_owned()),
                encoding: None,
            },
        );
        for diagnostic in &mut result.diagnostics {
            if Path::new(&diagnostic.path) == source
                || Path::new(&diagnostic.path).canonicalize().ok() == source.canonicalize().ok()
            {
                diagnostic.path = input_path.into();
            }
        }
        result.message = result
            .message
            .replace(source.to_string_lossy().as_ref(), input_path);
        result.input_path = input_path.into();
        result.output_path = None;
        result.mode = "check".into();
        Ok(result)
    })();
    for path in [&source, &output] {
        match std::fs::remove_file(path) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("清理检查文件失败：{e}")),
        }
    }
    attempt
}

#[tauri::command]
pub async fn check_truth_source(
    state: tauri::State<'_, crate::app_state::AppState>,
    input_path: String,
    script_kind: TruthScriptKind,
    source_text: String,
) -> Result<ScriptResult, String> {
    let root = state
        .current_project_root
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ctx = toolchain::effective_context(&state.config_manager.get_config(), root.as_deref());
    tauri::async_runtime::spawn_blocking(move || {
        check(
            &ctx.config,
            root.as_deref(),
            ctx.project.as_ref(),
            &input_path,
            script_kind,
            &source_text,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires prepared host truth tools"]
    fn buffer_check_preserves_file_maps_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dialog.msgspec");
        std::fs::write(&path, "original unsaved file").unwrap();
        std::fs::write(
            dir.path().join("custom.msgm"),
            "!msgmap\n!ins_names\n17 say\n",
        )
        .unwrap();
        let pc = ProjectConfig {
            game_version: "18".into(),
            encoding: "gbk".into(),
            ..Default::default()
        };
        let text="#pragma mapfile \"custom.msgm\"\nmeta {table:{0:{script:\"main\"}}}\nscript main {\n say(\"汉\");\n}";
        let good = check(
            &AppConfig::default(),
            None,
            Some(&pc),
            path.to_str().unwrap(),
            TruthScriptKind::Msg,
            text,
        )
        .unwrap();
        assert!(good.success, "{}", good.message);
        assert!(good.output_path.is_none());
        let bad = check(
            &AppConfig::default(),
            None,
            Some(&pc),
            path.to_str().unwrap(),
            TruthScriptKind::Msg,
            &text.replace("say(\"汉\")", "say()"),
        )
        .unwrap();
        assert!(!bad.success);
        assert_eq!(bad.diagnostics[0].path, path.to_str().unwrap());
        assert_eq!(bad.diagnostics[0].line, 4);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "original unsaved file"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
