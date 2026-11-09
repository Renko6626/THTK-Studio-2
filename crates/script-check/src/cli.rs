use script_check::*;
use std::{
    io::{self, Read},
    path::PathBuf,
};
const HELP: &str = "thtk-check check SOURCE [--project ROOT] [--kind msg|ending|std]\n    [--game VERSION] [--encoding ENCODING] [--truth-dir DIR]\n    [--map PATH ...] [--stdin] [--json]";

pub fn main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return 0;
    }
    let input_path = args.get(1).cloned().unwrap_or_default();
    match check(args) {
        Ok((context, result, code)) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"context": context, "result": result})
                );
            } else {
                println!("{}", result.message);
            }
            code
        }
        Err(error) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"context": null, "result": failure(None, input_path, error)})
                );
            } else {
                eprintln!("{error}");
            }
            2
        }
    }
}
fn failure(context: Option<&ScriptContext>, input_path: String, message: String) -> ScriptResult {
    ScriptResult {
        success: false,
        tool: context.map(|c| c.kind.tool()).unwrap_or("").into(),
        mode: "check".into(),
        script_kind: context.map(|c| c.kind.as_str()).unwrap_or("").into(),
        input_path,
        output_path: None,
        message,
        diagnostics: vec![],
    }
}
fn check(args: Vec<String>) -> Result<(ScriptContext, ScriptResult, i32), CheckError> {
    if args.first().map(String::as_str) != Some("check") {
        return Err(HELP.into());
    }
    let source = args.get(1).filter(|s| !s.starts_with('-')).ok_or(HELP)?;
    let input_path = std::path::absolute(source).map_err(|e| e.to_string())?;
    let mut project_root = None;
    let mut kind = None;
    let mut game = None;
    let mut encoding = None;
    let mut truth_dir = None;
    let mut maps = Vec::new();
    let mut stdin = false;
    let mut index = 2;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--stdin" {
            stdin = true;
            index += 1;
            continue;
        }
        if flag == "--json" {
            index += 1;
            continue;
        }
        if ![
            "--project",
            "--kind",
            "--game",
            "--encoding",
            "--truth-dir",
            "--map",
        ]
        .contains(&flag.as_str())
        {
            return Err(format!("Unknown option: {flag}"));
        }
        index += 1;
        let value = args
            .get(index)
            .filter(|s| !s.starts_with("--"))
            .ok_or_else(|| format!("Missing value for {flag}"))?;
        match flag.as_str() {
            "--project" => {
                project_root = Some(std::path::absolute(value).map_err(|e| e.to_string())?)
            }
            "--kind" => {
                kind = Some(match value.as_str() {
                    "msg" => ScriptKind::Msg,
                    "ending" => ScriptKind::Ending,
                    "std" => ScriptKind::Std,
                    _ => return Err(format!("Unknown kind: {value}")),
                })
            }
            "--game" => game = Some(value.clone()),
            "--encoding" => encoding = Some(value.clone()),
            "--truth-dir" => truth_dir = Some(PathBuf::from(value)),
            "--map" => maps.push(PathBuf::from(value)),
            _ => unreachable!(),
        }
        index += 1;
    }
    let kind = kind
        .or_else(|| {
            match input_path
                .extension()?
                .to_str()?
                .to_ascii_lowercase()
                .as_str()
            {
                "msgspec" => Some(ScriptKind::Msg),
                "endspec" => Some(ScriptKind::Ending),
                "stdspec" => Some(ScriptKind::Std),
                _ => None,
            }
        })
        .ok_or("Cannot infer kind; use --kind msg|ending|std")?;
    let explicit_project = project_root.is_some();
    let project_root = match project_root {
        Some(p) => Some(p),
        None => discover_project_root(&input_path)?,
    };
    let project = project_root
        .as_deref()
        .map(load_project_snapshot)
        .transpose()?
        .flatten();
    if explicit_project && project.is_none() {
        return Err(format!("Missing {PROJECT_FILE} in --project root"));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let bundled_truth_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .ok_or("Cannot locate bundled truth directory")?
        .join("truth");
    let context = resolve_context(&ContextInput {
        kind,
        input_path: input_path.clone(),
        project_root,
        explicit_game_version: game,
        explicit_encoding: encoding,
        explicit_truth_dir: truth_dir,
        explicit_map_paths: maps,
        app: load_app_snapshot()?,
        project,
        bundled_truth_dir,
    })?;
    let source_text = if stdin {
        let mut s = String::new();
        io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| e.to_string())?;
        Some(s)
    } else {
        None
    };
    let attempt = run(
        &context,
        &ScriptRequest {
            operation: ScriptOperation::Check,
            input_path: input_path.clone(),
            output_path: None,
            source_text,
        },
    );
    let (result, code) = match attempt {
        Ok(result) => {
            let code = if result.success { 0 } else { 1 };
            (result, code)
        }
        Err(error) => (
            failure(
                Some(&context),
                input_path.to_string_lossy().into_owned(),
                error,
            ),
            2,
        ),
    };
    Ok((context, result, code))
}
