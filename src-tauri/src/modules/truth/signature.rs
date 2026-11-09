//! Parses the map signature surface for editor hints, without implementing truth's
//! encoding or type checker. Attribute values stay in their original text form.

use super::maps::TruthParameter;
use std::collections::BTreeMap;

/// Finds a separator outside quoted strings (including escaped quotes).
fn find_unquoted(text: &str, separator: char) -> Option<usize> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
            continue;
        }
        if !quoted && ch == separator {
            return Some(index);
        }
    }
    None
}

fn parse_attributes(mut text: &str) -> Result<BTreeMap<String, Option<String>>, String> {
    let mut attributes = BTreeMap::new();
    while !text.trim().is_empty() {
        let end = find_unquoted(text, ';').unwrap_or(text.len());
        let part = text[..end].trim();
        let (name, value) = match find_unquoted(part, '=') {
            Some(index) => (
                part[..index].trim(),
                Some(part[index + 1..].trim().to_string()),
            ),
            None => (part, None),
        };
        if name.is_empty() || value.as_ref().is_some_and(|v| v.is_empty()) {
            return Err(format!("invalid signature attribute {part:?}"));
        }
        if attributes.insert(name.to_string(), value).is_some() {
            return Err(format!("duplicate signature attribute {name:?}"));
        }
        text = if end == text.len() {
            ""
        } else {
            &text[end + 1..]
        };
    }
    Ok(attributes)
}

pub fn parse_truth_signature(mut signature: &str) -> Result<Vec<TruthParameter>, String> {
    let mut params = Vec::new();
    while !signature.trim_start().is_empty() {
        signature = signature.trim_start();
        let code = signature.chars().next().unwrap();
        signature = &signature[code.len_utf8()..];
        signature = signature.trim_start();
        let attributes = if let Some(rest) = signature.strip_prefix('(') {
            let end = find_unquoted(rest, ')')
                .ok_or_else(|| "unclosed signature attributes".to_string())?;
            let attributes = parse_attributes(&rest[..end])?;
            signature = &rest[end + 1..];
            attributes
        } else {
            BTreeMap::new()
        };
        let type_name = match code {
            '_' | '-' => continue,
            'S' | 's' | 'c' | 'U' | 'u' | 'b' | 'n' | 'N' | 'E' | 'C' => "int",
            'f' => "float",
            'o' => "offset",
            't' => "time",
            'z' | 'm' | 'p' | 'P' => "string",
            _ => return Err(format!("unsupported truth signature type {code:?}")),
        };
        let enum_name = match attributes.get("enum") {
            Some(Some(value)) => Some(
                serde_json::from_str::<String>(value)
                    .map_err(|e| format!("invalid enum name {value:?}: {e}"))?,
            ),
            Some(None) => return Err("enum attribute requires a name".into()),
            None => match code {
                'n' => Some("AnmSprite".into()),
                'N' => Some("AnmScript".into()),
                'E' => Some("EclSub".into()),
                _ => None,
            },
        };
        params.push(TruthParameter {
            type_name: type_name.into(),
            encoding: code.to_string(),
            enum_name,
            attributes,
        });
    }
    Ok(params)
}
