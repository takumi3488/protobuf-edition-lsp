use crate::parser::{parse_proto, validate_proto};
use tower_lsp::lsp_types::*;

pub fn compute_diagnostics(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Parse the protocol buffer file
    match parse_proto(content) {
        Ok(proto_file) => {
            // Validate the parsed file
            let validation_errors = validate_proto(&proto_file);

            for error in validation_errors {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: error.line as u32,
                            character: error.column as u32,
                        },
                        end: Position {
                            line: error.line as u32,
                            character: error.column as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("protobuf-edition-lsp".to_string()),
                    message: error.message,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
        Err(e) => {
            // Parse error
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("protobuf-edition-lsp".to_string()),
                message: format!("Parse error: {e}"),
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }

    diagnostics
}

pub fn compute_completions(content: &str, position: Position) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Get the line and determine context
    let lines: Vec<&str> = content.lines().collect();
    if let Some(line) = lines.get(position.line as usize) {
        let line_before_cursor = &line[..position.character.min(line.len() as u32) as usize];

        // Check if we're inside a message
        let in_message = is_inside_message(&lines, position.line as usize);

        if in_message {
            // Field type completions
            if line_before_cursor.trim().is_empty()
                || line_before_cursor
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_whitespace())
            {
                // Scalar types
                for scalar_type in &[
                    "double", "float", "int32", "int64", "uint32", "uint64", "sint32", "sint64",
                    "fixed32", "fixed64", "sfixed32", "sfixed64", "bool", "string", "bytes",
                ] {
                    completions.push(CompletionItem {
                        label: scalar_type.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some("Protocol Buffers scalar type".to_string()),
                        documentation: None,
                        ..Default::default()
                    });
                }

                // Field modifiers
                completions.push(CompletionItem {
                    label: "repeated".to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Field modifier for repeated fields".to_string()),
                    ..Default::default()
                });

                completions.push(CompletionItem {
                    label: "optional".to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Field modifier for optional fields".to_string()),
                    ..Default::default()
                });

                completions.push(CompletionItem {
                    label: "oneof".to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("Define a oneof field".to_string()),
                    ..Default::default()
                });
            }
        } else {
            // Top-level keywords
            if line_before_cursor.trim().is_empty() {
                for keyword in &[
                    "syntax", "edition", "package", "import", "message", "enum", "service",
                    "option",
                ] {
                    completions.push(CompletionItem {
                        label: keyword.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(format!("Protocol Buffers {keyword} declaration")),
                        ..Default::default()
                    });
                }
            }
        }
    }

    completions
}

pub fn compute_hover(content: &str, position: Position) -> Option<Hover> {
    let lines: Vec<&str> = content.lines().collect();

    if let Some(line) = lines.get(position.line as usize) {
        let word = get_word_at_position(line, position.character as usize);

        // Provide hover information for scalar types
        let scalar_type_info = match word.as_str() {
            "double" => Some("64-bit floating point number"),
            "float" => Some("32-bit floating point number"),
            "int32" => Some("32-bit signed integer using variable-length encoding"),
            "int64" => Some("64-bit signed integer using variable-length encoding"),
            "uint32" => Some("32-bit unsigned integer using variable-length encoding"),
            "uint64" => Some("64-bit unsigned integer using variable-length encoding"),
            "sint32" => Some("32-bit signed integer using zigzag encoding"),
            "sint64" => Some("64-bit signed integer using zigzag encoding"),
            "fixed32" => Some("32-bit unsigned integer using fixed-width encoding"),
            "fixed64" => Some("64-bit unsigned integer using fixed-width encoding"),
            "sfixed32" => Some("32-bit signed integer using fixed-width encoding"),
            "sfixed64" => Some("64-bit signed integer using fixed-width encoding"),
            "bool" => Some("Boolean value (true or false)"),
            "string" => Some("UTF-8 encoded string"),
            "bytes" => Some("Arbitrary sequence of bytes"),
            _ => None,
        };

        if let Some(info) = scalar_type_info {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{word}**\n\n{info}"),
                }),
                range: None,
            });
        }

        // Provide hover for keywords
        let keyword_info = match word.as_str() {
            "message" => Some("Defines a message type"),
            "enum" => Some("Defines an enumeration"),
            "service" => Some("Defines an RPC service"),
            "repeated" => Some("Field can have zero or more values"),
            "optional" => Some("Field is optional (proto3)"),
            "oneof" => Some("Exactly one field from a set must be set"),
            "syntax" => Some("Specifies the protocol buffer syntax version"),
            "edition" => Some("Specifies the protocol buffer edition (2023)"),
            "package" => Some("Declares the package name"),
            "import" => Some("Imports definitions from another .proto file"),
            _ => None,
        };

        if let Some(info) = keyword_info {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{word}**\n\n{info}"),
                }),
                range: None,
            });
        }
    }

    None
}

fn is_inside_message(lines: &[&str], current_line: usize) -> bool {
    let mut brace_count = 0;
    let mut in_message = false;

    for (i, line) in lines.iter().enumerate() {
        if i > current_line {
            break;
        }

        let trimmed = line.trim();
        if trimmed.starts_with("message ") {
            in_message = true;
        }

        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    in_message = false;
                }
            }
        }
    }

    in_message && brace_count > 0
}

fn get_word_at_position(line: &str, position: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut start = position;
    let mut end = position;

    // Find word boundaries
    while start > 0 && start <= chars.len() && is_word_char(chars.get(start.saturating_sub(1))) {
        start -= 1;
    }

    while end < chars.len() && is_word_char(chars.get(end)) {
        end += 1;
    }

    chars[start..end].iter().collect()
}

fn is_word_char(ch: Option<&char>) -> bool {
    ch.is_some_and(|c| c.is_alphanumeric() || *c == '_')
}
