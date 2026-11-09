//! Source structure for the sidebar; time labels are retained, not executed.
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineNodeKind {
    Time,
    Instruction,
    Loop,
    Branch,
    Block,
    Interrupt,
    Label,
    Jump,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineNode {
    pub id: u32,
    pub kind: TimelineNodeKind,
    pub name: String,
    pub source: String,
    pub line: u32,
    pub end_line: u32,
    pub target_line: Option<u32>,
    pub children: Vec<TimelineNode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub name: String,
    pub line: u32,
    pub end_line: u32,
    pub nodes: Vec<TimelineNode>,
}

pub fn analyze(text: &str) -> Vec<TimelineEntry> {
    let code = super::source::code_mask(text);
    let mut parser = Parser::new(text, &code);
    let mut entries = Vec::new();
    while parser.pos < parser.tokens.len() {
        if !parser.at("script") {
            // Skip other top-level blocks (meta, functions), rather than finding fake scripts in them.
            if parser.at("{") {
                parser.skip_balanced("{", "}");
            } else {
                parser.pos += 1;
            }
            continue;
        }
        let start = parser.pos;
        parser.pos += 1;
        if parser
            .word()
            .is_some_and(|s| s.as_bytes()[0].is_ascii_digit())
        {
            parser.pos += 1;
        }
        let Some(name) = parser
            .word()
            .filter(|s| is_identifier(s))
            .map(str::to_owned)
        else {
            continue;
        };
        parser.pos += 1;
        if !parser.at("{") {
            continue;
        }
        parser.pos += 1;
        let (mut nodes, end_line) = parser.block();
        let mut initial = parser.node(TimelineNodeKind::Time, "0".into(), start, start + 1);
        initial.source.clear();
        // The implicit initial timestamp only groups instructions before the first explicit structure.
        let count = nodes
            .iter()
            .take_while(|n| n.kind == TimelineNodeKind::Instruction)
            .count();
        if count > 0 {
            initial.children = nodes.drain(..count).collect();
            initial.end_line = initial.children.last().unwrap().end_line;
            nodes.insert(0, initial);
        }
        resolve_targets(&mut nodes, end_line);
        entries.push(TimelineEntry {
            name,
            line: parser.tokens[start].line,
            end_line,
            nodes,
        });
    }
    entries
}

fn is_identifier(word: &str) -> bool {
    word.as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
}

#[derive(Clone, Copy)]
struct Token {
    start: usize,
    end: usize,
    line: u32,
}

struct Parser<'a> {
    text: &'a str,
    code: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    next_id: u32,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str, code: &'a str) -> Self {
        let (mut i, mut line, mut tokens) = (0, 1, Vec::new());
        while i < code.len() {
            let ch = code[i..].chars().next().unwrap();
            if ch.is_whitespace() {
                if ch == '\n' {
                    line += 1;
                }
                i += ch.len_utf8();
                continue;
            }
            let start = i;
            i += ch.len_utf8();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                while i < code.len()
                    && (code.as_bytes()[i].is_ascii_alphanumeric() || code.as_bytes()[i] == b'_')
                {
                    i += 1;
                }
            }
            tokens.push(Token {
                start,
                end: i,
                line,
            });
        }
        Self {
            text,
            code,
            tokens,
            pos: 0,
            next_id: 0,
        }
    }

    fn word_at(&self, pos: usize) -> Option<&'a str> {
        self.tokens.get(pos).map(|t| &self.code[t.start..t.end])
    }
    fn word(&self) -> Option<&'a str> {
        self.word_at(self.pos)
    }
    fn at(&self, word: &str) -> bool {
        self.word() == Some(word)
    }

    fn node(
        &mut self,
        kind: TimelineNodeKind,
        name: String,
        start: usize,
        end: usize,
    ) -> TimelineNode {
        let first = self.tokens[start];
        let last = self.tokens[end.saturating_sub(1).max(start)];
        let id = self.next_id;
        self.next_id += 1;
        TimelineNode {
            id,
            kind,
            name,
            source: self.text[first.start..last.end].trim().to_string(),
            line: first.line,
            end_line: last.line,
            target_line: None,
            children: Vec::new(),
        }
    }

    fn skip_balanced(&mut self, open: &str, close: &str) {
        let mut depth = 0;
        while let Some(word) = self.word() {
            if word == open {
                depth += 1;
            }
            if word == close {
                depth -= 1;
            }
            self.pos += 1;
            if depth == 0 {
                break;
            }
        }
    }

    fn block(&mut self) -> (Vec<TimelineNode>, u32) {
        let mut nodes: Vec<TimelineNode> = Vec::new();
        while self.pos < self.tokens.len() && !self.at("}") {
            // A new script during an unfinished edit should not be swallowed by the old entry.
            if self.at("script") {
                break;
            }
            let before = self.pos;
            if let Some(node) = self.statement() {
                match node.kind {
                    TimelineNodeKind::Instruction
                        if nodes
                            .last()
                            .is_some_and(|n| n.kind == TimelineNodeKind::Time) =>
                    {
                        let time = nodes.last_mut().unwrap();
                        time.end_line = node.end_line;
                        time.children.push(node);
                    }
                    TimelineNodeKind::Interrupt
                        if nodes
                            .last()
                            .is_some_and(|n| n.kind == TimelineNodeKind::Interrupt) =>
                    {
                        let previous = nodes.last_mut().unwrap();
                        previous.name.push_str(&format!(", {}", node.name));
                        previous.source.push_str(&format!("\n{}", node.source));
                        previous.end_line = node.end_line;
                    }
                    _ => nodes.push(node),
                }
            }
            if self.pos == before {
                self.pos += 1;
            }
        }
        let end_line = self
            .tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .map_or(1, |t| t.line);
        if self.at("}") {
            self.pos += 1;
        }
        (nodes, end_line)
    }

    fn statement(&mut self) -> Option<TimelineNode> {
        let start = self.pos;
        let word = self.word()?.to_string();
        if word == ";" {
            self.pos += 1;
            return None;
        }

        if word == "interrupt" && self.word_at(start + 1) == Some("[") {
            self.pos += 1;
            self.skip_balanced("[", "]");
            if self.at(":") {
                let name =
                    self.code[self.tokens[start].start..self.tokens[self.pos - 1].end].to_string();
                self.pos += 1;
                return Some(self.node(TimelineNodeKind::Interrupt, name, start, self.pos));
            }
            self.pos = start;
        }

        let number = if word == "+" || word == "-" {
            start + 1
        } else {
            start
        };
        if self
            .word_at(number)
            .is_some_and(|s| s.as_bytes()[0].is_ascii_digit())
            && self.word_at(number + 1) == Some(":")
        {
            let name = self.code[self.tokens[start].start..self.tokens[number].end].to_string();
            self.pos = number + 2;
            return Some(self.node(TimelineNodeKind::Time, name, start, self.pos));
        }
        if is_identifier(&word) && self.word_at(start + 1) == Some(":") {
            self.pos += 2;
            return Some(self.node(TimelineNodeKind::Label, word, start, self.pos));
        }

        if ["loop", "while", "do", "times", "if", "unless", "else", "{"].contains(&word.as_str()) {
            if word != "{" {
                self.pos += 1;
            }
            if word == "else" && self.at("if") {
                self.pos += 1;
            }
            if self.at("(") {
                self.skip_balanced("(", ")");
            }
            // An inline conditional jump is a jump, not a block with an invented extent.
            if self.at("goto") {
                return Some(self.jump(start));
            }
            let header_end = self.pos;
            let kind = if ["loop", "while", "do", "times"].contains(&word.as_str()) {
                TimelineNodeKind::Loop
            } else if word == "{" {
                TimelineNodeKind::Block
            } else {
                TimelineNodeKind::Branch
            };
            let (children, end_line) = if self.at("{") {
                self.pos += 1;
                self.block()
            } else {
                let child = self.statement();
                let end = child
                    .as_ref()
                    .map_or(self.tokens[start].line, |n| n.end_line);
                (child.into_iter().collect(), end)
            };
            let mut node = self.node(kind, word.clone(), start, header_end.max(start + 1));
            node.children = children;
            node.end_line = end_line;
            if node.kind == TimelineNodeKind::Loop {
                node.target_line = Some(node.line);
            }
            if word == "do" && self.at("while") {
                let suffix = self.pos;
                node.target_line = Some(self.tokens[suffix].line);
                self.pos += 1;
                if self.at("(") {
                    self.skip_balanced("(", ")");
                }
                if self.at(";") {
                    self.pos += 1;
                }
                node.source = format!(
                    "do {{ … }} {}",
                    &self.text[self.tokens[suffix].start..self.tokens[self.pos - 1].end]
                );
                node.end_line = self.tokens[self.pos - 1].line;
            }
            return Some(node);
        }

        if ["goto", "break", "continue", "return"].contains(&word.as_str()) {
            return Some(self.jump(start));
        }
        self.statement_end();
        if self.pos == start {
            return None;
        }
        let name = (start..self.pos)
            .find_map(|i| {
                let name = self.word_at(i)?;
                (is_identifier(name) && self.word_at(i + 1) == Some("(")).then(|| name.to_string())
            })
            .unwrap_or(word);
        Some(self.node(TimelineNodeKind::Instruction, name, start, self.pos))
    }

    fn statement_end(&mut self) {
        while let Some(word) = self.word() {
            if word == "}" || word == "script" {
                break;
            }
            if word == "(" {
                self.skip_balanced("(", ")");
            } else if word == "[" {
                self.skip_balanced("[", "]");
            } else {
                self.pos += 1;
                if word == ";" {
                    break;
                }
            }
        }
    }

    fn jump(&mut self, start: usize) -> TimelineNode {
        let name = if self.at("goto") {
            format!("goto {}", self.word_at(self.pos + 1).unwrap_or(""))
        } else {
            self.word().unwrap_or("").to_string()
        };
        self.statement_end();
        self.node(TimelineNodeKind::Jump, name, start, self.pos.max(start + 1))
    }
}

fn resolve_targets(nodes: &mut [TimelineNode], script_end: u32) {
    use std::collections::HashMap;
    fn labels(nodes: &[TimelineNode], result: &mut HashMap<String, u32>) {
        for node in nodes {
            if node.kind == TimelineNodeKind::Label {
                result.insert(node.name.clone(), node.line);
            }
            labels(&node.children, result);
        }
    }
    fn resolve(
        nodes: &mut [TimelineNode],
        labels: &HashMap<String, u32>,
        loop_range: Option<(u32, u32)>,
        script_end: u32,
    ) {
        for node in nodes {
            if node.kind == TimelineNodeKind::Jump {
                node.target_line = match node.name.as_str() {
                    "break" => loop_range.map(|(_, end)| end),
                    "continue" => loop_range.map(|(start, _)| start),
                    "return" => Some(script_end),
                    name => name
                        .strip_prefix("goto ")
                        .and_then(|name| labels.get(name).copied()),
                };
            }
            let range = if node.kind == TimelineNodeKind::Loop {
                Some((node.target_line.unwrap_or(node.line), node.end_line))
            } else {
                loop_range
            };
            resolve(&mut node.children, labels, range, script_end);
        }
    }
    let mut targets = HashMap::new();
    labels(nodes, &mut targets);
    resolve(nodes, &targets, None, script_end);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(nodes: &[TimelineNode]) -> Vec<&TimelineNode> {
        let mut result = Vec::new();
        for node in nodes {
            result.push(node);
            result.extend(all(&node.children));
        }
        result
    }

    #[test]
    fn loops_keep_nested_ranges_and_relative_time_groups() {
        let entries = analyze("script entry0 {\nloop\n{\n+4:\nfoo(\"loop { interrupt[99]:\");\n+6:\ntimes(3) {\ncontinue;\n}\nbreak;\n}\n}\n");
        assert_eq!(entries.len(), 1);
        let nodes = all(&entries[0].nodes);
        let loops: Vec<_> = nodes
            .iter()
            .filter(|n| n.kind == TimelineNodeKind::Loop)
            .collect();
        assert_eq!(loops.len(), 2);
        assert_eq!((loops[0].line, loops[0].end_line), (2, 11));
        assert_eq!((loops[1].line, loops[1].end_line), (7, 9));
        let four = nodes.iter().find(|n| n.name == "+4").unwrap();
        assert_eq!(four.children[0].name, "foo");
        assert!(four.children[0].source.contains("interrupt[99]"));
        assert!(nodes
            .iter()
            .any(|n| n.kind == TimelineNodeKind::Time && n.name == "+6"));
        assert!(!nodes.iter().any(|n| n.kind == TimelineNodeKind::Interrupt));
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.name == "continue")
                .unwrap()
                .target_line,
            Some(7)
        );
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.name == "break")
                .unwrap()
                .target_line,
            Some(11)
        );
    }

    #[test]
    fn interrupt_aliases_and_jumps_keep_source_order_and_entry_scope() {
        let entries = analyze("script entry0 {\nstart:\n120: foo();\ninterrupt[1]:\n/* an alias */ interrupt[2]:\n+30: bar();\ngoto start @ 0;\n}\nscript entry1 {\nstart:\ninterrupt[-1]: baz();\ngoto start;\n}\n");
        assert_eq!(entries.len(), 2);
        let first = all(&entries[0].nodes);
        let interrupt = first
            .iter()
            .find(|n| n.kind == TimelineNodeKind::Interrupt)
            .unwrap();
        assert_eq!(interrupt.name, "interrupt[1], interrupt[2]");
        assert_eq!((interrupt.line, interrupt.end_line), (4, 5));
        assert!(first
            .iter()
            .any(|n| n.kind == TimelineNodeKind::Time && n.name == "+30"));
        assert_eq!(
            first
                .iter()
                .find(|n| n.kind == TimelineNodeKind::Jump)
                .unwrap()
                .target_line,
            Some(2)
        );
        let second = all(&entries[1].nodes);
        assert_eq!(
            second
                .iter()
                .find(|n| n.kind == TimelineNodeKind::Jump)
                .unwrap()
                .target_line,
            Some(10)
        );
    }

    #[test]
    fn inline_branches_do_while_and_incomplete_edits_preserve_structure() {
        let entries = analyze("// script fake {\nscript main { if (flag) goto done; do { tick(); } while (flag); done: return; }\nscript editing { loop { +5: tick();\n");
        assert_eq!(entries.len(), 2);
        let nodes = all(&entries[0].nodes);
        assert!(nodes
            .iter()
            .any(|n| n.kind == TimelineNodeKind::Loop && n.source.contains("while (flag)")));
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.name == "goto done")
                .unwrap()
                .target_line,
            Some(2)
        );
        assert!(nodes.iter().any(|n| n.name == "return"));
        let editing = all(&entries[1].nodes);
        assert!(editing
            .iter()
            .any(|n| n.kind == TimelineNodeKind::Time && n.name == "+5"));
        assert!(editing.iter().any(|n| n.name == "tick"));
    }

    #[test]
    fn do_while_continue_targets_the_trailing_condition() {
        let entries = analyze("script main {\ndo {\ncontinue;\n}\nwhile (flag);\n}\n");
        let nodes = all(&entries[0].nodes);
        let repeat = nodes
            .iter()
            .find(|n| n.kind == TimelineNodeKind::Loop)
            .unwrap();
        assert_eq!(repeat.target_line, Some(5));
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.name == "continue")
                .unwrap()
                .target_line,
            Some(5)
        );
    }
}
