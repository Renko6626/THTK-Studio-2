//! Structural source outline, not a VM simulation. Nested control flow has unknown time.
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEvent {
    pub script: String,
    pub line: u32,
    pub time: Option<i64>,
    pub name: String,
    pub kind: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelLocation {
    pub script: String,
    pub name: String,
    pub line: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptJump {
    pub line: u32,
    pub column: u32,
    pub end_column: u32,
    pub target_line: Option<u32>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptAnalysis {
    pub events: Vec<ScriptEvent>,
    pub labels: Vec<LabelLocation>,
    pub jumps: Vec<ScriptJump>,
}

pub fn analyze(text: &str) -> ScriptAnalysis {
    let clean = super::source::without_comments(text);
    // Strings cannot introduce labels, scripts, calls or braces.
    let mut code = clean.as_bytes().to_vec();
    let (mut quote, mut escape) = (false, false);
    for byte in &mut code {
        let original = *byte;
        if quote {
            if original != b'\n' {
                *byte = b' ';
            }
            if escape {
                escape = false;
            } else if original == b'\\' {
                escape = true;
            } else if original == b'"' {
                quote = false;
            }
        } else if original == b'"' {
            quote = true;
            *byte = b' ';
        }
    }
    let code = String::from_utf8(code).unwrap();
    let script_re = regex::Regex::new(r"\bscript\s+(?:\d+\s+)?([A-Za-z_]\w*)\s*(?:\{|$)").unwrap();
    let time_re = regex::Regex::new(r"^\s*([+]?[-]?\d+)\s*:").unwrap();
    let label_re = regex::Regex::new(r"^\s*([A-Za-z_]\w*)\s*:").unwrap();
    let call_re = regex::Regex::new(r"\b([A-Za-z_]\w*)\s*\(").unwrap();
    let jump_re = regex::Regex::new(r"\bgoto\s+([A-Za-z_]\w*)").unwrap();
    let control_re = regex::Regex::new(r"\b(loop|while|if|else|times)\b").unwrap();
    let (mut events, mut labels, mut pending) = (Vec::new(), Vec::new(), Vec::new());
    let mut opened = false;
    let (mut script, mut depth, mut time, mut uncertain) = (String::new(), 0i32, Some(0i64), false);
    for (index, line) in code.lines().enumerate() {
        let number = index as u32 + 1;
        if let Some(c) = script_re.captures(line) {
            script = c[1].into();
            opened = false;
            depth = 0;
            time = Some(0);
            uncertain = false;
        }
        if script.is_empty() {
            continue;
        }
        let control = control_re.captures(line);
        if let Some(c) = control.as_ref() {
            events.push(ScriptEvent {
                script: script.clone(),
                line: number,
                time: None,
                name: c[1].into(),
                kind: "control".into(),
            });
            uncertain = true;
        }
        if let Some(c) = time_re.captures(line) {
            let v = c[1].parse::<i64>().ok();
            time = if c[1].starts_with('+') {
                time.and_then(|t| v.and_then(|v| t.checked_add(v)))
            } else {
                v
            };
        }
        if let Some(c) = label_re.captures(line) {
            labels.push(LabelLocation {
                script: script.clone(),
                name: c[1].into(),
                line: number,
            });
        }
        for c in call_re.captures_iter(line) {
            if ["if", "while", "times"].contains(&&c[1]) {
                continue;
            }
            events.push(ScriptEvent {
                script: script.clone(),
                line: number,
                time: if uncertain || depth > 1 { None } else { time },
                name: c[1].into(),
                kind: "instruction".into(),
            });
        }
        for c in jump_re.captures_iter(line) {
            let m = c.get(1).unwrap();
            pending.push((
                script.clone(),
                c[1].to_string(),
                number,
                line[..m.start()].encode_utf16().count() as u32 + 1,
                line[..m.end()].encode_utf16().count() as u32 + 1,
            ));
            uncertain = true;
        }
        opened |= line.contains('{');
        depth += line.bytes().filter(|b| *b == b'{').count() as i32
            - line.bytes().filter(|b| *b == b'}').count() as i32;
        if opened && depth <= 0 {
            script.clear();
        }
    }
    let jumps = pending
        .into_iter()
        .map(|(script, name, line, column, end_column)| ScriptJump {
            line,
            column,
            end_column,
            target_line: labels
                .iter()
                .find(|l| l.script == script && l.name == name)
                .map(|l| l.line),
        })
        .collect();
    ScriptAnalysis {
        events,
        labels,
        jumps,
    }
}
#[tauri::command]
pub async fn analyze_truth_source(source_text: String) -> Result<ScriptAnalysis, String> {
    tauri::async_runtime::spawn_blocking(move || analyze(&source_text))
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scripts_keep_labels_local_and_loops_do_not_claim_runtime_times() {
        let a=analyze("script a {\nstart:\nfoo();\n+30:\nloop {\nbar();\ngoto start;\n}\n}\nscript b {\nstart:\ngoto start;\n}\n");
        assert_eq!(a.jumps[0].target_line, Some(2));
        assert_eq!(a.jumps[1].target_line, Some(11));
        assert_eq!(a.events[0].time, Some(0));
        assert_eq!(
            a.events.iter().find(|e| e.name == "bar").unwrap().time,
            None
        );
    }
}
