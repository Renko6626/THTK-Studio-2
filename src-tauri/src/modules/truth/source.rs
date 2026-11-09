use std::path::{Path, PathBuf};

/// Blank comments while preserving byte offsets and newlines for source locations.
pub fn without_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = bytes.to_vec();
    let (mut i, mut quoted, mut escaped) = (0, false, false);
    while i < bytes.len() {
        if quoted {
            if escaped {
                escaped = false;
            } else if bytes[i] == b'\\' {
                escaped = true;
            } else if bytes[i] == b'"' {
                quoted = false;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            quoted = true;
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            out[i] = b' ';
            out[i + 1] = b' ';
            i += 2;
            while i < bytes.len() {
                if bytes[i..].starts_with(b"*/") {
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 2;
                    break;
                }
                if bytes[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).expect("comments replaced with ASCII spaces")
}

pub fn pragma_maps(text: &str, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let re = regex::Regex::new(r#"(?m)^\s*#pragma\s+mapfile\s+("(?:\\.|[^"\\])*")"#).unwrap();
    re.captures_iter(&without_comments(text))
        .map(|c| {
            let raw: String = serde_json::from_str(&c[1]).map_err(|e| e.to_string())?;
            let p = PathBuf::from(raw);
            Ok(if p.is_absolute() { p } else { cwd.join(p) })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_maps_follow_order_and_ignore_comments_without_damaging_strings() {
        let source="// #pragma mapfile \"ignored\"\n/* 中文\n#pragma mapfile \"also-ignored\" */\n#pragma mapfile \"first.msgm\"\n#pragma mapfile \"sub/last.msgm\"\nscript main { text(\"https://example/a/*b*/\"); }";
        let cwd = Path::new("project");
        assert_eq!(
            pragma_maps(source, cwd).unwrap(),
            [cwd.join("first.msgm"), cwd.join("sub/last.msgm")]
        );
        let clean = without_comments(source);
        assert_eq!(clean.len(), source.len());
        assert!(clean.contains("https://example/a/*b*/"));
        assert_eq!(clean.lines().count(), source.lines().count());
    }
}
