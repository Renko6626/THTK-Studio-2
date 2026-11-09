use crate::modules::ecl::error_parser::Diagnostic;
use std::path::Path;

pub fn parse(stderr: &str, cwd: &Path) -> Vec<Diagnostic> {
    let position = regex::Regex::new(r"(?:┌─|-->)[ \t]+(.+):(\d+):(\d+)\s*$").unwrap();
    let mut current: Option<(String, String)> = None;
    let mut out = Vec::new();
    for line in stderr.lines() {
        if let Some(message) = line.strip_prefix("error:") {
            current = Some(("error".into(), message.trim().into()));
        }
        if let Some(message) = line.strip_prefix("warning:") {
            current = Some(("warning".into(), message.trim().into()));
        }
        if let (Some(c), Some((severity, message))) = (position.captures(line), current.as_ref()) {
            let path = Path::new(c[1].trim());
            out.push(Diagnostic {
                path: if path.is_absolute() {
                    path.into()
                } else {
                    cwd.join(path)
                }
                .to_string_lossy()
                .into_owned(),
                line: c[2].parse().unwrap_or(1),
                column: c[3].parse().ok(),
                severity: severity.clone(),
                message: message.clone(),
            });
            current = None;
        }
    }
    out
}
