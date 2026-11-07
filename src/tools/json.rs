use serde::de::IgnoredAny;

use crate::error::{Result, ToolError};
use crate::style::{CYAN, DIM, GREEN, MAGENTA, RED, RESET, YELLOW};

const INVALID_JSON_HINT: &str =
    "JSON uses double-quoted strings and does not allow trailing commas or comments.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Str,
    Number,
    Bool,
    Null,
}

#[derive(Debug, Clone, Copy)]
struct Token<'a> {
    kind: TokenKind,
    text: &'a str,
}

/// Validates JSON and returns it pretty-printed, optionally with ANSI colors.
///
/// Key order, string escapes and number formatting are preserved.
pub fn format(input: &str, indent: usize, color: bool) -> Result<String> {
    validate(input)?;
    let tokens = tokenize(input)?;
    let mut output = String::with_capacity(input.len() + input.len() / 2);
    let pad = " ".repeat(indent.min(16));
    let mut level = 0usize;

    for (index, token) in tokens.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|i| tokens.get(i));
        let next = tokens.get(index + 1);

        match token.kind {
            TokenKind::OpenBrace | TokenKind::OpenBracket => {
                let (open, close) = if token.kind == TokenKind::OpenBrace {
                    ("{", TokenKind::CloseBrace)
                } else {
                    ("[", TokenKind::CloseBracket)
                };
                push_styled(&mut output, color, DIM, open);
                if next.map(|t| t.kind) != Some(close) {
                    level += 1;
                    newline(&mut output, level, &pad);
                }
            }
            TokenKind::CloseBrace | TokenKind::CloseBracket => {
                let (close, open) = if token.kind == TokenKind::CloseBrace {
                    ("}", TokenKind::OpenBrace)
                } else {
                    ("]", TokenKind::OpenBracket)
                };
                let empty = previous.map(|t| t.kind) == Some(open);
                if !empty {
                    level = level.saturating_sub(1);
                    newline(&mut output, level, &pad);
                }
                push_styled(&mut output, color, DIM, close);
            }
            TokenKind::Comma => {
                push_styled(&mut output, color, DIM, ",");
                newline(&mut output, level, &pad);
            }
            TokenKind::Colon => push_styled(&mut output, color, DIM, ": "),
            TokenKind::Str => {
                let is_key = next.map(|t| t.kind) == Some(TokenKind::Colon);
                let code = if is_key { CYAN } else { GREEN };
                push_styled(&mut output, color, code, token.text);
            }
            TokenKind::Number => push_styled(&mut output, color, YELLOW, token.text),
            TokenKind::Bool => push_styled(&mut output, color, MAGENTA, token.text),
            TokenKind::Null => push_styled(&mut output, color, RED, token.text),
        }
    }

    Ok(output)
}

/// Validates JSON and removes every insignificant whitespace character.
pub fn minify(input: &str) -> Result<String> {
    validate(input)?;
    let tokens = tokenize(input)?;
    Ok(tokens.iter().map(|token| token.text).collect())
}

fn validate(input: &str) -> Result<()> {
    serde_json::from_str::<IgnoredAny>(input)
        .map(|_| ())
        .map_err(|error| {
            ToolError::new(format!("Invalid JSON: {error}")).with_hint(INVALID_JSON_HINT)
        })
}

fn newline(output: &mut String, level: usize, pad: &str) {
    output.push('\n');
    for _ in 0..level {
        output.push_str(pad);
    }
}

fn push_styled(output: &mut String, color: bool, code: &str, text: &str) {
    if color {
        output.push_str(code);
        output.push_str(text);
        output.push_str(RESET);
    } else {
        output.push_str(text);
    }
}

fn tokenize(input: &str) -> Result<Vec<Token<'_>>> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'{' => {
                tokens.push(Token {
                    kind: TokenKind::OpenBrace,
                    text: "{",
                });
                i += 1;
            }
            b'}' => {
                tokens.push(Token {
                    kind: TokenKind::CloseBrace,
                    text: "}",
                });
                i += 1;
            }
            b'[' => {
                tokens.push(Token {
                    kind: TokenKind::OpenBracket,
                    text: "[",
                });
                i += 1;
            }
            b']' => {
                tokens.push(Token {
                    kind: TokenKind::CloseBracket,
                    text: "]",
                });
                i += 1;
            }
            b':' => {
                tokens.push(Token {
                    kind: TokenKind::Colon,
                    text: ":",
                });
                i += 1;
            }
            b',' => {
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    text: ",",
                });
                i += 1;
            }
            b'"' => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i += 2,
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Str,
                    text: &input[start..i],
                });
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < bytes.len()
                    && matches!(bytes[i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
                {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Number,
                    text: &input[start..i],
                });
            }
            b't' if input[i..].starts_with("true") => {
                tokens.push(Token {
                    kind: TokenKind::Bool,
                    text: "true",
                });
                i += 4;
            }
            b'f' if input[i..].starts_with("false") => {
                tokens.push(Token {
                    kind: TokenKind::Bool,
                    text: "false",
                });
                i += 5;
            }
            b'n' if input[i..].starts_with("null") => {
                tokens.push(Token {
                    kind: TokenKind::Null,
                    text: "null",
                });
                i += 4;
            }
            other => {
                return Err(ToolError::new(format!(
                    "Unexpected character `{}` in JSON.",
                    other as char
                ))
                .with_hint(INVALID_JSON_HINT));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{format, minify};
    use crate::error::Result;

    #[test]
    fn pretty_prints_and_preserves_key_order() -> Result<()> {
        let input = r#"{"z":1,"a":[true,null,{"k":"v"}]}"#;
        let expected = "{\n  \"z\": 1,\n  \"a\": [\n    true,\n    null,\n    {\n      \"k\": \"v\"\n    }\n  ]\n}";
        assert_eq!(format(input, 2, false)?, expected);
        Ok(())
    }

    #[test]
    fn minifies_whitespace_inside_and_outside_strings_correctly() -> Result<()> {
        let input = "{ \"message\" : \"hello, world\" , \"n\" : 1e3 }";
        assert_eq!(minify(input)?, r#"{"message":"hello, world","n":1e3}"#);
        Ok(())
    }

    #[test]
    fn pretty_prints_empty_containers() -> Result<()> {
        assert_eq!(format("{}", 2, false)?, "{}");
        assert_eq!(format("[[]]", 4, false)?, "[\n    []\n]");
        Ok(())
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(format("{\"a\":1,}", 2, false).is_err());
        assert!(minify("{").is_err());
    }
}
