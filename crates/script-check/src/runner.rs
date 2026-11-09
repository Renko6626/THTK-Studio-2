use crate::*;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub fn run(context: &ScriptContext, request: &ScriptRequest) -> Result<ScriptResult, CheckError> {
    if matches!(request.operation, ScriptOperation::Check) {
        return check(context, request);
    }
    if request.source_text.is_some() {
        return Err("源码文本只允许用于 check".into());
    }
    execute(context, request)
}
fn execute(context: &ScriptContext, request: &ScriptRequest) -> Result<ScriptResult, CheckError> {
    let input = std::path::absolute(&request.input_path).map_err(|e| e.to_string())?;
    if !input.is_file() {
        return Err(format!("输入文件不存在：{}", input.display()));
    }
    let cwd = input.parent().ok_or("Source has no parent directory")?;
    if matches!(request.operation, ScriptOperation::Compile)
        && !input
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(context.kind.extension()))
    {
        return Err(format!(
            "请使用 truth 原生 .{} 源码；旧 .dmsg/.dstd 已废弃",
            context.kind.extension()
        ));
    }
    let output = request
        .output_path
        .as_ref()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                cwd.join(p)
            }
        })
        .unwrap_or_else(|| {
            input.with_extension(match request.operation {
                ScriptOperation::Decompile => context.kind.extension(),
                _ if context.kind == ScriptKind::Std => "std",
                _ => "msg",
            })
        });
    if output == input
        || output
            .canonicalize()
            .ok()
            .zip(input.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
    {
        return Err("输出路径不能覆盖输入文件".into());
    }
    let mut cmd = Command::new(&context.tool_path);
    cmd.arg(request.operation.as_str())
        .arg(&input)
        .args(["-g", &context.game_version.to_string(), "-o"])
        .arg(&output)
        .current_dir(cwd)
        .env_remove("TRUTH_MAP_PATH");
    for map in &context.map_paths {
        cmd.arg("-m").arg(map);
    }
    if context.kind == ScriptKind::Ending {
        cmd.arg("--ending");
    }
    if let Some(encoding) = &context.encoding {
        cmd.args(["--encoding", encoding]);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let process = cmd
        .output()
        .map_err(|e| format!("无法启动 {}：{e}", context.kind.tool()))?;
    let stderr = String::from_utf8_lossy(&process.stderr);
    let stdout = String::from_utf8_lossy(&process.stdout);
    let success = process.status.success();
    let mut message = format!("{stdout}\n{stderr}").trim().to_string();
    if message.is_empty() {
        message = if success {
            "完成".into()
        } else {
            format!("{} 退出：{}", context.kind.tool(), process.status)
        };
    }
    Ok(ScriptResult {
        success,
        tool: context.kind.tool().into(),
        mode: request.operation.as_str().into(),
        script_kind: context.kind.as_str().into(),
        input_path: request.input_path.to_string_lossy().into_owned(),
        output_path: success.then(|| output.to_string_lossy().into_owned()),
        message,
        diagnostics: diagnostics::parse(&stderr, cwd),
    })
}
fn check(context: &ScriptContext, request: &ScriptRequest) -> Result<ScriptResult, CheckError> {
    let original = std::path::absolute(&request.input_path).map_err(|e| e.to_string())?;
    let parent = original.parent().ok_or("源码路径没有父目录")?;
    let id = uuid::Uuid::new_v4();
    let source = parent.join(format!(".thtk-check-{id}.{}", context.kind.extension()));
    let output = std::env::temp_dir().join(format!("thtk-check-{id}.bin"));
    let attempt = (|| {
        let input = if let Some(text) = &request.source_text {
            let mut file = std::fs::File::create_new(&source)
                .map_err(|e| format!("无法创建检查临时文件：{e}"))?;
            file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
            source.clone()
        } else {
            original.clone()
        };
        let mut result = execute(
            context,
            &ScriptRequest {
                operation: ScriptOperation::Compile,
                input_path: input.clone(),
                output_path: Some(output.clone()),
                source_text: None,
            },
        )?;
        if request.source_text.is_some() {
            for diagnostic in &mut result.diagnostics {
                if Path::new(&diagnostic.path) == source
                    || Path::new(&diagnostic.path)
                        .canonicalize()
                        .ok()
                        .zip(source.canonicalize().ok())
                        .is_some_and(|(a, b)| a == b)
                {
                    diagnostic.path = request.input_path.to_string_lossy().into_owned();
                }
            }
            result.message = result.message.replace(
                source.to_string_lossy().as_ref(),
                &request.input_path.to_string_lossy(),
            );
        }
        result.input_path = request.input_path.to_string_lossy().into_owned();
        result.output_path = None;
        result.mode = "check".into();
        Ok(result)
    })();
    let cleanup: Vec<PathBuf> = if request.source_text.is_some() {
        vec![source, output]
    } else {
        vec![output]
    };
    for path in cleanup {
        match std::fs::remove_file(path) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(format!("清理检查文件失败：{e}")),
        }
    }
    attempt
}
