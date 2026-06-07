use regex::{Match, Regex};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Diagnostic {
    pub path: String,
    pub line: u32,
    pub column: Option<u32>, // thecl 可能不提供列号
    pub severity: String,    // "error" | "warning"
    pub message: String,
}

/// 解析 thecl 的标准错误输出
pub fn parse_thecl_output(stderr: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let explicit_severity_re = Regex::new(r"^(error|warning):\s*(.+)$").unwrap();
    let location_re =
        Regex::new(r"^(?P<path>.+?):(?P<line>\d+)(?:[: ,](?P<column>\d+))?$").unwrap();

    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let without_tool_prefix = trimmed
            .strip_prefix("thecl.exe:")
            .or_else(|| trimmed.strip_prefix("thecl:"))
            .unwrap_or(trimmed);

        let Some((location_part, diagnostic_part)) = without_tool_prefix.rsplit_once(':') else {
            continue;
        };

        let diagnostic_text = diagnostic_part.trim();
        let (severity, message, location) =
            if let Some(caps) = explicit_severity_re.captures(diagnostic_text) {
                let severity = caps
                    .get(1)
                    .map(|m: Match| m.as_str().to_string())
                    .unwrap_or_else(|| "error".to_string());
                let message = caps
                    .get(2)
                    .map(|m: Match| m.as_str().trim().to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                (severity, message, location_part.trim().to_string())
            } else {
                (
                    infer_severity_from_message(diagnostic_text),
                    diagnostic_text.to_string(),
                    location_part.trim().to_string()
                )
            };

        if let Some(diagnostic) = parse_location(&location, &location_re, severity, message) {
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn parse_location(
    location: &str,
    location_re: &Regex,
    severity: String,
    message: String,
) -> Option<Diagnostic> {
    let caps = location_re.captures(location)?;
    let path = caps.name("path")?.as_str().trim().to_string();
    let line = caps.name("line")?.as_str().trim().parse::<u32>().ok()?;
    let column = caps
        .name("column")
        .and_then(|m| m.as_str().trim().parse::<u32>().ok());

    Some(Diagnostic {
        path,
        line,
        column,
        severity,
        message,
    })
}

fn infer_severity_from_message(message: &str) -> String {
    let lower = message.trim().to_lowercase();
    if lower.starts_with("warning") {
        "warning".to_string()
    } else {
        "error".to_string()
    }
}
